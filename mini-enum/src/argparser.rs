use clap::CommandFactory;
use clap::{Parser, Subcommand};

use crate::applocker;
use crate::arptable;
use crate::autorun;
use crate::cloud;
use crate::credentials;
use crate::credenum;
use crate::dnscaches;
use crate::enum_privs;
use crate::enum_users_groups;
use crate::helper::log_info;
use crate::logon_sessions;
use crate::microsoft_updates;
use crate::powershell_history;
use crate::procs;
use crate::rdp;
use crate::recyclebin;
use crate::scheduled_task;
use crate::tcp;
use crate::udp;
use crate::winlogon;
use crate::winsvc;

#[derive(Parser)]
#[command(name = "enum")]
#[command(version = "0.1.0")]
#[command(about = "mini enumeration tool")]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Args>,
}

#[derive(Subcommand)]
pub enum Args {
    #[command(about = "Enumerates applocker settings")]
    Applocker,

    #[command(about = "Prints arp tables like ipconfig.exe")]
    Arptables,

    #[command(about = "Auto run executables/scripts/programs")]
    Autoruns,

    #[command(about = "Enumerates cloud credentials")]
    Cloudcreds,

    #[command(about = "Enumerates the current user's saved credentials using CredEnumerate()")]
    Credenum,

    #[command(about = "Enumerates DNS caches")]
    DnsCaches,

    #[command(about = r#"Enumerates \Microsoft\Credentials"#)]
    EnumCreds,

    #[command(about = "Enumerates users & groups")]
    EnumUsersGroups,

    #[command(
        about = "Enumerates windows suspicious privileges (e.g: SeRestorePrivilege, SeManageVolumePrivilege, SeDebugPrivilege)"
    )]
    EnumPrivs,

    #[command(about = "Enumerates logon sessions")]
    LogonSessions,

    #[command(about = "Enumerates windows updates")]
    MicrosoftUpdates,

    #[command(about = "Gets powershell history")]
    PwshHistory {
        #[arg(long = "regex")]
        regex: bool,
    },

    #[command(about = "Enumerates processes")]
    Procs,

    #[command(about = "Enumerates RDP saved credentials")]
    RdpSavedCreds,

    #[command(about = "Checks for any Recycle Bin on Local or Active Directory Environment")]
    RecycleBin {
        #[arg(long = "ad")]
        active_directory: bool,

        #[arg(long = "local")]
        local: bool,
    },

    #[command(about = "Enumerates scheduled tasks")]
    ScheduledTask,

    #[command(about = "Current TCP connections and their associated processes and services")]
    Tcp,

    #[command(about = "Current UDP connections and their associated processes and services")]
    Udp,

    #[command(about = "Enumerates windows services")]
    Winsvc,

    #[command(about = "Registry autologon information")]
    Winlogon,
}

pub fn parse_cli() -> Cli {
    Cli::parse()
}

fn print_help_global() {
    Cli::command().print_help().unwrap();
    println!();
}

fn print_help_subcommand(cmd: &mut clap::Command, name: &str) {
    if let Some(subcmd) = cmd.get_subcommands_mut().find(|s| s.get_name() == name) {
        subcmd.print_help().unwrap();
    } else {
        cmd.print_help().unwrap();
        println!();
    }
}

pub fn run(cli: Cli) {
    match cli.command {
        Some(Args::Applocker) => {
            let _ = applocker::enum_applocker_settings();
        }
        Some(Args::Arptables) => arptable::print_arp_tables(),

        Some(Args::Autoruns) => autorun::get_autorun(),

        Some(Args::Cloudcreds) => cloud::enum_cloud_creds(),

        Some(Args::Credenum) => {
            credenum::list_credentials();
        }

        Some(Args::DnsCaches) => dnscaches::enum_dns_caches(),

        Some(Args::EnumCreds) => {
            credentials::enum_creds();
        }

        Some(Args::EnumPrivs) => {
            let _ = enum_privs::enum_privs();
        }

        Some(Args::EnumUsersGroups) => {
            enum_users_groups::enum_common_groups();
            let _ = enum_users_groups::enum_desc_users();
            let _ = enum_users_groups::enum_desc_groups();
        }

        Some(Args::LogonSessions) => {
            let _ = logon_sessions::enum_logon_sessions();
        }

        Some(Args::MicrosoftUpdates) => {
            let _ = microsoft_updates::enum_windows_updates();
        }

        Some(Args::RdpSavedCreds) => rdp::now(),

        Some(Args::RecycleBin {
            active_directory,
            local,
        }) => {
            if active_directory {
                let commands = vec![
                    "Get-ADOptionalFeature 'Recycle Bin Feature'",
                    "Get-ADForest | Select ForestMode",
                    "Get-ADObject -Filter 'isDeleted -eq $true -and name -ne \"Deleted Objects\"' -IncludeDeletedObjects",
                    "Get-ADObject -Filter 'isDeleted -eq $true' -IncludeDeletedObjects -Properties lastKnownParent,objectSid",
                    "https://0xdf.gitlab.io/2025/11/01/htb-voleur.html#recover-account",
                    "https://0xdf.gitlab.io/2025/10/11/htb-tombwatcher.html#ad-recycle-bin",
                    "https://0xdf.gitlab.io/2020/07/25/htb-cascade.html#ad-recycle",
                ];

                for cmd in commands {
                    log_info(cmd);
                }
                println!();
            } else if local {
                let _ = recyclebin::local_recyclebin();
            } else {
                let mut cmd = Cli::command();
                print_help_subcommand(&mut cmd, "recycle-bin");
            }
        }

        Some(Args::PwshHistory { regex }) => {
            powershell_history::enum_powershell_histories(regex);
        }

        Some(Args::Procs) => {
            let _ = procs::all_procs_print();
        }

        Some(Args::ScheduledTask) => {
            scheduled_task::enum_scheduled_tasks();
        }

        Some(Args::Tcp) => {
            let _ = tcp::tcp_connections();
        }

        Some(Args::Udp) => {
            let _ = udp::udp_connections();
        }

        Some(Args::Winsvc) => {
            let _ = winsvc::enum_windows_services();
        }

        Some(Args::Winlogon) => {
            let _ = winlogon::enum_autologon();
        }

        None => {
            print_help_global();
        }
    }
}
