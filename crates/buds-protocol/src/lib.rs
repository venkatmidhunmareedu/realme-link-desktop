//! Realme/OPPO "TL" earbuds protocol, reverse-engineered from the Realme Link app.
//! See PROTOCOL.md at the repo root.
mod caps_table;
pub mod caps;
pub mod cmd;
pub mod frame;
pub mod model;
pub mod packet;
pub mod parse;

/// Classic-BT SPP UUIDs the app picks per model (`sppUUIDType`).
pub const UUID_DEFAULT: &str = "00001101-0000-1000-8000-00805F9B34FB";
pub const UUID_SELF_PROTOCOL: &str = "0000079A-D102-11E1-9B23-00025B00A5A5";
