use clap::CommandFactory;
use clap::{Parser, Subcommand};

use crate::find;
use crate::get_date;
use crate::get_drives;
use crate::get_exts;
use crate::grep;

#[derive(Parser)]
#[command(name = "mg (minigrep)")]
#[command(version = "1.33.7")]
#[command(about = "mg (minigrep) implementation by @sharpicx.")]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Args>,
}

#[derive(Subcommand)]
enum Args {
    #[command(about = "Enumerates drives")]
    GetDrives,

    #[command(
        about = "Enumerates and collects all dates of created files, directories, in the system"
    )]
    GetDate {
        #[arg(
            long,
            help = r#"Select drives to be used for collecting dates, e.g: C:\"#
        )]
        drives: Option<String>,

        #[arg(
            long = "filter-date",
            help = "Like ffuf, select a date to be removed from scan actions"
        )]
        filter_date: Option<String>,
    },

    #[command(about = "Enumerates all file extensions")]
    GetExts {
        #[arg(long = "roots", help = "Select roots folder you want to enumerate")]
        roots: Option<String>,

        #[arg(long = "exclude-roots", help = "Excluding roots folder / drives")]
        exclude_roots: Option<String>,
    },

    #[command(about = "Enumerates all suspicious created files, directories, folders by date")]
    Find {
        #[arg(long)]
        match_date: Option<String>,

        #[arg(long)]
        match_name: Option<String>,

        #[arg(long = "add-roots")]
        add_roots: Vec<String>,

        #[arg(long = "exclude-roots")]
        exclude_roots: Option<String>,

        #[arg(long = "exclude-exts")]
        exclude_exts: Option<String>,

        #[arg(long = "exclude-date")]
        exclude_date: Option<String>,

        #[arg(long = "exclude-folders")]
        exclude_dirs: Option<String>,
    },

    #[command(
        about = "Enumerates all files, directories, folders that have sensitive keywords (general & date)"
    )]
    Grep {
        #[arg(long = "roots")]
        roots: Option<String>,

        #[arg(long = "add-words", conflicts_with = "only")]
        add_words: Option<String>,

        #[arg(long, conflicts_with = "add_words")]
        only: Option<String>,

        #[arg(long = "exclude-exts")]
        exclude_exts: Option<String>,

        #[arg(long = "exclude-dirs")]
        exclude_dirs: Option<String>,

        #[arg(long = "exclude-files")]
        exclude_files: Option<String>,

        #[arg(long = "exclude-paths")]
        exclude_paths: Option<String>,

        #[arg(long = "exclude-words")]
        exclude_words: Option<String>,

        #[arg(long = "exclude-date")]
        exclude_date: Option<String>,

        #[arg(long = "match-date")]
        match_date: Option<String>,

        #[arg(short = 'i', long = "ignore-case", default_value_t = true)]
        ignore_case: bool,

        #[arg(long)]
        binary: bool,
    },
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
        Some(Args::GetDate {
            drives,
            filter_date,
        }) => {
            if drives.is_none() && filter_date.is_none() {
                let mut cmd = Cli::command();
                print_help_subcommand(&mut cmd, "get-date");
                return;
            }
            get_date::enum_date(drives.unwrap_or_default(), filter_date.unwrap_or_default())
        }
        Some(Args::GetExts {
            roots,
            exclude_roots,
        }) => {
            if roots.is_none() && exclude_roots.is_none() {
                let mut cmd = Cli::command();
                print_help_subcommand(&mut cmd, "get-exts");
                return;
            }
            let r = roots.unwrap_or_else(|| "C:\\".to_string());
            let er = exclude_roots.unwrap_or_else(|| "".to_string());
            get_exts::run_get_exts(r, er);
        }
        Some(Args::GetDrives) => get_drives::enum_drives(),
        Some(Args::Grep {
            roots,
            add_words,
            exclude_exts,
            exclude_dirs,
            exclude_files,
            exclude_paths,
            exclude_words,
            exclude_date,
            match_date,
            only,
            ignore_case,
            binary,
        }) => {
            if roots.is_none() {
                let mut cmd = Cli::command();
                print_help_subcommand(&mut cmd, "grep");
                return;
            }
            grep::enum_sus_keywords(
                roots.unwrap(),
                add_words.unwrap_or_default(),
                exclude_exts.unwrap_or_default(),
                exclude_dirs.unwrap_or_default(),
                exclude_files.unwrap_or_default(),
                exclude_paths.unwrap_or_default(),
                exclude_words.unwrap_or_default(),
                exclude_date.unwrap_or_default(),
                match_date.unwrap_or_default(),
                only.unwrap_or_default(),
                ignore_case,
                binary,
            );
        }
        Some(Args::Find {
            match_date,
            match_name,
            add_roots,
            exclude_roots,
            exclude_exts,
            exclude_date,
            exclude_dirs,
        }) => {
            if match_date.is_none()
                && match_name.is_none()
                && add_roots.is_empty()
                && exclude_roots.is_none()
                && exclude_exts.is_none()
                && exclude_date.is_none()
                && exclude_dirs.is_none()
            {
                let mut cmd = Cli::command();
                print_help_subcommand(&mut cmd, "find");
                return;
            }

            find::enum_sus_files_dirs(
                match_date,
                match_name,
                add_roots,
                exclude_roots,
                exclude_exts,
                exclude_date,
                exclude_dirs,
            )
        }
        None => {
            print_help_global();
        }
    }
}
