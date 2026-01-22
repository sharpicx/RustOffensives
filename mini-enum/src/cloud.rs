use colored::*;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub fn enum_cloud_creds() {
    let users_root = Path::new("C:\\Users");

    if let Ok(users) = fs::read_dir(users_root) {
        for user in users.flatten() {
            let home = user.path();
            let name = home.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if matches!(name, "Public" | "Default" | "Default User" | "All Users") {
                continue;
            }

            scan_paths(&home, aws_paths(&home), "AWS");
            scan_paths(&home, gcp_paths(&home), "GCP");
            scan_paths(&home, azure_paths(&home), "Azure");
            scan_paths(&home, bluemix_paths(&home), "Bluemix");
            scan_paths(&home, terraform_paths(&home), "Terraform");
            scan_paths(&home, docker_paths(&home), "Docker");
            scan_paths(&home, kube_paths(&home), "Kubernetes");
            scan_paths(&home, devtool_paths(&home), "DevTooling");
        }
    }

    print_env();
    println!();
}

fn scan_paths(_home: &Path, paths: Vec<PathBuf>, provider: &str) {
    for p in paths {
        if p.is_file() {
            print_file(provider, &p);
        } else if p.is_dir() {
            print_dir(provider, &p);
            if let Ok(rd) = fs::read_dir(&p) {
                for e in rd.flatten() {
                    let ep = e.path();
                    if ep.is_file() {
                        print_file(provider, &ep);
                    }
                }
            }
        }
    }
}

fn print_file(provider: &str, path: &Path) {
    if let Ok(m) = fs::metadata(path) {
        let a = m.accessed().unwrap_or(SystemTime::UNIX_EPOCH);
        let w = m.modified().unwrap_or(SystemTime::UNIX_EPOCH);

        println!(
            "{} | {} | {} | accessed={:?} modified={:?} size={}",
            provider.bold(),
            "FILE".blue(),
            path.display().to_string().dimmed(),
            a,
            w,
            m.len()
        );
    }
}

fn print_dir(provider: &str, path: &Path) {
    println!(
        "{} | {} | {}",
        provider.bold(),
        "DIR".yellow(),
        path.display().to_string().dimmed()
    );
}

// fn print_file(provider: &str, path: &Path) {
//     if let Ok(m) = fs::metadata(path) {
//         let a = m.accessed().unwrap_or(SystemTime::UNIX_EPOCH);
//         let w = m.modified().unwrap_or(SystemTime::UNIX_EPOCH);
//         println!(
//             "{} | FILE | {:?} | accessed={:?} modified={:?} size={}",
//             provider,
//             path,
//             a,
//             w,
//             m.len()
//         );
//     }
// }

// fn print_dir(provider: &str, path: &Path) {
//     println!("{} | DIR  | {:?}", provider, path);
// }

fn aws_paths(home: &Path) -> Vec<PathBuf> {
    vec![
        home.join(".aws\\credentials"),
        home.join(".aws\\config"),
        home.join(".aws\\cli\\cache"),
        local_app().join("AWSToolkit\\CachedCredentials"),
        local_app().join("AWSToolkit\\RegisteredAccounts.json"),
    ]
}

fn gcp_paths(home: &Path) -> Vec<PathBuf> {
    vec![
        roaming().join("gcloud\\credentials.db"),
        roaming().join("gcloud\\access_tokens.db"),
        roaming().join("gcloud\\legacy_credentials"),
        roaming().join("gcloud\\application_default_credentials.json"),
        roaming().join("gcloud\\configurations"),
        local_app().join("Google\\Cloud SDK"),
    ]
}

fn azure_paths(home: &Path) -> Vec<PathBuf> {
    vec![
        home.join(".azure\\azureProfile.json"),
        home.join(".azure\\TokenCache.dat"),
        home.join(".azure\\AzureRmContext.json"),
        home.join(".azure\\msal_token_cache.json"),
        home.join(".azure\\msal_http_cache.bin"),
        roaming().join("Windows Azure Powershell\\TokenCache.dat"),
        roaming().join("Windows Azure Powershell\\AzureRmContext.json"),
        home.join(".azuredevops"),
        roaming().join("Microsoft\\Team Foundation"),
        local_app().join("Microsoft\\IdentityCache"),
    ]
}

fn bluemix_paths(home: &Path) -> Vec<PathBuf> {
    vec![
        home.join(".bluemix\\config.json"),
        home.join(".bluemix\\.cf\\config.json"),
        home.join(".bluemix\\plugins"),
        roaming().join("ibmcloud"),
    ]
}

fn terraform_paths(home: &Path) -> Vec<PathBuf> {
    vec![
        roaming().join("terraform.d\\credentials.tfrc.json"),
        home.join(".terraform.d\\credentials.tfrc.json"),
        roaming().join("Terraform"),
    ]
}

fn docker_paths(home: &Path) -> Vec<PathBuf> {
    vec![
        home.join(".docker\\config.json"),
        roaming().join("Docker\\settings.json"),
        roaming().join("Docker Desktop"),
        local_app().join("Docker"),
    ]
}

fn kube_paths(home: &Path) -> Vec<PathBuf> {
    vec![
        home.join(".kube\\config"),
        roaming().join("kube\\config"),
        local_app().join("kube\\cache"),
    ]
}

fn devtool_paths(home: &Path) -> Vec<PathBuf> {
    vec![
        home.join(".git-credentials"),
        home.join(".gitconfig"),
        home.join(".npmrc"),
        home.join(".yarnrc"),
        home.join(".pip\\pip.ini"),
        roaming().join("Code\\User\\settings.json"),
        roaming().join("JetBrains"),
    ]
}

fn roaming() -> PathBuf {
    env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(""))
}

fn local_app() -> PathBuf {
    env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(""))
}

fn print_env() {
    let vars = [
        "AWS_ACCESS_KEY_ID",
        "AWS_SECRET_ACCESS_KEY",
        "AWS_SESSION_TOKEN",
        "AWS_SHARED_CREDENTIALS_FILE",
        "AWS_CONFIG_FILE",
        "GOOGLE_APPLICATION_CREDENTIALS",
        "AZURE_CLIENT_ID",
        "AZURE_TENANT_ID",
        "AZURE_CLIENT_SECRET",
        "AZURE_SUBSCRIPTION_ID",
    ];
    for v in vars {
        if let Ok(val) = env::var(v) {
            println!("ENV | {} = {}", v, val);
        }
    }
}
