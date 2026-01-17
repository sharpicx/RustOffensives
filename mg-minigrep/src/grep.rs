use crate::get_date;
use chrono::{DateTime, Datelike, Local};
use colored::*;
use memmap2::Mmap;
use std::fs::File;
use std::os::windows::ffi::OsStrExt;
use std::time::SystemTime;
use std::{ffi::OsString, os::windows::ffi::OsStringExt, path::PathBuf};
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

fn is_utf16_le(data: &[u8]) -> bool {
    data.len() >= 2 && data[0] == 0xFF && data[1] == 0xFE
}

fn match_dir_name(path: &PathBuf, exclude: &[String]) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    let name = name.to_lowercase();
    exclude.iter().any(|d| name.contains(d))
}

fn match_file_name(path: &PathBuf, exclude: &[String]) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    let name = name.to_lowercase();
    exclude.iter().any(|f| name.contains(f))
}

fn match_path(path: &PathBuf, exclude: &[String]) -> bool {
    let p = path.to_string_lossy().to_lowercase();
    exclude.iter().any(|x| p.contains(x))
}

fn split_csv(raw: String) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

fn match_file_date(t: SystemTime, pattern: &str) -> bool {
    let dt: DateTime<Local> = t.into();
    let (y, m, d) = (dt.year(), dt.month(), dt.day());
    let parts: Vec<_> = pattern.split('-').collect();

    match parts.len() {
        1 => parts[0]
            .parse::<i32>()
            .map_or(false, |v| if v > 31 { v == y } else { v == d as i32 }),
        2 => {
            if let (Ok(a), Ok(b)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                if b > 31 {
                    a == m && b as i32 == y
                } else {
                    a == d && b == m
                }
            } else {
                false
            }
        }
        3 => {
            if let (Ok(dd), Ok(mm), Ok(yy)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<i32>(),
            ) {
                dd == d && mm == m && yy == y
            } else {
                false
            }
        }
        _ => false,
    }
}

fn build_keywords(user: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = DEFAULT_KEYWORDS.iter().map(|s| s.to_string()).collect();
    out.extend(user);
    out
}

fn scan_binary(path: &PathBuf, keywords: &[String], ignore_case: bool) {
    let Ok(file) = File::open(path) else { return };
    let mmap = unsafe {
        match Mmap::map(&file) {
            Ok(m) => m,
            Err(_) => return,
        }
    };

    for kw in keywords {
        let kw_bytes = kw.as_bytes();
        let kw_len = kw_bytes.len();
        if mmap.len() < kw_len {
            continue;
        }

        for i in 0..=(mmap.len() - kw_len) {
            let window = &mmap[i..i + kw_len];
            let is_match = if ignore_case {
                window
                    .iter()
                    .zip(kw_bytes.iter())
                    .all(|(a, b)| a.to_ascii_lowercase() == b.to_ascii_lowercase())
            } else {
                window == kw_bytes
            };

            if is_match {
                let start = i.saturating_sub(30);
                let end = std::cmp::min(i + kw_len + 30, mmap.len());

                let context: String = mmap[start..end]
                    .iter()
                    .map(|&b| {
                        if b.is_ascii_graphic() || b == b' ' {
                            b as char
                        } else {
                            '.'
                        }
                    })
                    .collect();

                println!(
                    "{}: ...{}...",
                    path.display().to_string().bright_red(),
                    context.cyan()
                );
                break;
            }
        }
    }
}

fn scan_utf16(path: &PathBuf, keywords: &[String], ignore_case: bool) {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return,
    };

    let mmap = unsafe {
        match Mmap::map(&file) {
            Ok(m) => m,
            Err(_) => return,
        }
    };

    for kw in keywords {
        let kw_u16: Vec<u16> = if ignore_case {
            kw.to_lowercase().encode_utf16().collect()
        } else {
            kw.encode_utf16().collect()
        };

        let kw_len_bytes = kw_u16.len() * 2;

        if mmap.len() < kw_len_bytes {
            continue;
        }

        let found = mmap.windows(kw_len_bytes).step_by(2).any(|window| {
            let w_u16 = window
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]));

            if ignore_case {
                w_u16
                    .map(|u| {
                        std::char::from_u32(u as u32)
                            .unwrap_or(' ')
                            .to_lowercase()
                            .next()
                            .unwrap() as u16
                    })
                    .eq(kw_u16.iter().copied())
            } else {
                w_u16.eq(kw_u16.iter().copied())
            }
        });

        if found {
            println!(
                "{}:{}",
                path.display().to_string().bright_red(),
                kw.bright_yellow()
            );
        }
    }
}

