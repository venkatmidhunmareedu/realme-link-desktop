//! Reconnect test: `reconnect_test <MAC> leak|close`
//! `leak`  = drop the Link without aborting its tasks (the old Disconnect bug) -> 2nd connect should fail.
//! `close` = Link::abort() before dropping (the fix) -> 2nd connect should work.
#[path = "../src/transport.rs"]
mod transport;

use buds_protocol::{cmd::*, frame, packet::Packet};
use std::time::Duration;

async fn caps_ok(link: &mut transport::Link) -> bool {
    for f in frame::encode(&get(GET_CAPS, 1).encode(), frame::DEFAULT_MAX_FRAME) {
        let _ = link.tx.send(f).await;
    }
    let mut de = frame::Deframer::default();
    let end = tokio::time::Instant::now() + Duration::from_secs(4);
    while tokio::time::Instant::now() < end {
        if let Ok(Some(b)) = tokio::time::timeout(Duration::from_millis(500), link.rx.recv()).await {
            if de.push(&b).iter().filter_map(|p| Packet::decode(p)).any(|p| p.cmd == 0x8100) {
                return true;
            }
        }
    }
    false
}

#[tokio::main]
async fn main() {
    let mut a = std::env::args().skip(1);
    let addr = a.next().expect("usage: reconnect_test <MAC> leak|close");
    let mode = a.next().unwrap_or_else(|| "close".into());
    let uuid = buds_protocol::UUID_DEFAULT;

    let mut l1 = transport::connect_bluez(&addr, uuid).await.expect("first connect");
    println!("1st connect: ok, caps reply = {}", caps_ok(&mut l1).await);
    if mode == "close" {
        l1.abort();
    }
    drop(l1);
    println!("disconnected ({mode}); waiting 1s");
    tokio::time::sleep(Duration::from_secs(1)).await;

    match transport::connect_bluez(&addr, uuid).await {
        Ok(mut l2) => println!("2nd connect: ok, caps reply = {}", caps_ok(&mut l2).await),
        Err(e) => println!("2nd connect: FAILED: {e}"),
    }
}
