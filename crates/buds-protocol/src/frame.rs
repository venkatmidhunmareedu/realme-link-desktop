//! Outer framing (`OPPOv1Wrapper`):
//! `0xAA | len varint | ctrl | 0x00 | [fsn] | payload`, len counts everything after itself.
//! ctrl & 3: 0 single, 1 first, 2 middle, 3 last. Multi-frame frames carry an fsn byte.
pub const SOF: u8 = 0xAA;
pub const DEFAULT_MAX_FRAME: usize = 2000;

fn put_varint(mut n: usize, out: &mut Vec<u8>) {
    loop {
        let b = (n & 0x7F) as u8;
        n >>= 7;
        if n == 0 {
            out.push(b);
            return;
        }
        out.push(b | 0x80);
    }
}

/// Returns (value, bytes consumed).
fn get_varint(b: &[u8]) -> Option<(usize, usize)> {
    let (mut v, mut i) = (0usize, 0);
    loop {
        let x = *b.get(i)?;
        v |= ((x & 0x7F) as usize) << (7 * i);
        i += 1;
        if x & 0x80 == 0 {
            return Some((v, i));
        }
        if i > 4 {
            return None;
        }
    }
}

/// Length-field size the app assumes when sizing frames (`OPPOv1Wrapper.j`).
fn len_size(idx: usize, remaining: usize, max: usize) -> usize {
    if max <= 129 {
        1
    } else if idx == 0 {
        if remaining <= 125 { 1 } else { 2 }
    } else if remaining <= 124 {
        1
    } else {
        2
    }
}

pub fn encode(payload: &[u8], max_frame: usize) -> Vec<Vec<u8>> {
    let mut frames = Vec::new();
    let (mut left, mut off, mut idx) = (payload.len(), 0usize, 0usize);
    while left > 0 {
        let room = max_frame - len_size(idx, left, max_frame);
        let (take, ty, fsn) = if idx == 0 {
            if left <= room - 3 { (left, 0u8, false) } else { (room - 4, 1, true) }
        } else if left <= room - 4 {
            (left, 3, true)
        } else {
            (room - 4, 2, true)
        };
        let mut f = vec![SOF];
        put_varint(take + if fsn { 3 } else { 2 }, &mut f);
        f.push(ty);
        f.push(0);
        if fsn {
            f.push(idx as u8);
        }
        f.extend_from_slice(&payload[off..off + take]);
        frames.push(f);
        off += take;
        left -= take;
        idx += 1;
    }
    frames
}

/// Stream reassembler: feed raw RFCOMM bytes, get complete payloads (Packet bytes).
#[derive(Default)]
pub struct Deframer {
    buf: Vec<u8>,
    msg: Vec<u8>,
}

impl Deframer {
    pub fn push(&mut self, data: &[u8]) -> Vec<Vec<u8>> {
        self.buf.extend_from_slice(data);
        let mut out = Vec::new();
        loop {
            match self.buf.iter().position(|&b| b == SOF) {
                Some(0) => {}
                Some(n) => {
                    self.buf.drain(..n);
                }
                None => {
                    self.buf.clear();
                    break;
                }
            }
            let Some((len, ls)) = get_varint(&self.buf[1..]) else { break };
            let total = 1 + ls + len;
            if self.buf.len() < total {
                break;
            }
            let frame: Vec<u8> = self.buf.drain(..total).collect();
            let ctrl = frame[1 + ls] & 3;
            let start = if ctrl == 0 { 1 + ls + 2 } else { 1 + ls + 3 };
            if start > frame.len() {
                continue;
            }
            match ctrl {
                0 => out.push(frame[start..].to_vec()),
                1 => self.msg = frame[start..].to_vec(),
                2 => self.msg.extend_from_slice(&frame[start..]),
                _ => {
                    self.msg.extend_from_slice(&frame[start..]);
                    out.push(std::mem::take(&mut self.msg));
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_frame_layout() {
        assert_eq!(encode(&[1, 2, 3], DEFAULT_MAX_FRAME), vec![vec![0xAA, 5, 0, 0, 1, 2, 3]]);
    }

    #[test]
    fn roundtrip_multi_and_byte_by_byte() {
        let payload: Vec<u8> = (0..5000u32).map(|i| (i % 251) as u8).collect();
        let frames = encode(&payload, 300);
        assert!(frames.len() > 1);
        assert!(frames.iter().all(|f| f.len() <= 300));
        let mut d = Deframer::default();
        let mut got = Vec::new();
        for b in frames.concat() {
            got.extend(d.push(&[b]));
        }
        assert_eq!(got, vec![payload]);
    }

    #[test]
    fn two_messages_one_read_and_garbage_prefix() {
        let mut wire = vec![0x00, 0x13];
        wire.extend(encode(&[9], 2000).concat());
        wire.extend(encode(&[8, 8], 2000).concat());
        assert_eq!(Deframer::default().push(&wire), vec![vec![9], vec![8, 8]]);
    }
}