pub fn enum_sus_keywords(
    roots_raw: String,
    add_words_raw: String,
    exclude_exts_raw: String,
    exclude_dirs_raw: String,
    exclude_files_raw: String,
    exclude_paths_raw: String,
    exclude_words_raw: String,
    exclude_date_raw: String,
    match_date: String,
    only_raw: String,
    ignore_case: bool,
    binary: bool,
) {
    let roots = split_csv(roots_raw);
    let exclude_dirs = split_csv(exclude_dirs_raw);
    let exclude_files = split_csv(exclude_files_raw);
    let exclude_paths = split_csv(exclude_paths_raw);
    let exclude_exts: Vec<String> = split_csv(exclude_exts_raw)
        .into_iter()
        .map(|e| e.trim_start_matches('.').to_string())
        .collect();
    let exclude_words = split_csv(exclude_words_raw);

    let exclude_dates: Vec<get_date::DateFilter> = exclude_date_raw
        .split(',')
        .filter_map(|s| get_date::parse_filter(s.trim()))
        .collect();

    let mut keywords = if !only_raw.is_empty() {
        split_csv(only_raw)
    } else {
        let mut k = build_keywords(split_csv(add_words_raw));
        k.retain(|x| !exclude_words.iter().any(|e| e.eq_ignore_ascii_case(x)));
        k
    };

    if ignore_case {
        keywords.iter_mut().for_each(|k| *k = k.to_lowercase());
    }

    let mut stack: Vec<PathBuf> = roots.into_iter().map(PathBuf::from).collect();

    while let Some(root) = stack.pop() {
        if match_path(&root, &exclude_paths) {
            continue;
        }

        let mut pat = root.clone();
        pat.push("*");
        let pat_w: Vec<u16> = pat.as_os_str().encode_wide().chain([0]).collect();

        let mut data = WIN32_FIND_DATAW::default();
        let h = unsafe { FindFirstFileW(pat_w.as_ptr(), &mut data) };
        if h == INVALID_HANDLE_VALUE {
            continue;
        }

        loop {
            let len = data.cFileName.iter().position(|&c| c == 0).unwrap_or(260);
            let name = OsString::from_wide(&data.cFileName[..len]);

            if name != "." && name != ".." {
                let mut full = root.clone();
                full.push(&name);

                if !match_path(&full, &exclude_paths) {
                    let is_dir = (data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0;

                    if is_dir {
                        if !match_dir_name(&full, &exclude_dirs) {
                            stack.push(full);
                        }
                    } else if !match_file_name(&full, &exclude_files) {
                        let ext = full
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("")
                            .to_lowercase();

                        if !exclude_exts.contains(&ext) {
                            let Ok(meta) = full.metadata() else {
                                if unsafe { FindNextFileW(h, &mut data) } == 0 {
                                    break;
                                }
                                continue;
                            };

                            let t = meta.created().unwrap_or_else(|_| meta.modified().unwrap());

                            if !exclude_dates
                                .iter()
                                .any(|f| get_date::matches_filter(&t.into(), f))
                                && (match_date.is_empty() || match_file_date(t, &match_date))
                            {
                                use memmap2::Mmap;
                                use std::fs::File;

                                let Ok(file) = File::open(&full) else {
                                    if unsafe { FindNextFileW(h, &mut data) } == 0 {
                                        break;
                                    }
                                    continue;
                                };

                                let mmap = unsafe {
                                    match Mmap::map(&file) {
                                        Ok(m) => m,
                                        Err(_) => {
                                            if FindNextFileW(h, &mut data) == 0 {
                                                break;
                                            }
                                            continue;
                                        }
                                    }
                                };

                                if is_utf16_le(&mmap) {
                                    scan_utf16(&full, &keywords, ignore_case);
                                } else if binary {
                                    scan_binary(&full, &keywords, ignore_case);
                                } else {
                                    let text = String::from_utf8_lossy(&mmap);
                                    for (i, line) in text.lines().enumerate() {
                                        for kw in &keywords {
                                            let target = if ignore_case {
                                                line.to_lowercase()
                                            } else {
                                                line.to_string()
                                            };
                                            if let Some(pos) = target.find(kw) {
                                                let end = pos + kw.len();
                                                println!(
                                                    "{}:{}: {}{}{}",
                                                    full.display().to_string().bright_red(),
                                                    (i + 1).to_string().bright_yellow(),
                                                    &line[..pos],
                                                    &line[pos..end].red().bold(),
                                                    &line[end..]
                                                );
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if unsafe { FindNextFileW(h, &mut data) } == 0 {
                break;
            }
        }
        unsafe { FindClose(h) };
    }
}
