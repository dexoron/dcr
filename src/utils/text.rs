pub const RESET: &str = "\x1b[0m";

pub const BOLD: &str = "\x1b[1m";

pub const BRIGHT_RED: &str = "\x1b[91m";
pub const BRIGHT_GREEN: &str = "\x1b[92m";
pub const BRIGHT_YELLOW: &str = "\x1b[93m";
pub const BRIGHT_CYAN: &str = "\x1b[96m";

pub fn colored(msg: &str, style: &str) -> String {
    format!("{style}{msg}{RESET}")
}

pub fn printc(msg: &str, style: &str) {
    println!("{style}{msg}{RESET}");
}
