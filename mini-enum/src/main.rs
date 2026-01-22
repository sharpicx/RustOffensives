mod applocker;
mod argparser;
mod arptable;
mod autorun;
mod cloud;
mod credentials;
mod credenum;
mod dnscaches;
mod enum_privs;
mod enum_users_groups;
mod helper;
mod lib;
mod logon_sessions;
mod microsoft_updates;
mod powershell_history;
mod procs;
mod rdp;
mod recyclebin;
mod scheduled_task;
mod tcp;
mod udp;
mod winlogon;
mod winsvc;

fn main() {
    let cli = argparser::parse_cli();
    argparser::run(cli);
}
