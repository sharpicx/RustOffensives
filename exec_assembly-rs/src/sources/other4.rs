// NtTraceControl
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

pub fn build_string4() -> String {
    let blob: [u8; 14] = [107, 236, 108, 85, 30, 177, 147, 194, 253, 158, 119, 84, 189, 74];
    let key1: [u8; 14] = [75, 143, 40, 89, 133, 245, 22, 43, 169, 186, 222, 131, 249, 166];
    let key2: [u8; 14] = [35, 133, 93, 204, 251, 78, 252, 20, 34, 56, 200, 106, 197, 72];

    let mut out = [0u8; 14];
    let mut state: u8 = 48;

    for i in 0..14 {
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
