use crate::caps_table::CAP_TABLE;
use std::collections::HashSet;

/// Commands that are always allowed (`Protocol.c2`).
const ALWAYS: [u16; 7] = [0x100, 0x101, 0x102, 0x103, 0x104, 0x10B, 0xF00];

#[derive(Debug, Default, Clone)]
pub struct Capabilities(HashSet<u16>);

impl Capabilities {
    /// `bitmap` is the 0x8100 reply data after the status byte; bit i (LSB first) enables `CAP_TABLE[i]`.
    pub fn from_bitmap(bitmap: &[u8]) -> Self {
        let mut s: HashSet<u16> = ALWAYS.into_iter().collect();
        for (i, cmds) in CAP_TABLE.iter().enumerate().take(bitmap.len() * 8) {
            if bitmap[i / 8] & (1 << (i % 8)) != 0 {
                s.extend(cmds.iter().copied());
            }
        }
        Self(s)
    }

    pub fn supports(&self, cmd: u16) -> bool {
        self.0.contains(&cmd)
    }

    /// Everything the table can express; used by the mock device.
    pub fn all() -> Self {
        Self(CAP_TABLE.iter().flat_map(|c| c.iter().copied()).chain(ALWAYS).collect())
    }

    pub fn list(&self) -> Vec<u16> {
        let mut v: Vec<u16> = self.0.iter().copied().collect();
        v.sort();
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_map_to_commands() {
        // bit 1 -> 0x0106 (battery), bit 8 -> 0x010C,0x0404 (noise)
        let c = Capabilities::from_bitmap(&[0b0000_0010, 0b0000_0001]);
        assert!(c.supports(0x106) && c.supports(0x404) && c.supports(0x10C));
        assert!(!c.supports(0x400));
        assert!(c.supports(0x100));
    }
}
