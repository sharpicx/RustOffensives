use crate::enum_date;
use chrono::{DateTime, Datelike, Local};
use colored::*;
use std::os::windows::ffi::OsStrExt;
use std::time::SystemTime;
use std::{
    ffi::OsString,
    fs::File,
    io::{BufRead, BufReader},
    os::windows::ffi::OsStringExt,
    path::PathBuf,
};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Storage::FileSystem::*;

const DEFAULT_KEYWORDS: &[&str] = &[
    "password",
    "passwords",
    "passwd",
    "passw",
    "pass",
    "pwd",
    "secret",
    "secrets",
    "api",
    "api_key",
    "apikey",
    "api_secret",
    "access_key",
    "secret_key",
    "private_key",
    "public_key",
    "auth_token",
    "token",
    "jwt",
    "jwtsecret",
    "redis",
    "bearer_token",
    "refresh_token",
    "session_token",
    "session_id",
    "sessionid",
    "auth",
    "authorization",
    "credentials",
    "creds",
    "login",
    "username",
    "user",
    "userid",
    "user_id",
    "hash",
    "hashed_password",
    "salt",
    "bcrypt_hash",
    "sha256",
    "md5_hash",
    "otp",
    "one_time_password",
    "passcode",
    "pin",
    "mfa",
    "two_factor",
    "2fa",
    "totp_secret",
    "shared_secret",
    "recovery_code",
    "backup_code",
    "client_id",
    "client_secret",
    "authentication",
    "authorization_code",
    "authorization_token",
    "authz",
    "authn",
    "identity",
    "identity_token",
    "id_token",
    "access_token",
    "assertion",
    "saml",
    "saml_token",
    "saml_assertion",
    "openid_connect",
    "oidc",
    "oauth",
    "oauth_token",
    "oauth2",
    "oauth2_token",
    "request_token",
    "verifier",
    "consumer_key",
    "consumer_secret",
    "signing_key",
    "verification_key",
    "x509_certificate",
    "cert",
    "certificate",
    "ssl_certificate",
    "server",
    "tls_certificate",
    "keystore",
    "truststore",
    "jks",
    "p12",
    "pem",
    "key",
    "keyfile",
    "key_store",
    "key_store_password",
    "master_key",
    "encryption_key",
    "decryption_key",
    "hmac_key",
    "webhook_secret",
    "webhook_token",
    "slack_token",
    "github_token",
    "gitlab_token",
    "bitbucket_token",
    "jira_token",
    "jenkins_token",
    "circleci_token",
    "travis_token",
    "docker_hub_token",
    "registry_token",
    "npm_token",
    "pypi_token",
    "rubygems_token",
    "vault_token",
    "consul_token",
    "kubernetes_token",
    "k8s_token",
    "service_account_token",
    "gcp_service_account",
    "aws_access_key_id",
    "aws_secret_access_key",
    "aws_session_token",
    "azure",
    "azure_client_id",
    "azure_client_secret",
    "azure_tenant_id",
    "azure_subscription_id",
    "sfdc_client_secret",
    "salesforce_consumer_secret",
    "okta_client_secret",
    "auth0_client_secret",
    "firebase_private_key",
    "google_private_key",
    "ssh_key",
    "ssh_private_key",
    "ssh_public_key",
    "id_rsa",
    "id_dsa",
    "id_ecdsa",
    "id_ed25519",
    "authorized_keys",
    "hostname",
    "domain",
    "host",
    "known_hosts",
    "passphrase",
    "key_passphrase",
    "db_password",
    "db",
    "DB",
    "server",
    "database",
    "database_password",
    "db_user",
    "db_pass",
    "root_password",
    "admin_password",
    "sa_password",
    "sys_password",
    "oracle_password",
    "postgres_password",
    "mysql_password",
    "redis_password",
    "mongodb_password",
    "rabbitmq_password",
    "activemq_password",
    "ldap_password",
    "bind_password",
    "proxy_password",
    "mail_password",
    "smtp_password",
    "imap_password",
    "pop3_password",
    "api_username",
    "api_password",
    "basic_auth",
    "digest_auth",
    "ntlm",
    "active",
    "directory",
    "AD",
    "kerberos",
    "krb5_keytab",
    "keytab",
    "ticket_granting_ticket",
    "tgs",
    "service_ticket",
    "st",
    "csrf_token",
    "xsrf_token",
    "anti_forgery_token",
    "state_token",
    "nonce",
    "captcha_secret",
    "recaptcha_secret",
    "hcaptcha_secret",
    "turnstile_secret",
    "device_secret",
    "device_token",
    "push_token",
    "fcm_token",
    "apns_token",
    "biometric_secret",
    "fingerprint_template",
    "face_id_template",
    "voice_print",
    "retina_scan",
    "signature_pad_data",
    "digital_signature",
    "certificate_revocation_list",
    "crl",
    "certificate_authority",
    "ca",
    "intermediate_certificate",
    "root_certificate",
    "signing_certificate",
    "encryption_certificate",
    "ssl_key",
    "tls_key",
    "rsa_private_key",
    "ec_private_key",
    "dsa_private_key",
    "pem_key",
    "der_key",
    "pkcs12",
    "pfx",
    "keystore_password",
    "truststore_password",
    "alias_password",
    "key_password",
    "store_password",
    "<PSCredential>",
    "<UserName>",
    "</UserName>",
    "</Password>",
    "<Password>",
    "</PSCredential>",
    "PSCredential",
    "System.Management.Automation.",
    "New-Object",
    "New-PSSession",
    "-ComputerName",
    "-Credential",
    "ConvertTo-SecureString",
    "Export-Clixml",
    "Import-Clixml",
    "Credential",
    "credential",
    "SecureString",
    "PSSessionOption",
    "Invoke-Command",
    "System.Security.SecureString",
    "SecureString",
    "Security",
    "System",
    "NetworkCredential",
    "System.Net",
    "ProtectedData",
    "EncryptedStandardString",
    "ClientSecret",
    "New-PSDrive",
    "ConnectionString",
    "DbPassword",
    "SqlPassword",
    "VaultPassword",
    "Get-VaultCredential",
    "Vault",
    "VaultCredential",
    "SecurePassword",
    "SecurePasswd",
    "SecurePwd",
    "SecurePass",
];

