use crate::utils::text::{BOLD_CYAN, colored};

#[allow(dead_code)]
pub fn error(msg: &str) {
    println!("{}: {msg}", colored("error", BOLD_CYAN),);
}

#[allow(dead_code)]
pub fn warn(msg: &str) {
    println!("{}: {msg}", colored("warn", BOLD_CYAN),);
}

// #[allow(dead_code)]
// pub fn info(msg: &str) {
//     println!("{}: {msg}", colored("info", BOLD));
// }
//
// #[allow(dead_code)]
// pub fn ok(msg: &str) {
//     println!(
//         "{}: {msg}",
//         colored("ok", &(BRIGHT_GREEN.to_owned() + BOLD))
//     );
// }
