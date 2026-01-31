// sh4rp1cx1337h4x0r0x133777
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

pub fn build_string9() -> Vec<u8> {
    let blob: [u8; 25] = [94, 6, 172, 124, 232, 1, 253, 87, 109, 63, 15, 35, 51, 59, 251, 228, 30, 1, 226, 9, 254, 254, 21, 219, 145];
    let key1: [u8; 25] = [144, 25, 226, 222, 104, 68, 57, 51, 156, 245, 53, 182, 228, 111, 207, 14, 74, 45, 206, 147, 62, 164, 94, 55, 61];
    let key2: [u8; 25] = [38, 20, 199, 5, 73, 199, 169, 167, 203, 172, 120, 13, 2, 141, 37, 226, 183, 196, 83, 96, 82, 82, 212, 198, 83];

    let mut out = [0u8; 25];
    let mut state: u8 = 218;

    for i in 0..25 {
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

    out.to_vec()
}
