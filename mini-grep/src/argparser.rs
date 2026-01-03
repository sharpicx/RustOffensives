use clap::CommandFactory;
use clap::{Parser, Subcommand};

use crate::enum_date;
use crate::enum_drives;
use crate::enum_sus_files_dirs;
use crate::enum_sus_keywords;

#[derive(Parser)]
#[command(name = "EFEW")]
#[command(version = "1.33.7")]
#[command(about = "minigrep implementation by @sharpicx.")]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Args>,
}

#[derive(Subcommand)]
pub enum Args {
    #[command(about = "Enumerates drives")]
    EnumDrives,

    #[command(
        about = "Enumerates and collects all dates of created files, directories, in the system"
    )]
    EnumDate {
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

    #[command(about = "Enumerates all suspicious created files, directories, folders by date")]
    EnumSusFiles {
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
    },

    #[command(
        about = "Enumerates all files, directories, folders that have sensitive keywords (general & date)"
    )]
    EnumSusWord {
        #[arg(long)]
        dirs: Option<String>,

        #[arg(long = "add-words")]
        add_words: Option<String>,

        #[arg(long = "exclude-exts")]
        exclude_exts: Option<String>,

        #[arg(long = "exclude-dirs")]
        exclude_dirs: Option<String>,

        #[arg(long = "exclude-words")]
        exclude_words: Option<String>,

        #[arg(long = "exclude-date")]
        exclude_date: Option<String>,

        #[arg(long = "match-date")]
        match_date: Option<String>,
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
        Some(Args::EnumDate {
            drives,
            filter_date,
        }) => {
            if drives.is_none() && filter_date.is_none() {
                let mut cmd = Cli::command();
                print_help_subcommand(&mut cmd, "enum-date");
                return;
            }
            enum_date::enum_date(drives.unwrap_or_default(), filter_date.unwrap_or_default())
        }

        Some(Args::EnumDrives) => enum_drives::enum_drives(),
        Some(Args::EnumSusWord {
            dirs,
            add_words,
            exclude_exts,
            exclude_dirs,
            exclude_words,
            exclude_date,
            match_date,
        }) => {
            if dirs.is_none()
                && add_words.is_none()
                && exclude_words.is_none()
                && exclude_dirs.is_none()
                && exclude_dirs.is_none()
                && exclude_words.is_none()
                && exclude_date.is_none()
                && match_date.is_none()
            {
                let mut cmd = Cli::command();
                print_help_subcommand(&mut cmd, "enum-sus-word");
                return;
            }
            enum_sus_keywords::enum_sus_keywords(
                dirs.unwrap_or_default(),
                add_words.unwrap_or_default(),
                exclude_exts.unwrap_or_default(),
                exclude_dirs.unwrap_or_default(),
                exclude_words.unwrap_or_default(),
                exclude_date.unwrap_or_default(),
                match_date.unwrap_or_default(),
            );
        }
        Some(Args::EnumSusFiles {
            match_date,
            match_name,
            add_roots,
            exclude_roots,
            exclude_exts,
            exclude_date,
        }) => {
            if match_date.is_none()
                && match_name.is_none()
                && add_roots.is_empty()
                && exclude_roots.is_none()
                && exclude_exts.is_none()
                && exclude_date.is_none()
            {
                let mut cmd = Cli::command();
                print_help_subcommand(&mut cmd, "enum-sus-files");
                return;
            }

            enum_sus_files_dirs::enum_sus_files_dirs(
                match_date,
                match_name,
                add_roots,
                exclude_roots,
                exclude_exts,
                exclude_date,
            )
        }
        None => {
            print_help_global();
        }
    }
}
