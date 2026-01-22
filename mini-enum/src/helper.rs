use colored::Colorize;

pub fn log_info(msg: &str) {
    println!("[{}] {}: {}", "*".bright_blue(), "INFO".bright_blue(), msg);
}

pub fn log_warn(msg: &str) {
    println!(
        "[{}] {}: {}",
        "!".bright_yellow(),
        "WARNING".bright_yellow(),
        msg
    );
}

pub fn log_error(msg: &str) {
    println!("[{}] {}", "ERR".red(), msg);
}

pub fn log_success(msg: &str) {
    println!("[{}] {}", "+".bright_red(), msg);
}
