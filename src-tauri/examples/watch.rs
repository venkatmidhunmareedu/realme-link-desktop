//! Live watcher (read-only): prints battery / wear changes and every notification the buds push.
//! `cargo run -p realme-link-desktop --example watch -- <MAC> [seconds=120]`
//! Try: take buds out, put one in the case, both in the case, close the lid, plug/unplug the case cable.
#[path = "../src/transport.rs"]
mod transport;

use buds_protocol::{caps::Capabilities, cmd::*, frame, packet::Packet, parse};
use std::time::Duration;

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect::<Vec<_>>().join(" ")
}

async fn send(link: &transport::Link, tid: &mut u8, cmd: u16, body: Vec<u8>) {
    *tid = tid.wrapping_add(1);
    for f in frame::encode(&Packet::new(cmd, *tid, body).encode(), frame::DEFAULT_MAX_FRAME) {
        let _ = link.tx.send(f).await;
    }
}

#[tokio::main]
async fn main() {
    let mut a = std::env::args().skip(1);
    let addr = a.next().expect("usage: watch <MAC> [seconds]");
    let secs: u64 = a.next().map_or(120, |s| s.parse().expect("seconds"));
    let mut link = transport::connect_bluez(&addr, buds_protocol::UUID_DEFAULT).await.expect("connect");
    let (mut de, mut tid) = (frame::Deframer::default(), 0u8);
    let t0 = tokio::time::Instant::now();
    let stamp = move || format!("[{:>5.1}s]", t0.elapsed().as_secs_f32());
    let (mut last_batt, mut last_wear) = (String::new(), String::new());
    let mut caps = Capabilities::default();

    send(&link, &mut tid, GET_CAPS, vec![]).await;
    let mut tick = tokio::time::interval(Duration::from_secs(2));
    let end = t0 + Duration::from_secs(secs);
    println!("watching for {secs}s; move the buds and the case around now...");

    while tokio::time::Instant::now() < end {
        tokio::select! {
            _ = tick.tick() => {
                send(&link, &mut tid, GET_BATTERY, vec![]).await;
                send(&link, &mut tid, GET_WEAR, vec![]).await;
            }
            Some(bytes) = link.rx.recv() => {
                for raw in de.push(&bytes) {
                    let Some(p) = Packet::decode(&raw) else { continue };
                    let ok = p.status() == Some(0);
                    let d = if ok { &p.data[1..] } else { &p.data[..] };
                    match (p.base_cmd(), p.is_response()) {
                        (GET_CAPS, true) if ok => {
                            caps = Capabilities::from_bitmap(d);
                            // one-off raw dump of getters whose meaning we haven't resolved
                            for c in [0x0101u16, 0x0102, 0x0103, 0x0104, 0x0105, 0x0107, 0x010D, 0x0128, 0x0111, 0x0112] {
                                if caps.supports(c) { send(&link, &mut tid, c, vec![]).await; tokio::time::sleep(Duration::from_millis(60)).await; }
                            }
                            send(&link, &mut tid, NOTIFY_GET_CAPS, vec![]).await;
                        }
                        (NOTIFY_GET_CAPS, true) if ok => {
                            println!("{} notification types offered: {:02x?}", stamp(), d);
                            for ty in d.iter().skip(1) { send(&link, &mut tid, NOTIFY_REGISTER, vec![*ty]).await; tokio::time::sleep(Duration::from_millis(60)).await; }
                        }
                        (GET_BATTERY, true) if ok => {
                            let s = format!("{:?}", parse::battery(d).iter().map(|b| format!("dev{}={}%{}", b.device, b.level, if b.charging { " CHARGING" } else { "" })).collect::<Vec<_>>());
                            if s != last_batt { println!("{} battery  {s}   raw [{}]", stamp(), hex(d)); last_batt = s; }
                        }
                        (GET_WEAR, true) if ok => {
                            let s = format!("{:?}", parse::wear(d).iter().map(|w| format!("dev{} in_box={} in_ear={}", w.device, w.in_box, w.in_ear)).collect::<Vec<_>>());
                            if s != last_wear { println!("{} wear     {s}   raw [{}]", stamp(), hex(d)); last_wear = s; }
                        }
                        (c, true) if [0x0101u16, 0x0102, 0x0103, 0x0104, 0x0105, 0x0107, 0x010D, 0x0128, 0x0111, 0x0112].contains(&c) =>
                            println!("{} reply {c:04x} status {:?} data [{}]", stamp(), p.status(), hex(&p.data)),
                        (NOTIFY, false) => println!("{} NOTIFY   [{}]", stamp(), hex(&p.data)),
                        (_, false) => println!("{} device-initiated {:04x} tid {} [{}]", stamp(), p.cmd, p.tid, hex(&p.data)),
                        _ => {}
                    }
                }
            }
        }
    }
    link.abort();
    println!("done");
}
