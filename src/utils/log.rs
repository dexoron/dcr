use crate::utils::text::{BOLD, BRIGHT_GREEN, BRIGHT_RED, BRIGHT_YELLOW, colored};

#[allow(dead_code)]
pub fn error(msg: &str) {
    println!(
        "{}: {msg}",
        colored("error", &(BRIGHT_RED.to_owned() + BOLD))
    );
}

#[allow(dead_code)]
pub fn warn(msg: &str) {
    println!(
        "{}: {msg}",
        colored("warn", &(BRIGHT_YELLOW.to_owned() + BOLD))
    );
}

#[allow(dead_code)]
pub fn info(msg: &str) {
    println!("{}: {msg}", colored("info", BOLD));
}

#[allow(dead_code)]
pub fn ok(msg: &str) {
    println!(
        "{}: {msg}",
        colored("ok", &(BRIGHT_GREEN.to_owned() + BOLD))
    );
}