fn match_file_date(t: SystemTime, pattern: &str) -> bool {
    let dt: DateTime<Local> = t.into();
    let year = dt.year();
    let month = dt.month();
    let day = dt.day();

    let parts: Vec<_> = pattern.split('-').collect();

    match parts.len() {
        1 => {
            if let Ok(y) = parts[0].parse::<i32>() {
                if y > 31 {
                    return y == year;
                } // anggap aja dulu: >31 = tahun
                return y == day as i32; // anggap ke i32: <=31 = tanggal
            }
        }
        2 => {
            if let (Ok(p1), Ok(p2)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                if p2 > 31 {
                    // format mm-yyyy
                    return p1 == month && p2 as i32 == year;
                } else {
                    // format dd-mm
                    return p1 == day && p2 == month;
                }
            }
        }
        3 => {
            // dd-mm-yyyy
            if let (Ok(d), Ok(m), Ok(y)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<i32>(),
            ) {
                return d == day && m == month && y == year;
            }
        }
        _ => {}
    }

    false
}

fn build_keywords(user_keywords: Vec<String>, default_keywords: &[&str]) -> Vec<String> {
    let mut final_kw = Vec::new();

    for &d in default_keywords {
        final_kw.push(d.to_string());
    }

    for uk in user_keywords {
        let low = uk.to_lowercase();

        if default_keywords
            .iter()
            .any(|d| d.eq_ignore_ascii_case(&low))
        {
            println!(
                "[!] keyword: '{}' already declared in the DEFAULT_KEYWORDS.",
                uk
            );
        }

        final_kw.push(uk);
    }

    final_kw
}

