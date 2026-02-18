#[allow(dead_code)]
pub const RESET: &str = "\x1b[0m";

#[allow(dead_code)]
pub const BOLD: &str = "\x1b[1m";

#[allow(dead_code)]
pub const BRIGHT_RED: &str = "\x1b[91m";
#[allow(dead_code)]
pub const BRIGHT_GREEN: &str = "\x1b[92m";
#[allow(dead_code)]
pub const BRIGHT_YELLOW: &str = "\x1b[93m";
#[allow(dead_code)]
pub const BRIGHT_CYAN: &str = "\x1b[96m";
#[allow(dead_code)]
pub const BOLD_RED: &str = "\x1b[1m\x1b[91m";
#[allow(dead_code)]
pub const BOLD_GREEN: &str = "\x1b[1m\x1b[92m";
#[allow(dead_code)]
pub const BOLD_YELLOW: &str = "\x1b[1m\x1b[93m";
#[allow(dead_code)]
pub const BOLD_CYAN: &str = "\x1b[1m\x1b[96m";

#[allow(dead_code)]
pub fn colored(msg: &str, style: &str) -> String {
    format!("{style}{msg}{RESET}")
}

#[allow(dead_code)]
pub fn printc(msg: &str, style: &str) {
    println!("{style}{msg}{RESET}");
}
