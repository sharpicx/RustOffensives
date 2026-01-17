// Error during verification: 
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

pub fn build_string5() -> String {
    let blob: [u8; 27] = [82, 124, 63, 230, 74, 224, 136, 40, 223, 90, 141, 173, 209, 70, 30, 106, 213, 133, 212, 161, 30, 52, 210, 88, 253, 138, 156];
    let key1: [u8; 27] = [92, 222, 176, 180, 31, 150, 182, 161, 168, 150, 72, 232, 222, 212, 244, 117, 82, 56, 136, 64, 21, 52, 118, 209, 217, 48, 117];
    let key2: [u8; 27] = [60, 66, 152, 28, 227, 48, 206, 228, 240, 190, 234, 23, 2, 16, 235, 195, 132, 9, 1, 126, 251, 149, 2, 133, 229, 2, 243];

    let mut out = [0u8; 27];
    let mut state: u8 = 23;

    for i in 0..27 {
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
