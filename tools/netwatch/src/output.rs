//! Non-interactive output modes: table, JSON, CSV.

use std::io::{self, Write};

use crate::config::Config;
use crate::tracker::{Connection, Tracker};

/// Check if stdout is a TTY.
pub fn is_tty() -> bool {
    crossterm::tty::IsTty::is_tty(&io::stdout())
}

/// Run a single snapshot and print results.
pub fn run_oneshot(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut tracker = Tracker::new();
    let connections = tracker.refresh(config);

    if config.json {
        print_json(&connections)?;
    } else if config.csv {
        print_csv(&connections)?;
    } else {
        print_table(&connections, config)?;
    }

    Ok(())
}

fn print_table(
    connections: &[Connection],
    config: &Config,
) -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let use_color = !config.no_color && is_tty();

    if connections.is_empty() {
        if !config.quiet {
            writeln!(out, "No connections found.")?;
        }
        return Ok(());
    }

    // Header
    if use_color {
        writeln!(
            out,
            "\x1b[1m{:<7} {:<15} {:<5} {:<12} {:<22} {:<22} {:<8}\x1b[0m",
            "PID", "PROCESS", "PROTO", "STATE", "LOCAL", "REMOTE", "USER"
        )?;
    } else {
        writeln!(
            out,
            "{:<7} {:<15} {:<5} {:<12} {:<22} {:<22} {:<8}",
            "PID", "PROCESS", "PROTO", "STATE", "LOCAL", "REMOTE", "USER"
        )?;
    }

    for conn in connections {
        let pid_str = conn
            .pid
            .map(|p| p.to_string())
            .unwrap_or_else(|| "-".to_string());

        let remote_str = match &conn.hostname {
            Some(host) => format!("{}:{}", host, conn.remote.port()),
            None => conn.remote.to_string(),
        };

        // Truncate process name
        let proc_name = if conn.process_name.len() > 15 {
            format!("{}…", &conn.process_name[..14])
        } else {
            conn.process_name.clone()
        };

        if use_color {
            let state_color = match conn.state.as_str() {
                "ESTABLISHED" => "\x1b[32m",
                "LISTEN" => "\x1b[36m",
                "TIME_WAIT" | "CLOSE_WAIT" => "\x1b[33m",
                _ => "\x1b[2m",
            };

            writeln!(
                out,
                "{:<7} {:<15} {:<5} {}{:<12}\x1b[0m {:<22} {:<22} {:<8}",
                pid_str,
                proc_name,
                conn.protocol.as_str(),
                state_color,
                conn.state.as_str(),
                conn.local.to_string(),
                remote_str,
                conn.user,
            )?;
        } else {
            writeln!(
                out,
                "{:<7} {:<15} {:<5} {:<12} {:<22} {:<22} {:<8}",
                pid_str,
                proc_name,
                conn.protocol.as_str(),
                conn.state.as_str(),
                conn.local.to_string(),
                remote_str,
                conn.user,
            )?;
        }
    }

    if !config.quiet {
        writeln!(out)?;
        writeln!(out, "{} connections", connections.len())?;
    }

    Ok(())
}

fn print_json(connections: &[Connection]) -> io::Result<()> {
    let entries: Vec<serde_json::Value> = connections
        .iter()
        .map(|c| {
            serde_json::json!({
                "pid": c.pid,
                "process": c.process_name,
                "protocol": c.protocol.as_str(),
                "state": c.state.as_str(),
                "local": c.local.to_string(),
                "remote": c.remote.to_string(),
                "hostname": c.hostname,
                "user": c.user,
            })
        })
        .collect();

    let stdout = io::stdout();
    serde_json::to_writer_pretty(stdout.lock(), &entries)?;
    println!();
    Ok(())
}

fn print_csv(connections: &[Connection]) -> io::Result<()> {
    let stdout = io::stdout();
    let mut wtr = csv::Writer::from_writer(stdout.lock());

    wtr.write_record([
        "pid", "process", "protocol", "state", "local", "remote", "hostname", "user",
    ])?;

    for c in connections {
        wtr.write_record(&[
            c.pid.map(|p| p.to_string()).unwrap_or_default(),
            c.process_name.clone(),
            c.protocol.as_str().to_string(),
            c.state.as_str().to_string(),
            c.local.to_string(),
            c.remote.to_string(),
            c.hostname.clone().unwrap_or_default(),
            c.user.clone(),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}
