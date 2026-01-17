use generic_array::GenericArray;
use rc4::{KeyInit, Rc4, StreamCipher};
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

    let mut key_string = String::from("a");
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

    let key_bytes = key_string.as_bytes();

    for input_filename in files {
        let mut buf = match read_file(input_filename) {
            Ok(content) => content,
            Err(_) => {
                println!("[!] Failed to read: {}", input_filename);
                continue;
            }
        };

        let mut rc4 = Rc4::new(GenericArray::<u8, U25>::from_slice(key_bytes));
        rc4.apply_keystream(&mut buf);

        let filename_without_extension = match input_filename.rfind('.') {
            Some(index) => &input_filename[..index],
            None => input_filename,
        };

        let output_filename = format!("{}.bin", filename_without_extension);

        match write_file(&output_filename, &buf) {
            Ok(_) => println!(
                "[+] Success: {} -> {} (Key: '{}')",
                input_filename, output_filename, key_string
            ),
            Err(_) => println!("[!] Failed to write: {}", output_filename),
        }
    }
}

fn print_usage(args: &[String]) {
    let path = &args[0];
    println!("[!] Usage: {} [--key <secret>] <file1> <file2> ...", path);
    println!(
        "    Example: {} --key mysecretpass file1.txt file2.exe",
        path
    );
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
