mod cli;
mod config;
mod ssh;
mod forward;
mod manager;

use std::process;

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("warn"),
    )
    .format_timestamp(Some(env_logger::TimestampPrecision::Seconds))
    .init();

    let args = cli::parse();
    let config = config::Config::load(&args);

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

    let result = rt.block_on(async {
        match &args.command {
            cli::Command::Forward {
                local_port,
                remote,
                via,
            } => {
                manager::run_adhoc_forward(
                    *local_port, remote, via, false, &config,
                )
                .await
            }
            cli::Command::Reverse {
                remote_port,
                local,
                via,
            } => {
                manager::run_adhoc_reverse(
                    *remote_port, local, via, &config,
                )
                .await
            }
            cli::Command::Socks { port, via } => {
                manager::run_adhoc_socks(*port, via, &config).await
            }
            cli::Command::Start { name, background } => {
                manager::start_named(name, *background, &config).await
            }
            cli::Command::Stop { name } => manager::stop_named(name, &config).await,
            cli::Command::Restart { name } => {
                manager::restart_named(name, &config).await
            }
            cli::Command::Status { name } => {
                manager::status(name.as_deref(), &config).await
            }
            cli::Command::List => manager::list(&config).await,
            cli::Command::Logs { name } => manager::logs(name, &config).await,
            cli::Command::Test { name } => manager::test_tunnel(name, &config).await,
        }
    });

    if let Err(e) = result {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
