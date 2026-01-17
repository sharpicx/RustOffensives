// Failed to read file
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

pub fn build_string7() -> String {
    let blob: [u8; 19] = [175, 32, 56, 137, 55, 74, 233, 199, 117, 55, 4, 188, 106, 44, 43, 186, 209, 46, 202];
    let key1: [u8; 19] = [0, 37, 76, 237, 73, 148, 142, 22, 49, 51, 137, 59, 111, 107, 171, 159, 213, 167, 96];
    let key2: [u8; 19] = [249, 39, 16, 219, 204, 187, 174, 120, 250, 197, 93, 79, 101, 29, 249, 156, 44, 128, 63];

    let mut out = [0u8; 19];
    let mut state: u8 = 62;

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
