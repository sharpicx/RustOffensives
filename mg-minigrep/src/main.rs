mod argparser;
mod find;
mod get_date;
mod get_drives;
mod get_exts;
mod grep;

fn main() {
    let cli = argparser::parse_cli();
    argparser::run(cli);
}
