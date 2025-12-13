use std::fs;
use std::path::Path;
use std::time::SystemTime;

fn main() {
    let users_dir = Path::new("C:\\Users");

    if let Ok(entries) = fs::read_dir(users_dir) {
        for entry in entries.flatten() {
            let dir = entry.path();
            if let Some(dir_name) = dir.file_name().and_then(|n| n.to_str()) {
                if ["Public", "Default", "Default User", "All Users"].contains(&dir_name) {
                    continue;
                }

                let aws_files = [dir.join(".aws\\credentials"), dir.join(".aws\\config")];
                for file in &aws_files {
                    if file.exists() {
                        print_file_info("AWS", file);
                    }
                }

                let google_files = [
                    dir.join("AppData\\Roaming\\gcloud\\credentials.db"),
                    dir.join("AppData\\Roaming\\gcloud\\legacy_credentials"),
                    dir.join("AppData\\Roaming\\gcloud\\access_tokens.db"),
                ];
                for file in &google_files {
                    if file.exists() {
                        print_file_info("Google", file);
                    }
                }

                let azure_files = [
                    dir.join(".azure\\azureProfile.json"),
                    dir.join(".azure\\TokenCache.dat"),
                    dir.join(".azure\\AzureRMContext.json"),
                    dir.join("AppData\\Roaming\\Windows Azure Powershell\\TokenCache.dat"),
                    dir.join("AppData\\Roaming\\Windows Azure Powershell\\AzureRMContext.json"),
                ];
                for file in &azure_files {
                    if file.exists() {
                        print_file_info("Azure", file);
                    }
                }

                let bluemix_files = [
                    dir.join(".bluemix\\config.json"),
                    dir.join(".bluemix\\.cf\\config.json"),
                ];
                for file in &bluemix_files {
                    if file.exists() {
                        print_file_info("Bluemix", file);
                    }
                }
            }
        }
    }
}

fn print_file_info(provider: &str, path: &Path) {
    if let Ok(metadata) = fs::metadata(path) {
        let last_accessed = metadata.accessed().unwrap_or(SystemTime::UNIX_EPOCH);
        let last_modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        let size = metadata.len();
        println!(
            "{} | {:?} | accessed: {:?} | modified: {:?} | size: {} bytes",
            provider, path, last_accessed, last_modified, size
        );
    }
}
