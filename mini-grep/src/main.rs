mod argparser;
mod enum_date;
mod enum_drives;
mod enum_sus_files_dirs;
mod enum_sus_keywords;

fn main() {
    let cli = argparser::parse_cli();
    argparser::run(cli);
}
