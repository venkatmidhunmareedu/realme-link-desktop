//! A fake earbud speaking the real wire protocol, so the UI can be built and tested without hardware.
use crate::transport::Link;
use buds_protocol::{cmd::*, frame, packet::Packet};
use std::collections::BTreeMap;
use tokio::sync::mpsc;

struct Buds {
    noise: u8,
    eq: u8,
    high_vol: u8,
    features: BTreeMap<u8, u32>,
    keys: Vec<[u8; 4]>,
}

impl Buds {
    fn new() -> Self {
        Self {
            noise: 8,
            eq: 1,
            high_vol: 0,
            features: BTreeMap::from([(4, 0), (7, 0), (17, 1), (24, 1), (27, 0)]),
            keys: vec![[1, 1, 2, 1], [1, 1, 3, 6], [1, 1, 4, 8], [2, 1, 2, 1], [2, 1, 3, 6], [2, 1, 4, 8]],
        }
    }

    fn eq_blob() -> Vec<u8> {
        let presets = [(1u8, "Balanced", [0i8, 0, 0, 0, 0, 0]), (2, "Bass Boost", [6, 4, 1, 0, 0, 0]), (3, "Clear Vocals", [-2, 0, 3, 4, 2, 0])];
        let freqs = [100u16, 170, 400, 4300, 9500, 12000];
        let mut d = vec![presets.len() as u8];
        for (id, name, db) in presets {
            d.extend_from_slice(&[(id == 1) as u8, (-10i8) as u8, 10, id, name.len() as u8]);
            d.extend_from_slice(name.as_bytes());
            d.push(freqs.len() as u8);
            for (f, g) in freqs.iter().zip(db) {
                d.extend_from_slice(&f.to_le_bytes());
                d.push(g as u8);
            }
        }
        d
    }

    /// Returns reply body (after the status byte), or None if unknown.
    fn handle(&mut self, p: &Packet) -> Option<Vec<u8>> {
        let ok = |mut v: Vec<u8>| {
            v.insert(0, 0);
            Some(v)
        };
        match p.cmd {
            GET_CAPS => ok(vec![0xFF; 8]),
            GET_BATTERY => ok(vec![3, 1, 82, 2, 0x80 | 74, 3, 100]),
            GET_WEAR => ok(vec![3, 1, 3, 2, 0, 3, 4]),
            GET_KEYMAP => ok(std::iter::once(self.keys.len() as u8).chain(self.keys.iter().flatten().copied()).collect()),
            GET_COLOR => ok(vec![2]),
            GET_NOISE => ok(vec![1, 1, self.noise]),
            GET_EQ_ID => ok(vec![self.eq]),
            GET_HIGH_VOL => ok(vec![self.high_vol]),
            GET_ALL_EQ => ok(Self::eq_blob()),
            GET_FEATURES => {
                let mut v = vec![self.features.len() as u8];
                for (k, x) in &self.features {
                    v.push(*k);
                    v.extend_from_slice(&x.to_le_bytes());
                }
                ok(v)
            }
            SET_NOISE if p.data.len() >= 3 => {
                self.noise = p.data[2];
                ok(vec![])
            }
            SET_EQ_ID if !p.data.is_empty() => {
                self.eq = p.data[0];
                ok(vec![])
            }
            SET_HIGH_VOL if !p.data.is_empty() => {
                self.high_vol = p.data[0];
                ok(vec![])
            }
            SET_FEATURE if p.data.len() >= 2 => {
                self.features.insert(p.data[0], p.data[1] as u32);
                ok(vec![])
            }
            SET_KEYMAP => {
                self.keys = p.data[1..].chunks_exact(4).map(|c| [c[0], c[1], c[2], c[3]]).collect();
                ok(vec![])
            }
            SET_FIND | NOTIFY_REGISTER => ok(vec![]),
            _ => None,
        }
    }
}

pub fn spawn() -> Link {
    let (to_dev, mut dev_rx) = mpsc::channel::<Vec<u8>>(64);
    let (dev_tx, from_dev) = mpsc::channel::<Vec<u8>>(64);
    let task = tokio::spawn(async move {
        let (mut buds, mut de) = (Buds::new(), frame::Deframer::default());
        while let Some(bytes) = dev_rx.recv().await {
            for raw in de.push(&bytes) {
                let Some(req) = Packet::decode(&raw) else { continue };
                if let Some(body) = buds.handle(&req) {
                    let rsp = Packet::new(req.cmd | 0x8000, req.tid, body).encode();
                    for f in frame::encode(&rsp, frame::DEFAULT_MAX_FRAME) {
                        let _ = dev_tx.send(f).await;
                    }
                }
            }
        }
    });
    Link { tx: to_dev, rx: from_dev, tasks: vec![task.abort_handle()] }
}
