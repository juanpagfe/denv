mod cli;
mod config;
mod proc_net;
mod proc_pid;
mod tracker;
mod output;
mod tui;
mod dns;

use std::process;

fn main() {
    let args = cli::parse();
    let config = config::Config::load(&args);

    let result = match &args.command {
        Some(cli::Command::Top) | None => {
            if output::is_tty() && !config.json && !config.csv {
                tui::run(&config)
            } else {
                output::run_oneshot(&config)
            }
        }
        Some(cli::Command::Connections) => output::run_oneshot(&config),
        Some(cli::Command::Process { pid }) => {
            let mut cfg = config;
            cfg.filter_pid = Some(*pid);
            output::run_oneshot(&cfg)
        }
        Some(cli::Command::Host { host }) => {
            let mut cfg = config;
            cfg.filter_host = Some(host.clone());
            output::run_oneshot(&cfg)
        }
        Some(cli::Command::Port { port }) => {
            let mut cfg = config;
            cfg.filter_port = Some(*port);
            output::run_oneshot(&cfg)
        }
        Some(cli::Command::Watch) => tui::run(&config),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
