# string_generator

dynamic string obfuscation for Rust built-in source. evading sensitive strings (signature-based).

## Usage

execute the python script, and put the strings to be obfuscated in the arguments.

```console
via@ARCI$ python3 generator.py 'amsi.dll' 'AmsiScanBuffer' 'ntdll.dll' 'NtTraceControl' 'Error during verification: ' 'Failed to open file' 'Failed to read file' '[+] Results:' 'Ropcaster1337motherfucker'
[+] 'amsi.dll' -> sources/other1.rs
[+] 'AmsiScanBuffer' -> sources/other2.rs
[+] 'ntdll.dll' -> sources/other3.rs
[+] 'NtTraceControl' -> sources/other4.rs
[+] 'Error during verification: ' -> sources/other5.rs
[+] 'Failed to open file' -> sources/other6.rs
[+] 'Failed to read file' -> sources/other7.rs
[+] '[+] Results:' -> sources/other8.rs
[+] 'Ropcaster1337motherfucker' -> sources/other9.rs
[+] Generated sources/mod.rs with references
via@ARCI$ mv sources /path/to/the/rust/source
```

then move the `sources/` to the rust source tree in the `your_project/src/sources`. after that write `mod sources;` in the `main.rs`.

for example.

```rs
mod sources; // put it here

use rc4::cipher::generic_array::GenericArray;
use std::ffi::CString;
use std::mem;
use std::mem::zeroed;
use std::ptr::null_mut;
use winapi::ctypes::c_void;
...
const S_OK: i32 = 0;
const AMSI_RESULT_CLEAN: i32 = 0;
...
```

after that, write `sources::build_stringN();` into the callable string.

for example.

```rs
fn read_file(filename: &str) -> Vec<u8> {
    //                                                  like this
    //                                                      |
    let mut file = File::open(filename).expect(&sources::build_string6());
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .expect(&sources::build_string7());
    contents
}
```

this scenario has been implemented in [Ropcaster/exec-assembly](https://github.com/Ropcaster/exec-assembly).
