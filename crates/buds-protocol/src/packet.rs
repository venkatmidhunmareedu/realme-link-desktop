/// Inner packet: cmd u16 LE | transferId u8 | len u16 LE | data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub cmd: u16,
    pub tid: u8,
    pub data: Vec<u8>,
}

impl Packet {
    pub fn new(cmd: u16, tid: u8, data: Vec<u8>) -> Self {
        Self { cmd, tid, data }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(5 + self.data.len());
        v.extend_from_slice(&self.cmd.to_le_bytes());
        v.push(self.tid);
        v.extend_from_slice(&(self.data.len() as u16).to_le_bytes());
        v.extend_from_slice(&self.data);
        v
    }

    /// The app takes everything after the 5-byte header regardless of the declared length; so do we.
    pub fn decode(b: &[u8]) -> Option<Self> {
        if b.len() < 5 {
            return None;
        }
        Some(Self { cmd: u16::from_le_bytes([b[0], b[1]]), tid: b[2], data: b[5..].to_vec() })
    }

    pub fn is_response(&self) -> bool {
        self.cmd & 0x8000 != 0
    }

    /// Request id with the response bit stripped.
    pub fn base_cmd(&self) -> u16 {
        self.cmd & 0x7FFF
    }

    /// Responses lead with a status byte, 0 = OK.
    pub fn status(&self) -> Option<u8> {
        if self.is_response() { self.data.first().copied() } else { None }
    }
}

/// Per-device transfer-id counter (wraps at 255).
#[derive(Default)]
pub struct Tid(u8);

impl Tid {
    pub fn next(&mut self) -> u8 {
        let t = self.0;
        self.0 = self.0.wrapping_add(1);
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip() {
        let p = Packet::new(0x0106, 7, vec![1, 2]);
        assert_eq!(p.encode(), vec![0x06, 0x01, 7, 2, 0, 1, 2]);
        assert_eq!(Packet::decode(&p.encode()), Some(p));
        let r = Packet::new(0x8106, 7, vec![0, 1]);
        assert!(r.is_response() && r.base_cmd() == 0x0106 && r.status() == Some(0));
    }
}