pub fn enum_sus_keywords(
    dirs_raw: String,
    add_words_raw: String,
    exclude_exts_raw: String,
    exclude_dirs_raw: String,
    exclude_words_raw: String,
    exclude_date_raw: String,
    match_date: String,
) {
    let dirs = dirs_raw
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    let exclude_dirs = exclude_dirs_raw
        .split(',')
        .map(|s| s.trim().replace(r".\", ""))
        .map(|s| s.to_lowercase())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    let user_keywords = add_words_raw
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    let exclude_exts = exclude_exts_raw
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    let exclude_exts = exclude_exts
        .into_iter()
        .map(|x| x.trim().trim_start_matches('.').to_lowercase())
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>();

    let exclude_words = exclude_words_raw
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    let exclude_dates: Vec<enum_date::DateFilter> = exclude_date_raw
        .split(',')
        .filter_map(|s| enum_date::parse_filter(s.trim()))
        .collect();

    let mut keywords = build_keywords(user_keywords, DEFAULT_KEYWORDS);
    keywords.retain(|k| !exclude_words.iter().any(|ex| ex.eq_ignore_ascii_case(k)));

    let kws = keywords
        .into_iter()
        .map(|k| k.to_lowercase())
        .collect::<Vec<_>>();

    let mut stack = Vec::<PathBuf>::new();
    for d in dirs {
        stack.push(PathBuf::from(d));
    }

    while let Some(root) = stack.pop() {
        let root_lc = root.to_string_lossy().to_lowercase();
        if exclude_dirs.iter().any(|d| root_lc.contains(d)) {
            continue;
        }
        let mut pat = root.clone();
        pat.push("*");

        let pat_w: Vec<u16> = pat.as_os_str().encode_wide().chain([0]).collect();
        let pat_ptr = pat_w.as_ptr();

        let mut data = WIN32_FIND_DATAW::default();
        let h = unsafe { FindFirstFileW(pat_ptr, &mut data) };
        if h == INVALID_HANDLE_VALUE {
            continue;
        }

        let mut first = true;
        loop {
            if !first && unsafe { FindNextFileW(h, &mut data) } == 0 {
                break;
            }
            first = false;

            let wide = &data.cFileName;
            let len = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
            let name = OsString::from_wide(&wide[..len]);

            if name == "." || name == ".." {
                continue;
            }

            let mut full = root.clone();
            full.push(&name);

            let attrs = data.dwFileAttributes;
            let is_dir = (attrs & FILE_ATTRIBUTE_DIRECTORY) != 0;

            if is_dir {
                let full_lc = full.to_string_lossy().to_lowercase();
                if exclude_dirs.iter().any(|d| full_lc.contains(d)) {
                    continue;
                }
                stack.push(full);
                continue;
            }

            let ext = full
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| x.to_lowercase())
                .unwrap_or_default();

            if exclude_exts.contains(&ext) {
                continue;
            }

            let file = match File::open(&full) {
                Ok(f) => f,
                Err(_) => continue,
            };

            let metadata = match full.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let dt: DateTime<Local> = metadata
                .created()
                .unwrap_or(metadata.modified().unwrap())
                .into();

            if exclude_dates
                .iter()
                .any(|f| enum_date::matches_filter(&dt, f))
            {
                continue;
            }

            if !match_date.is_empty()
                && !match_file_date(
                    metadata.created().unwrap_or(metadata.modified().unwrap()),
                    &match_date,
                )
            {
                continue;
            }

            let reader = BufReader::new(file);
            for (i, line) in reader.lines().enumerate() {
                let Ok(line) = line else { continue };
                let l = line.to_lowercase();

                for kw in &kws {
                    if let Some(pos) = l.find(kw) {
                        let end = pos + kw.len();

                        let highlighted = format!(
                            "{}{}{}",
                            &line[..pos],
                            &line[pos..end].red().bold(),
                            &line[end..]
                        );

                        let p = full.display().to_string().bright_red();
                        let ln = format!("{}", i + 1).bright_yellow();

                        println!("{p}:{ln}:{highlighted}");
                        break;
                    }
                }
            }
        }

        unsafe { FindClose(h) };
    }
}
