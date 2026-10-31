//! Byte-pipe to a device: real BlueZ RFCOMM, or the in-process mock. Both yield a `Link`.
use bluer::rfcomm::{Profile, Role};
use futures::StreamExt;
use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;

pub struct Link {
    pub tx: mpsc::Sender<Vec<u8>>,
    pub rx: mpsc::Receiver<Vec<u8>>,
    /// Background I/O tasks. Aborting them drops the socket and BlueZ profile registration,
    /// which is what frees the earbuds' single control channel for the next connection.
    pub tasks: Vec<tokio::task::AbortHandle>,
}

impl Link {
    pub fn abort(&self) {
        self.tasks.iter().for_each(|t| t.abort());
    }
}

#[derive(Serialize, Clone)]
pub struct Paired {
    pub name: String,
    pub address: String,
    pub connected: bool,
}

pub async fn paired() -> Result<Vec<Paired>, String> {
    let s = bluer::Session::new().await.map_err(|e| e.to_string())?;
    let a = s.default_adapter().await.map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for addr in a.device_addresses().await.map_err(|e| e.to_string())? {
        let d = a.device(addr).map_err(|e| e.to_string())?;
        if !d.is_paired().await.unwrap_or(false) {
            continue;
        }
        out.push(Paired {
            name: d.name().await.ok().flatten().unwrap_or_default(),
            address: addr.to_string(),
            connected: d.is_connected().await.unwrap_or(false),
        });
    }
    Ok(out)
}

pub async fn connect_bluez(address: &str, uuid: &str) -> Result<Link, String> {
    let session = bluer::Session::new().await.map_err(|e| e.to_string())?;
    let adapter = session.default_adapter().await.map_err(|e| e.to_string())?;
    let addr: bluer::Address = address.parse().map_err(|_| "bad address".to_string())?;
    let device = adapter.device(addr).map_err(|e| e.to_string())?;
    let uuid: bluer::Uuid = uuid.parse().map_err(|_| "bad uuid".to_string())?;

    let profile = Profile { uuid, role: Some(Role::Client), require_authentication: Some(false), ..Default::default() };
    let mut handle = session.register_profile(profile).await.map_err(|e| e.to_string())?;
    // BlueZ hands the socket to the registered profile once ConnectProfile succeeds.
    let dev = device.clone();
    let connecting = tokio::spawn(async move { dev.connect_profile(&uuid).await });
    let req = match tokio::time::timeout(std::time::Duration::from_secs(15), handle.next()).await {
        Ok(Some(r)) => r,
        Ok(None) => return Err("profile closed".into()),
        Err(_) => {
            // Surface BlueZ's own reason when it already gave up.
            if connecting.is_finished() {
                if let Ok(Err(e)) = connecting.await {
                    return Err(format!("BlueZ could not connect: {e}"));
                }
            }
            return Err("timed out waiting for the earbuds' control channel (are they awake and out of the case?)".into());
        }
    };
    let stream = req.accept().map_err(|e| e.to_string())?;
    connecting.abort();

    let (mut rd, mut wr) = tokio::io::split(stream);
    let (in_tx, in_rx) = mpsc::channel(64);
    let (out_tx, mut out_rx) = mpsc::channel::<Vec<u8>>(64);
    let reader = tokio::spawn(async move {
        let _keep = (session, handle); // profile registration must outlive the socket
        let mut buf = [0u8; 2048];
        while let Ok(n) = rd.read(&mut buf).await {
            if n == 0 || in_tx.send(buf[..n].to_vec()).await.is_err() {
                break;
            }
        }
    });
    let writer = tokio::spawn(async move {
        while let Some(b) = out_rx.recv().await {
            if wr.write_all(&b).await.is_err() {
                break;
            }
        }
    });
    Ok(Link { tx: out_tx, rx: in_rx, tasks: vec![reader.abort_handle(), writer.abort_handle()] })
}
