use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

pub fn is_dotnet_assembly(path: &str) -> bool {
    let mut f = match File::open(path) {
        Ok(x) => x,
        Err(_) => return false,
    };

    let len = match f.metadata() {
        Ok(m) => m.len(),
        Err(_) => return false,
    };
    if len < 64 {
        return false;
    }

    if f.seek(SeekFrom::Start(0x3C)).is_err() {
        return false;
    }
    let pe_header_pointer = match read_u32(&mut f) {
        Some(0) => 0x80,
        Some(v) => v as u64,
        None => return false,
    };

    if pe_header_pointer > len.saturating_sub(256) {
        return false;
    }

    if f.seek(SeekFrom::Start(pe_header_pointer)).is_err() {
        return false;
    }
    let sig = match read_u32(&mut f) {
        Some(v) => v,
        None => return false,
    };
    if sig != 0x00004550 {
        return false;
    }

    if f.seek(SeekFrom::Current(20)).is_err() {
        return false;
    }

    let pe_format = match read_u16(&mut f) {
        Some(v) => v,
        None => return false,
    };
    const PE32: u16 = 0x10b;
    const PE32_PLUS: u16 = 0x20b;

    if pe_format != PE32 && pe_format != PE32_PLUS {
        return false;
    }

    let dd_start = pe_header_pointer + if pe_format == PE32 { 232 } else { 248 };

    if f.seek(SeekFrom::Start(dd_start)).is_err() {
        return false;
    }

    let cli_header_rva = match read_u32(&mut f) {
        Some(v) => v,
        None => return false,
    };

    cli_header_rva != 0
}

fn read_u16<R: Read>(r: &mut R) -> Option<u16> {
    let mut b = [0u8; 2];
    if r.read_exact(&mut b).is_ok() {
        Some(u16::from_le_bytes(b))
    } else {
        None
    }
}

fn read_u32<R: Read>(r: &mut R) -> Option<u32> {
    let mut b = [0u8; 4];
    if r.read_exact(&mut b).is_ok() {
        Some(u32::from_le_bytes(b))
    } else {
        None
    }
}
