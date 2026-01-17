import os
import sys
import random
from pwn import log

if len(sys.argv) < 2:
    sys.exit(1)

strings = sys.argv[1:]
os.makedirs("sources", exist_ok=True)

RUST_TEMPLATE = r"""// {original_string}
use std::cell::Cell;

#[inline(always)]
fn rotl(v: u8, r: u8) -> u8 {{
    (v << r) | (v >> (8 - r))
}}

#[inline(always)]
fn rotr(v: u8, r: u8) -> u8 {{
    (v >> r) | (v << (8 - r))
}}

#[inline(never)]
fn stage_a(x: u8, k: u8) -> u8 {{
    rotl(x ^ k, 3).wrapping_add(19)
}}

#[inline(never)]
fn stage_b(x: u8, k: u8) -> u8 {{
    rotr(x.wrapping_sub(7) ^ k, 2)
}}

#[inline(never)]
fn opaque(x: u8) -> u8 {{
    let y = Cell::new(x);
    if (y.get().wrapping_mul(y.get())) & 1 == 0 {{
        y.get() ^ 0x5A
    }} else {{
        y.get() ^ 0x5A
    }}
}}

pub fn build_string{idx}() -> String {{
    let blob: [u8; {n}] = {blob};
    let key1: [u8; {n}] = {key1};
    let key2: [u8; {n}] = {key2};

    let mut out = [0u8; {n}];
    let mut state: u8 = {state};

    for i in 0..{n} {{
        let mut v = blob[i];

        match state & 3 {{
            0 => v = stage_a(v, key1[i]),
            1 => v = stage_b(v, key2[i]),
            2 => v = stage_b(stage_a(v, key1[i]), key2[i]),
            _ => v = stage_a(stage_b(v, key2[i]), key1[i]),
        }}

        v = opaque(v);
        out[i] = v;
        state = rotl(state ^ v, 1);
    }}

    String::from_utf8(out.to_vec()).unwrap()
}}
"""

def rotl(v, r):
    return ((v << r) & 0xFF) | (v >> (8 - r))

def rotr(v, r):
    return (v >> r) | ((v << (8 - r)) & 0xFF)

def solve_stage_a(target, k):
    v = (target - 19) & 0xFF
    v = rotr(v, 3)
    return v ^ k

def solve_stage_b(target, k):
    v = rotl(target, 2)
    v = v ^ k
    return (v + 7) & 0xFF

metadata = []
for idx, s in enumerate(strings, start=1):
    data = s.encode()
    n = len(data)
    key1 = [random.randint(0, 255) for _ in range(n)]
    key2 = [random.randint(0, 255) for _ in range(n)]
    initial_state = random.randint(0, 255)

    blob = []
    current_state = initial_state

    for i in range(n):
        v = data[i] ^ 0x5A
        mode = current_state & 3
        if mode == 0: v = solve_stage_a(v, key1[i])
        elif mode == 1: v = solve_stage_b(v, key2[i])
        elif mode == 2: v = solve_stage_a(solve_stage_b(v, key2[i]), key1[i])
        else: v = solve_stage_b(solve_stage_a(v, key1[i]), key2[i])
        blob.append(v)
        current_state = rotl(current_state ^ data[i], 1)

    rust_code = RUST_TEMPLATE.format(
        original_string=s, idx=idx, n=n, blob=blob,
        key1=key1, key2=key2, state=initial_state
    )

    path = f"sources/other{idx}.rs"
    with open(path, "w", encoding="utf-8") as f:
        f.write(rust_code)
    
    metadata.append((idx, s))
    log.success(f"'{s}' -> {path}")

with open("sources/mod.rs", "w", encoding="utf-8") as f:
    for idx, original in metadata:
        f.write(f"pub mod other{idx};\n")
    
    f.write("\n")
    for idx, original in metadata:
        f.write(f"pub use crate::sources::other{idx}::build_string{idx}; // {original}\n")

log.success("Generated sources/mod.rs with references")
