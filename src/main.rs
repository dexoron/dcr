use crate::utils::log::error;
mod cli;
mod config;
mod core;
mod platform;
mod utils;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        std::process::exit(cli::help::help());
    }

    let cmd = args[1].as_str();
    let rest = &args[2..];

    let code = match cmd {
        "new" => cli::r#new::new(rest),
        "init" => cli::init::init(rest),
        "build" => cli::build::build(rest),
        "run" => cli::run::run(rest),
        "clean" => cli::clean::clean(rest),
        "--version" => {
            println!("dcr {} ({})", env!("CARGO_PKG_VERSION"), env!("DCR_TARGET"));
            0
        }
        "--help" => cli::help::help(),
        "--update" => cli::flag_update::flag_update(rest),
        _ => {
            error("Unknown command or argument");
            0
        }
    };

    std::process::exit(code);
}
