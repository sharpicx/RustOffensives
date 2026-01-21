use generic_array::GenericArray;
use hc_256::Hc256;
use hc_256::cipher::{KeyIvInit, StreamCipher as HcStreamCipher};
use rc4::{KeyInit, Rc4};
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use typenum::U25;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage(&args);
        return;
    }

    let mut key_string = String::from("default_secret_key_32_chars_long");
    let mut files = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--key" | "-k" => {
                if i + 1 < args.len() {
                    key_string = args[i + 1].clone();
                    i += 2;
                } else {
                    println!("[!] Error: --key requires a value");
                    return;
                }
            }
            _ => {
                files.push(&args[i]);
                i += 1;
            }
        }
    }

    if files.is_empty() {
        println!("[!] No files provided");
        return;
    }

    for input_filename in files {
        let mut buf = match read_file(input_filename) {
            Ok(content) => content,
            Err(_) => {
                println!("[!] Failed to read: {}", input_filename);
                continue;
            }
        };

        let hc_key = prepare_32_byte_array(&key_string);
        let hc_nonce = prepare_32_byte_array(&format!("nonce-{}", key_string));
        let mut hc_cipher = Hc256::new(&hc_key.into(), &hc_nonce.into());
        hc_cipher.apply_keystream(&mut buf);

        let rc4_key = key_string.as_bytes();
        let mut rc4 = Rc4::new(GenericArray::<u8, U25>::from_slice(
            &rc4_key[..std::cmp::min(rc4_key.len(), 256)],
        ));
        rc4.apply_keystream(&mut buf);

        let xor_key = rc4_key;
        let xor_len = xor_key.len();
        if xor_len > 0 {
            for (idx, byte) in buf.iter_mut().enumerate() {
                *byte ^= xor_key[idx % xor_len] ^ 0xAA;
            }
        }

        let filename_without_extension = match input_filename.rfind('.') {
            Some(index) => &input_filename[..index],
            None => input_filename,
        };

        let output_filename = format!("{}.bin", filename_without_extension);

        match write_file(&output_filename, &buf) {
            Ok(_) => println!("[+] Success: {} -> {}", input_filename, output_filename),
            Err(_) => println!("[!] Failed to write: {}", output_filename),
        }
    }
}

fn prepare_32_byte_array(input: &str) -> [u8; 32] {
    let mut key = [0u8; 32];
    let bytes = input.as_bytes();
    for i in 0..32 {
        if i < bytes.len() {
            key[i] = bytes[i];
        } else {
            key[i] = (i as u8).wrapping_add(0x55);
        }
    }
    key
}

fn print_usage(args: &[String]) {
    let path = &args[0];
    println!("[!] Usage: {} [--key <secret>] <file1> <file2> ...", path);
}

fn read_file(filename: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(filename)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}

fn write_file(filename: &str, content: &[u8]) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(content)?;
    Ok(())
}
