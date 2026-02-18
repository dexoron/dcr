use crate::utils::text::{BOLD_CYAN, BOLD_GREEN, printc};

pub fn help() -> i32 {
    println!("DCR (Dexoron Cargo Realization)");
    println!("C project manager inspired by Cargo.");
    println!();
    printc("USAGE:", BOLD_GREEN);
    printc("    dcr <command> [options]", BOLD_CYAN);
    println!();
    printc("COMMANDS:", BOLD_GREEN);
    println!("    new <name>        Create a new project");
    println!("    init              Initialize the current directory as a project");
    println!("    build [--profile] Build the project (default: --debug)");
    println!("    run [--profile]   Build and run the project (default: --debug)");
    println!("    clean             Remove the target directory");
    printc("FLAGS:", BOLD_GREEN);
    println!("    --help            Show command help");
    println!("    --update          Update dcr to the latest version");
    println!("    --version         Show dcr version");
    println!();
    printc("OPTIONS:", BOLD_GREEN);
    println!("    --debug           Build with debug profile");
    println!("    --release         Build with release profile");
    println!();
    printc("EXAMPLES:", BOLD_GREEN);
    printc("    dcr new hello", BOLD_CYAN);
    printc("    dcr build --release", BOLD_CYAN);
    printc("    dcr run --debug", BOLD_CYAN);
    println!();
    printc("TIP:", BOLD_GREEN);
    println!("    Run 'dcr <command> --help' for command-specific help.");
    0
}
