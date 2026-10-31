//! Parsers for reply data (the bytes AFTER the status byte). Truncated input yields empty/None, never a panic.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Battery {
    /// 1 left, 2 right, 3 case
    pub device: u8,
    pub level: u8,
    pub charging: bool,
}

fn records(d: &[u8], size: usize) -> impl Iterator<Item = &[u8]> {
    let n = *d.first().unwrap_or(&0) as usize;
    d.get(1..1 + n * size).unwrap_or(&[]).chunks_exact(size)
}

pub fn battery(d: &[u8]) -> Vec<Battery> {
    records(d, 2)
        .map(|c| Battery { device: c[0], level: c[1] & 0x7F, charging: c[1] & 0x80 != 0 })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Wear {
    /// 1 left, 2 right, 3 case (the case only has an entry while a bud is docked)
    pub device: u8,
    pub in_box: bool,
    pub in_ear: bool,
    /// Lid closed (flag bit 2 clear). Meaningful for the case entry.
    pub cover_closed: bool,
}

pub fn wear(d: &[u8]) -> Vec<Wear> {
    records(d, 2)
        .map(|c| Wear { device: c[0], in_box: c[1] & 1 == 0, in_ear: c[1] & 2 != 0, cover_closed: c[1] & 4 == 0 })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyBinding {
    pub device: u8,
    pub button: u8,
    pub action: u8,
    pub function: u8,
}

pub fn keymap(d: &[u8]) -> Vec<KeyBinding> {
    records(d, 4)
        .map(|c| KeyBinding { device: c[0], button: c[1], action: c[2], function: c[3] })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Noise {
    pub action: u8,
    pub ty: u8,
    pub value: u8,
}

pub fn noise(d: &[u8]) -> Option<Noise> {
    (d.len() >= 3).then(|| Noise { action: d[0], ty: d[1], value: d[2] })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EqPreset {
    pub selected: bool,
    pub min: i8,
    pub max: i8,
    pub id: u8,
    pub name: String,
    /// (freq Hz, dB)
    pub points: Vec<(u16, i8)>,
}

pub fn eq_presets(d: &[u8]) -> Option<Vec<EqPreset>> {
    let n = *d.first()? as usize;
    let mut i = 1;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        let h = d.get(i..i + 5)?;
        i += 5;
        let name = String::from_utf8_lossy(d.get(i..i + h[4] as usize)?).into_owned();
        i += h[4] as usize;
        let np = *d.get(i)? as usize;
        i += 1;
        let mut points = Vec::with_capacity(np);
        for _ in 0..np {
            let p = d.get(i..i + 3)?;
            points.push((u16::from_le_bytes([p[0], p[1]]), p[2] as i8));
            i += 3;
        }
        out.push(EqPreset { selected: h[0] != 0, min: h[1] as i8, max: h[2] as i8, id: h[3], name, points });
    }
    Some(out)
}

/// 0x8129: `n, {featureId u8, value u32 LE}*n`.
pub fn feature_states(d: &[u8]) -> Vec<(u8, u32)> {
    records(d, 5).map(|c| (c[0], u32::from_le_bytes([c[1], c[2], c[3], c[4]]))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battery_bits() {
        let b = battery(&[3, 1, 0x80 | 55, 2, 60, 3, 0x80 | 100]);
        assert_eq!(b[0], Battery { device: 1, level: 55, charging: true });
        assert_eq!(b[1], Battery { device: 2, level: 60, charging: false });
        assert_eq!(b[2].level, 100);
    }

    #[test]
    fn wear_flags_from_real_t310() {
        // captured: right bud in the case, lid open, then both docked with lid closed
        let w = wear(&[3, 3, 0x04, 1, 0x00, 2, 0x00]);
        assert!(w[0].device == 3 && w[0].in_box && !w[0].cover_closed);
        let w = wear(&[2, 3, 0x00, 1, 0x00]);
        assert!(w[0].cover_closed);
        let w = wear(&[2, 1, 0x03, 2, 0x00]);
        assert!(w[0].in_ear && !w[0].in_box && w[1].in_box);
    }

    #[test]
    fn truncated_is_empty_not_panic() {
        assert!(battery(&[5, 1]).is_empty());
        assert!(eq_presets(&[1, 0, 0]).is_none());
    }

    #[test]
    fn eq_parse() {
        let mut d = vec![1, 1, 0xF6, 10, 7, 3];
        d.extend_from_slice(b"Pop");
        d.push(2);
        d.extend_from_slice(&[100, 0, 3, 0x2C, 1, 0xFE]);
        let e = eq_presets(&d).unwrap();
        assert_eq!(e[0].name, "Pop");
        assert_eq!(e[0].points, vec![(100, 3), (300, -2)]);
        assert_eq!(e[0].min, -10);
    }
}
