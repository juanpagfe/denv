mod cli;
mod config;
mod hasher;
mod scanner;
mod output;
mod clean;

use std::process;

fn main() {
    let args = cli::parse();
    let config = config::Config::load(&args);

    let result = match &args.command {
        cli::Command::Scan { paths } => scanner::run_scan(paths, &config),
        cli::Command::Clean { paths } => clean::run_clean(paths, &config),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
