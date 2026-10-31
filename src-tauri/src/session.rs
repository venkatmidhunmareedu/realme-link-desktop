//! One connected device: speaks the protocol over a `Link` and keeps a `DeviceState` the UI renders.
use crate::transport::Link;
use buds_protocol::{
    caps::Capabilities,
    cmd::*,
    frame,
    model::{ModelConfig, ViewItem},
    packet::{Packet, Tid},
    parse::{self, Battery, EqPreset, KeyBinding, Noise, Wear},
};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

#[derive(Serialize, Default, Clone)]
pub struct DeviceState {
    pub connected: bool,
    pub mock: bool,
    pub name: String,
    pub address: String,
    /// Catalog key the device matched, if any.
    pub model: Option<String>,
    pub view_items: Vec<ViewItem>,
    pub product: serde_json::Value,
    pub supported: Vec<u16>,
    pub battery: Vec<Battery>,
    pub wear: Vec<Wear>,
    pub noise: Option<Noise>,
    pub eq_id: Option<u8>,
    pub eq: Vec<EqPreset>,
    pub keymap: Vec<KeyBinding>,
    pub features: BTreeMap<u8, u32>,
    pub high_volume: Option<bool>,
    pub color: Option<u8>,
}

pub struct Session {
    out: mpsc::Sender<Vec<u8>>,
    tid: Mutex<Tid>,
    caps: Mutex<Capabilities>,
    pub state: Mutex<DeviceState>,
    interval: Duration,
    app: AppHandle,
    tasks: Mutex<Vec<tokio::task::AbortHandle>>,
}

impl Session {
    pub fn start(app: AppHandle, link: Link, init: DeviceState, cfg: Option<&ModelConfig>) -> Arc<Self> {
        let interval = Duration::from_millis(cfg.map_or(10, |c| c.android.cmd_interval_ms.max(10)) as u64);
        let s = Arc::new(Self {
            out: link.tx,
            tid: Mutex::new(Tid::default()),
            caps: Mutex::new(Capabilities::default()),
            state: Mutex::new(DeviceState { connected: true, ..init }),
            interval,
            app,
            tasks: Mutex::new(link.tasks),
        });
        let me = s.clone();
        let mut rx = link.rx;
        let reader = tokio::spawn(async move {
            let mut de = frame::Deframer::default();
            while let Some(bytes) = rx.recv().await {
                for raw in de.push(&bytes) {
                    if let Some(p) = Packet::decode(&raw) {
                        me.on_packet(p).await;
                    }
                }
            }
            me.state.lock().unwrap().connected = false;
            me.emit();
        });
        let me = s.clone();
        let kick = tokio::spawn(async move { me.raw_send(get(GET_CAPS, me.next_tid())).await });
        s.tasks.lock().unwrap().extend([reader.abort_handle(), kick.abort_handle()]);
        s
    }

    /// Tear the connection down for real. The reader task holds an `Arc<Session>`, so simply dropping the
    /// handle would leave the RFCOMM socket open and block reconnecting.
    pub fn close(&self) {
        self.tasks.lock().unwrap().drain(..).for_each(|t| t.abort());
        self.state.lock().unwrap().connected = false;
    }

    fn next_tid(&self) -> u8 {
        self.tid.lock().unwrap().next()
    }

    fn emit(&self) {
        let st = self.state.lock().unwrap().clone();
        let _ = self.app.emit("state", st);
    }

    async fn raw_send(&self, p: Packet) {
        for f in frame::encode(&p.encode(), frame::DEFAULT_MAX_FRAME) {
            if self.out.send(f).await.is_err() {
                return;
            }
        }
        tokio::time::sleep(self.interval).await;
    }

    /// Sends only if the device advertised support for the command.
    pub async fn send(&self, cmd: u16, data: Vec<u8>) -> bool {
        if !self.caps.lock().unwrap().supports(cmd) {
            return false;
        }
        let tid = self.next_tid();
        self.raw_send(Packet::new(cmd, tid, data)).await;
        true
    }

