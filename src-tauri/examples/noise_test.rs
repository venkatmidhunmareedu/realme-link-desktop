//! Hardware test of the noise-mode SET: read current -> set `target` -> read back -> ALWAYS restore -> verify.
//! `cargo run -p realme-link-desktop --example noise_test -- <MAC> [target-wire-value=1]`
#[path = "../src/transport.rs"]
mod transport;

use buds_protocol::{cmd::*, frame, packet::Packet, parse};
use std::time::Duration;

struct Dev {
    link: transport::Link,
    de: frame::Deframer,
    tid: u8,
}

impl Dev {
    /// Send a request and wait for its reply (matched on cmd|0x8000 and tid).
    async fn call(&mut self, cmd: u16, body: Vec<u8>) -> Result<Packet, String> {
        self.tid = self.tid.wrapping_add(1);
        let tid = self.tid;
        for f in frame::encode(&Packet::new(cmd, tid, body.clone()).encode(), frame::DEFAULT_MAX_FRAME) {
            self.link.tx.send(f).await.map_err(|e| e.to_string())?;
        }
        println!("-> {cmd:04x} {body:02x?}");
        let end = tokio::time::Instant::now() + Duration::from_secs(5);
        while tokio::time::Instant::now() < end {
            let Ok(Some(bytes)) = tokio::time::timeout(Duration::from_millis(500), self.link.rx.recv()).await else { continue };
            for raw in self.de.push(&bytes) {
                if let Some(p) = Packet::decode(&raw) {
                    println!("<- {:04x} tid {} {:02x?}", p.cmd, p.tid, p.data);
                    if p.cmd == cmd | 0x8000 && p.tid == tid {
                        return Ok(p);
                    }
                }
            }
        }
        Err(format!("no reply to {cmd:04x}"))
    }

    async fn noise(&mut self) -> Result<u8, String> {
        let p = self.call(GET_NOISE, get_body(GET_NOISE)).await?;
        if p.status() != Some(0) {
            return Err("noise query rejected".into());
        }
        parse::noise(&p.data[1..]).map(|n| n.value).ok_or_else(|| "short noise reply".into())
    }

    async fn set(&mut self, v: u8) -> Result<bool, String> {
        Ok(self.call(SET_NOISE, vec![1, 1, v]).await?.status() == Some(0))
    }
}

#[tokio::main]
async fn main() {
    let mut a = std::env::args().skip(1);
    let addr = a.next().expect("usage: noise_test <MAC> [target]");
    let target: u8 = a.next().map_or(1, |s| s.parse().expect("target must be a number"));
    let link = transport::connect_bluez(&addr, buds_protocol::UUID_DEFAULT).await.expect("connect");
    let mut d = Dev { link, de: frame::Deframer::default(), tid: 0 };
    d.call(GET_CAPS, vec![]).await.expect("caps");

    let original = d.noise().await.expect("read original");
    println!("ORIGINAL noise value = {original}");

    let test = async {
        println!("set -> {target}: accepted = {}", d.set(target).await?);
        tokio::time::sleep(Duration::from_millis(600)).await;
        let now = d.noise().await?;
        println!("READ BACK = {now}  ({})", if now == target { "MATCHES" } else { "DOES NOT MATCH" });
        Ok::<_, String>(())
    }
    .await;
    if let Err(e) = test {
        println!("TEST ERROR: {e}");
    }

    // Always restore.
    let restored = d.set(original).await;
    tokio::time::sleep(Duration::from_millis(600)).await;
    let after = d.noise().await;
    println!("RESTORE accepted = {restored:?}, now = {after:?}, expected {original} -> {}", if after == Ok(original) { "RESTORED" } else { "NOT RESTORED, set it back manually" });
}
