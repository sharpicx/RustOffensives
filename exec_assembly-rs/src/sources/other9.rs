// Ropcaster1337motherfucker
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
    let blob: [u8; 25] = [
        77, 221, 98, 133, 213, 35, 188, 21, 207, 27, 209, 153, 12, 247, 226, 166, 29, 46, 65, 181,
        38, 135, 67, 218, 246,
    ];
    let key1: [u8; 25] = [
        243, 153, 64, 132, 103, 225, 37, 144, 55, 200, 119, 197, 71, 111, 79, 17, 143, 171, 127,
        144, 165, 51, 231, 31, 81,
    ];
    let key2: [u8; 25] = [
        19, 213, 181, 154, 34, 46, 96, 22, 158, 7, 228, 174, 130, 12, 173, 86, 104, 195, 93, 213,
        7, 85, 245, 198, 32,
    ];

    let mut out = [0u8; 25];
    let mut state: u8 = 64;

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
