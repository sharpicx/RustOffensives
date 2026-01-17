// Failed to open file
use std::cell::Cell;

#[inline(always)]
fn rotl(v: u8, r: u8) -> u8 {
    (v << r) | (v >> (8 - r))
}

#[inline(always)]
fn rotr(v: u8, r: u8) -> u8 {
    (v >> r) | (v << (8 - r))
}

#[inline(never)]
fn stage_a(x: u8, k: u8) -> u8 {
    rotl(x ^ k, 3).wrapping_add(19)
}

#[inline(never)]
fn stage_b(x: u8, k: u8) -> u8 {
    rotr(x.wrapping_sub(7) ^ k, 2)
}

#[inline(never)]
fn opaque(x: u8) -> u8 {
    let y = Cell::new(x);
    if (y.get().wrapping_mul(y.get())) & 1 == 0 {
        y.get() ^ 0x5A
    } else {
        y.get() ^ 0x5A
    }
}

pub fn build_string6() -> String {
    let blob: [u8; 19] = [90, 142, 205, 8, 15, 69, 38, 231, 64, 245, 168, 82, 87, 180, 101, 175, 141, 158, 180];
    let key1: [u8; 19] = [112, 204, 119, 108, 138, 101, 37, 132, 168, 163, 164, 32, 149, 219, 252, 14, 102, 197, 49];
    let key2: [u8; 19] = [35, 160, 10, 234, 61, 62, 205, 58, 237, 7, 34, 227, 16, 125, 30, 4, 167, 62, 82];

    let mut out = [0u8; 19];
    let mut state: u8 = 173;

    for i in 0..19 {
        let mut v = blob[i];

        match state & 3 {
            0 => v = stage_a(v, key1[i]),
            1 => v = stage_b(v, key2[i]),
            2 => v = stage_b(stage_a(v, key1[i]), key2[i]),
            _ => v = stage_a(stage_b(v, key2[i]), key1[i]),
        }

        v = opaque(v);
        out[i] = v;
        state = rotl(state ^ v, 1);
    }

    String::from_utf8(out.to_vec()).unwrap()
}
