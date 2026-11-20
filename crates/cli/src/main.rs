use std::process::ExitCode;

use clap::FromArgMatches;
use phonedriver::commands::{Cli, command, run};

// Windows gives a program's first thread 1 MB of stack. A debug build overflowed it before it read its arguments
// (CI, 2026-11-20), and `doctor` with an iPhone on Wi-Fi overflowed it in a debug build (#29).
const STACK: usize = 16 * 1024 * 1024;

fn main() -> ExitCode {
    std::thread::Builder::new()
        .stack_size(STACK)
        .spawn(|| {
            let matches = command().get_matches();
            run(Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit()))
        })
        .expect("the OS refused a thread for phonedriver")
        .join()
        .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
}
