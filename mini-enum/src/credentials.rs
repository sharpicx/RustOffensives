use crate::helper::{log_info, log_success};
use std::fs::{self, File, metadata};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Debug)]
struct CredentialFileInfo {
    name: String,
    description: String,
    master_key: Uuid,
    created: SystemTime,
    accessed: SystemTime,
    size: u64,
}

#[derive(Debug)]
struct WindowsCredentialFileDTO {
    path: PathBuf,
    credentials: Vec<CredentialFileInfo>,
}

pub fn enum_creds() {
    let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
    let system_drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string());
    let user_folder = format!("{}\\Users\\", system_drive);

    let mut credential_folders = vec![
        format!(
            "{}\\System32\\config\\systemprofile\\AppData\\Local\\Microsoft\\Credentials",
            system_root
        ),
        format!(
            "{}\\System32\\config\\systemprofile\\AppData\\Roaming\\Microsoft\\Credentials",
            system_root
        ),
        format!(
            "{}\\ServiceProfiles\\LocalService\\AppData\\Local\\Microsoft\\Credentials",
            system_root
        ),
        format!(
            "{}\\ServiceProfiles\\LocalService\\AppData\\Roaming\\Microsoft\\Credentials",
            system_root
        ),
        format!(
            "{}\\ServiceProfiles\\NetworkService\\AppData\\Local\\Microsoft\\Credentials",
            system_root
        ),
        format!(
            "{}\\ServiceProfiles\\NetworkService\\AppData\\Roaming\\Microsoft\\Credentials",
            system_root
        ),
    ];

    if let Ok(entries) = fs::read_dir(&user_folder) {
        for entry in entries.flatten() {
            let dir = entry.path();
            if let Some(name) = dir.file_name().and_then(|n| n.to_str()) {
                if ["Public", "Default", "Default User", "All Users"].contains(&name) {
                    continue;
                }
                credential_folders.push(
                    dir.join("AppData\\Local\\Microsoft\\Credentials")
                        .to_string_lossy()
                        .to_string(),
                );
                credential_folders.push(
                    dir.join("AppData\\Roaming\\Microsoft\\Credentials")
                        .to_string_lossy()
                        .to_string(),
                );
            }
        }
    }

    for cred_path in credential_folders {
        for dto in get_credentials_from_directory(&cred_path) {
            log_success(&format!("Folder: {:?}", dto.path));
            for cred in dto.credentials {
                log_success(&format!(
                    "File: {}\n    Desc: {}\n    Masterkey: {}\n    Size: {}\n    Created: {:?}\n    Accessed: {:?}\n",
                    cred.name,
                    cred.description,
                    cred.master_key,
                    cred.size,
                    cred.created,
                    cred.accessed
                ));
            }
        }
    }
    log_info(&format!(
        "\n    - https://0xdf.gitlab.io/2025/11/01/htb-voleur.html#recover-credential\n"
    ));
}

fn get_credentials_from_directory(path: &str) -> Vec<WindowsCredentialFileDTO> {
    let mut result = vec![];
    let dir = Path::new(path);
    if !dir.exists() {
        return result;
    }

    let mut user_credentials = vec![];
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let file_path = entry.path();
            if let Some(cred) = credential_file(&file_path) {
                user_credentials.push(cred);
            }
        }
    }

    if !user_credentials.is_empty() {
        result.push(WindowsCredentialFileDTO {
            path: dir.to_path_buf(),
            credentials: user_credentials,
        });
    }

    result
}

fn credential_file(file_path: &Path) -> Option<CredentialFileInfo> {
    let metadata = metadata(file_path).ok()?;
    let size = metadata.len();
    let mut file = File::open(file_path).ok()?;
    let mut bytes = vec![];
    file.read_to_end(&mut bytes).ok()?;

    if bytes.len() < 60 {
        return None;
    }

    let guid_master_key = Uuid::from_slice(&bytes[36..52]).ok()?;

    let desc_len = u32::from_le_bytes(bytes[56..60].try_into().ok()?) as usize;
    if bytes.len() < 60 + desc_len - 4 {
        return None;
    }

    let desc_bytes = &bytes[60..60 + desc_len - 4];
    let description = String::from_utf16_lossy(
        &desc_bytes
            .chunks(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect::<Vec<u16>>(),
    );

    Some(CredentialFileInfo {
        name: file_path.file_name()?.to_string_lossy().to_string(),
        description,
        master_key: guid_master_key,
        created: metadata.created().unwrap_or(SystemTime::UNIX_EPOCH),
        accessed: metadata.accessed().unwrap_or(SystemTime::UNIX_EPOCH),
        size,
    })
}
