//! Read-only probe against real earbuds: `cargo run -p realme-link-desktop --example probe -- <MAC> [uuid]`
//! Sends only GET commands and prints raw + parsed replies. Never writes settings.
#[path = "../src/transport.rs"]
mod transport;

use buds_protocol::{caps::Capabilities, cmd::*, frame, packet::Packet, parse};
use std::time::Duration;

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect::<Vec<_>>().join(" ")
}

#[tokio::main]
async fn main() {
    let mut a = std::env::args().skip(1);
    let addr = a.next().expect("usage: probe <MAC> [uuid]");
    let uuid = a.next().unwrap_or_else(|| buds_protocol::UUID_DEFAULT.to_string());
    println!("connecting {addr} on {uuid} ...");
    let mut link = match transport::connect_bluez(&addr, &uuid).await {
        Ok(l) => l,
        Err(e) => return println!("CONNECT FAILED: {e}"),
    };
    println!("connected; RFCOMM open");

    let send = |link: &transport::Link, p: Packet| {
        println!("-> cmd {:04x} tid {} data [{}]", p.cmd, p.tid, hex(&p.data));
        let tx = link.tx.clone();
        let frames = frame::encode(&p.encode(), frame::DEFAULT_MAX_FRAME);
        async move {
            for f in frames {
                println!("   wire [{}]", hex(&f));
                let _ = tx.send(f).await;
            }
        }
    };

    let mut tid = 0u8;
    let mut de = frame::Deframer::default();
    let mut caps = Capabilities::default();
    send(&link, get(GET_CAPS, tid)).await;

    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
    let mut queue: Vec<u16> = vec![];
    while tokio::time::Instant::now() < deadline {
        let Ok(Some(bytes)) = tokio::time::timeout(Duration::from_millis(1500), link.rx.recv()).await else {
            if let Some(c) = queue.pop() {
                tid += 1;
                send(&link, Packet::new(c, tid, get_body(c))).await;
                continue;
            }
            break;
        };
        println!("<- raw [{}]", hex(&bytes));
        for payload in de.push(&bytes) {
            let Some(p) = Packet::decode(&payload) else {
                println!("   (undecodable payload [{}])", hex(&payload));
                continue;
            };
            println!("<- cmd {:04x} tid {} status {:?} data [{}]", p.cmd, p.tid, p.status(), hex(&p.data));
            if !p.is_response() || p.status() != Some(0) {
                continue;
            }
            let d = &p.data[1..];
            match p.base_cmd() {
                GET_CAPS => {
                    caps = Capabilities::from_bitmap(d);
                    println!("   capabilities: {:04x?}", caps.list());
                    queue = [GET_BATTERY, GET_WEAR, GET_NOISE, GET_KEYMAP, GET_COLOR, GET_EQ_ID, GET_ALL_EQ, GET_HIGH_VOL, GET_FEATURES, GET_AUTO_OFF]
                        .into_iter()
                        .filter(|c| caps.supports(*c))
                        .rev()
                        .collect();
                }
                GET_BATTERY => println!("   battery {:?}", parse::battery(d)),
                GET_WEAR => println!("   wear {:?}", parse::wear(d)),
                GET_NOISE => println!("   noise {:?}", parse::noise(d)),
                GET_KEYMAP => println!("   keymap {:?}", parse::keymap(d)),
                GET_EQ_ID => println!("   eq id {:?}", d.first()),
                GET_ALL_EQ => println!("   eq presets {:?}", parse::eq_presets(d)),
                GET_FEATURES => println!("   features {:?}", parse::feature_states(d)),
                _ => {}
            }
        }
    }
    println!("done");
}
