//! Command ids and request builders. Layouts are documented in PROTOCOL.md.
use crate::packet::Packet;
use crate::parse::KeyBinding;

pub const GET_CAPS: u16 = 0x0100;
pub const GET_BATTERY: u16 = 0x0106;
pub const GET_KEYMAP: u16 = 0x0108;
pub const GET_WEAR: u16 = 0x0109;
pub const GET_COLOR: u16 = 0x010B;
pub const GET_NOISE: u16 = 0x010C;
pub const GET_EQ_ID: u16 = 0x010F;
pub const GET_HIGH_VOL: u16 = 0x0110;
pub const GET_AUTO_OFF: u16 = 0x0111;
pub const GET_ALL_EQ: u16 = 0x0122;
pub const GET_BASS: u16 = 0x0124;
pub const GET_FEATURES: u16 = 0x0129;
pub const NOTIFY_GET_CAPS: u16 = 0x0200;
pub const NOTIFY_REGISTER: u16 = 0x0201;
pub const NOTIFY: u16 = 0x0204;
pub const SET_FIND: u16 = 0x0400;
pub const SET_KEYMAP: u16 = 0x0401;
pub const SET_FEATURE: u16 = 0x0403;
pub const SET_NOISE: u16 = 0x0404;
pub const SET_EQ_ID: u16 = 0x0406;
pub const SET_HIGH_VOL: u16 = 0x0407;
pub const SET_AUTO_OFF: u16 = 0x040A;
pub const SET_BASS: u16 = 0x041B;

/// Body the app sends with a GET for `cmd` (most are empty; noise control is queried with `[action=1, type=1]`).
pub fn get_body(cmd: u16) -> Vec<u8> {
    if cmd == GET_NOISE { vec![1, 1] } else { vec![] }
}

/// Request with an empty body.
pub fn get(cmd: u16, tid: u8) -> Packet {
    Packet::new(cmd, tid, vec![])
}

pub fn set_bool(cmd: u16, tid: u8, on: bool) -> Packet {
    Packet::new(cmd, tid, vec![on as u8])
}

pub fn set_u8(cmd: u16, tid: u8, v: u8) -> Packet {
    Packet::new(cmd, tid, vec![v])
}

pub fn set_feature(tid: u8, feature: u8, on: bool) -> Packet {
    Packet::new(SET_FEATURE, tid, vec![feature, on as u8])
}

/// `action` 1 = select mode, 2 = configure cycle; `ty` is 1 or 2 depending on model.
pub fn set_noise(tid: u8, action: u8, ty: u8, value: u8) -> Packet {
    Packet::new(SET_NOISE, tid, vec![action, ty, value])
}

pub fn set_keymap(tid: u8, keys: &[KeyBinding]) -> Packet {
    let mut d = vec![keys.len() as u8];
    for k in keys {
        d.extend_from_slice(&[k.device, k.button, k.action, k.function]);
    }
    Packet::new(SET_KEYMAP, tid, d)
}

pub fn register_notification(tid: u8, ty: u8) -> Packet {
    Packet::new(NOTIFY_REGISTER, tid, vec![ty])
}