    async fn refresh_all(&self) {
        for c in [GET_BATTERY, GET_WEAR, GET_NOISE, GET_KEYMAP, GET_COLOR, GET_EQ_ID, GET_ALL_EQ, GET_HIGH_VOL, GET_FEATURES] {
            self.send(c, get_body(c)).await;
        }
        self.send(NOTIFY_GET_CAPS, vec![]).await;
    }

    async fn on_packet(&self, p: Packet) {
        // Device-initiated time sync: it expects `[0, u32 LE unix seconds]` back.
        if !p.is_response() && p.cmd == 0x0500 {
            let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs()) as u32;
            let mut d = vec![0];
            d.extend_from_slice(&secs.to_le_bytes());
            self.raw_send(Packet::new(p.cmd | 0x8000, p.tid, d)).await;
            return;
        }
        if p.cmd == NOTIFY {
            self.on_notification(&p.data);
            self.emit();
            return;
        }
        if !p.is_response() || p.status() != Some(0) {
            return;
        }
        let d = &p.data[1..];
        match p.base_cmd() {
            GET_CAPS => {
                let caps = Capabilities::from_bitmap(d);
                self.state.lock().unwrap().supported = caps.list();
                *self.caps.lock().unwrap() = caps;
                self.refresh_all().await;
            }
            NOTIFY_GET_CAPS => {
                for ty in d.iter().skip(1).take(d.first().copied().unwrap_or(0) as usize) {
                    self.send(NOTIFY_REGISTER, vec![*ty]).await;
                }
            }
            c => {
                let mut s = self.state.lock().unwrap();
                match c {
                    GET_BATTERY => s.battery = parse::battery(d),
                    GET_WEAR => s.wear = parse::wear(d),
                    GET_NOISE => s.noise = parse::noise(d),
                    GET_KEYMAP => s.keymap = parse::keymap(d),
                    GET_COLOR => s.color = d.first().copied(),
                    GET_EQ_ID => s.eq_id = d.first().copied(),
                    GET_ALL_EQ => s.eq = parse::eq_presets(d).unwrap_or_default(),
                    GET_HIGH_VOL => s.high_volume = d.first().map(|b| *b != 0),
                    GET_FEATURES => s.features = parse::feature_states(d).into_iter().collect(),
                    _ => return,
                }
            }
        }
        self.emit();
    }

    /// 0x0204 push: `[type, ...]` (see PROTOCOL.md).
    fn on_notification(&self, d: &[u8]) {
        let Some((&ty, body)) = d.split_first() else { return };
        let mut s = self.state.lock().unwrap();
        match ty {
            1 => s.battery = parse::battery(body),
            2 => s.wear = parse::wear(body),
            3 => {
                s.noise = match body {
                    [t, v] => Some(Noise { action: 1, ty: *t, value: *v }),
                    _ => parse::noise(body),
                }
            }
            _ => {}
        }
    }

    // ---- user actions: set, then read back (the official app does the same) ----

    pub async fn set_noise(&self, wire: u8) {
        if self.send(SET_NOISE, vec![1, 1, wire]).await {
            self.send(GET_NOISE, get_body(GET_NOISE)).await;
        }
    }

    pub async fn set_feature(&self, id: u8, on: bool) {
        if self.send(SET_FEATURE, vec![id, on as u8]).await {
            self.send(GET_FEATURES, vec![]).await;
        }
    }

    pub async fn set_eq(&self, id: u8) {
        if self.send(SET_EQ_ID, vec![id]).await {
            self.send(GET_EQ_ID, vec![]).await;
        }
    }

    pub async fn set_high_volume(&self, on: bool) {
        if self.send(SET_HIGH_VOL, vec![on as u8]).await {
            self.send(GET_HIGH_VOL, vec![]).await;
        }
    }

    pub async fn set_keymap(&self, keys: Vec<KeyBinding>) {
        let p = set_keymap(0, &keys);
        if self.send(SET_KEYMAP, p.data).await {
            self.send(GET_KEYMAP, vec![]).await;
        }
    }

    pub async fn find(&self, on: bool) {
        self.send(SET_FIND, vec![on as u8]).await;
    }
}
