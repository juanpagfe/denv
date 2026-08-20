//! Tunnel lifecycle management: start/stop named tunnels, reconnection logic.

use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;

use tokio::signal;

use crate::config::Config;
use crate::forward;
use crate::ssh;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Run an ad-hoc local forward from CLI args.
pub async fn run_adhoc_forward(
    local_port: u16,
    remote: &str,
    via: &str,
    _background: bool,
    config: &Config,
) -> Result<()> {
    let (ssh_host, ssh_port) = ssh::parse_host_port(via, config.ssh_port);
    let (remote_host, remote_port) = ssh::parse_host_port(remote, 0);
    let user = get_ssh_user(config, None);

    let quiet = config.quiet;
    run_with_reconnect(config, &ssh_host, ssh_port, &user, move |session| {
        let rh = remote_host.clone();
        async move {
            forward::local_forward(&session, "127.0.0.1", local_port, &rh, remote_port, quiet)
                .await
        }
    })
    .await
}

/// Run an ad-hoc remote forward from CLI args.
pub async fn run_adhoc_reverse(
    remote_port: u16,
    local: &str,
    via: &str,
    config: &Config,
) -> Result<()> {
    let (ssh_host, ssh_port) = ssh::parse_host_port(via, config.ssh_port);
    let (_local_host, _local_port) = ssh::parse_host_port(local, 0);
    let user = get_ssh_user(config, None);

    let session = connect_with_retry(
        &ssh_host,
        ssh_port,
        &user,
        config.identity_file.as_deref(),
        config.keepalive_secs,
        config,
    )
    .await?;

    if !config.quiet {
        eprintln!(
            "Reverse forward remote:{remote_port} -> {local} via {ssh_host}"
        );
    }

    // Request remote forwarding
    {
        let mut handle = session.handle.lock().await;
        let _accepted = handle
            .tcpip_forward("0.0.0.0", remote_port as u32)
            .await?;
    }

    if !config.quiet {
        eprintln!("Remote forwarding established (port {remote_port})");
    }

    // Wait for Ctrl-C
    signal::ctrl_c().await?;
    if !config.quiet {
        eprintln!("\nShutting down...");
    }

    Ok(())
}

/// Run an ad-hoc SOCKS proxy from CLI args.
pub async fn run_adhoc_socks(port: u16, via: &str, config: &Config) -> Result<()> {
    let (ssh_host, ssh_port) = ssh::parse_host_port(via, config.ssh_port);
    let user = get_ssh_user(config, None);

    let quiet = config.quiet;
    run_with_reconnect(config, &ssh_host, ssh_port, &user, move |session| {
        async move {
            forward::socks_proxy(&session, port, quiet).await
        }
    })
    .await
}

/// Start a named tunnel from config.
pub async fn start_named(name: &str, background: bool, config: &Config) -> Result<()> {
    let tunnel_def = config
        .get_tunnel(name)
        .ok_or_else(|| format!("tunnel '{name}' not found in config"))?
        .clone();

    let ssh_host = tunnel_def
        .ssh
        .host
        .as_deref()
        .ok_or_else(|| format!("tunnel '{name}' missing ssh.host"))?
        .to_string();
    let ssh_port = tunnel_def.ssh.port.unwrap_or(config.ssh_port);
    let user = get_ssh_user(config, tunnel_def.ssh.user.as_deref());

    if background {
        write_pid_file(name, config)?;
        eprintln!("Note: for true background mode, use: nohup tunnel start {name} &");
    }

    if !config.quiet {
        eprintln!("Starting tunnel '{name}'");
    }

    let quiet = config.quiet;
    match tunnel_def.tunnel_type.as_str() {
        "local" => {
            let local = tunnel_def
                .local
                .as_deref()
                .ok_or("missing 'local' in tunnel config")?;
            let remote = tunnel_def
                .remote
                .as_deref()
                .ok_or("missing 'remote' in tunnel config")?;
            let (local_host, local_port) = ssh::parse_host_port(local, 0);
            let (remote_host, remote_port) = ssh::parse_host_port(remote, 0);

            run_with_reconnect(config, &ssh_host, ssh_port, &user, move |session| {
                let rh = remote_host.clone();
                let lh = local_host.clone();
                async move {
                    forward::local_forward(&session, &lh, local_port, &rh, remote_port, quiet)
                        .await
                }
            })
            .await
        }
        "socks" => {
            let port = tunnel_def
                .port
                .ok_or("missing 'port' in socks tunnel config")?;
            run_with_reconnect(config, &ssh_host, ssh_port, &user, move |session| {
                async move {
                    forward::socks_proxy(&session, port, quiet).await
                }
            })
            .await
        }
        other => Err(format!("unsupported tunnel type: {other}").into()),
    }
}

/// Stop a named tunnel by removing its PID file.
pub async fn stop_named(name: &str, config: &Config) -> Result<()> {
    let pid_file = config.pid_dir.join(format!("{name}.pid"));
    if !pid_file.exists() {
        if !config.quiet {
            eprintln!("Tunnel '{name}' is not running");
        }
        return Ok(());
    }

    let pid_str = std::fs::read_to_string(&pid_file)?;
    if let Ok(pid) = pid_str.trim().parse::<i32>() {
        unsafe {
            libc::kill(pid, libc::SIGTERM);
        }
        std::fs::remove_file(&pid_file)?;
        if !config.quiet {
            eprintln!("Stopped tunnel '{name}' (pid {pid})");
        }
    } else {
        std::fs::remove_file(&pid_file)?;
        eprintln!("Removed stale PID file for '{name}'");
    }

    Ok(())
}

/// Restart a named tunnel.
pub async fn restart_named(name: &str, config: &Config) -> Result<()> {
    stop_named(name, config).await.ok();
    tokio::time::sleep(Duration::from_secs(1)).await;
    start_named(name, false, config).await
}

/// Show status of one or all tunnels.
pub async fn status(name: Option<&str>, config: &Config) -> Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();

    if let Some(name) = name {
        let running = is_tunnel_running(name, config);
        let defined = config.tunnels.contains_key(name);

        if config.json {
            let status = serde_json::json!({
                "name": name,
                "defined": defined,
                "running": running,
            });
            serde_json::to_writer_pretty(&mut out, &status)?;
            writeln!(out)?;
        } else {
            let status_str = if running { "running" } else { "stopped" };
            writeln!(out, "{name}: {status_str}")?;
        }
    } else {
        if config.tunnels.is_empty() {
            if !config.quiet {
                writeln!(out, "No tunnels configured.")?;
            }
            return Ok(());
        }

        if config.json {
            let statuses: Vec<serde_json::Value> = config
                .tunnels
                .iter()
                .map(|(name, def)| {
                    serde_json::json!({
                        "name": name,
                        "type": def.tunnel_type,
                        "running": is_tunnel_running(name, config),
                    })
                })
                .collect();
            serde_json::to_writer_pretty(&mut out, &statuses)?;
            writeln!(out)?;
        } else {
            writeln!(
                out,
                "{:<15} {:<10} {:<10} {:<20} {:<20}",
                "NAME", "TYPE", "STATUS", "LOCAL", "REMOTE"
            )?;
            for (name, def) in &config.tunnels {
                let status = if is_tunnel_running(name, config) {
                    "running"
                } else {
                    "stopped"
                };
                let local = def.local.as_deref().unwrap_or("-");
                let remote = def
                    .remote
                    .as_deref()
                    .or(def.ssh.host.as_deref())
                    .unwrap_or("-");
                writeln!(
                    out,
                    "{:<15} {:<10} {:<10} {:<20} {:<20}",
                    name, def.tunnel_type, status, local, remote
                )?;
            }
        }
    }

    Ok(())
}

/// List all configured tunnels.
pub async fn list(config: &Config) -> Result<()> {
    status(None, config).await
}

/// Show logs for a named tunnel.
pub async fn logs(name: &str, config: &Config) -> Result<()> {
    let log_file = config.log_dir.join(format!("{name}.log"));
    if !log_file.exists() {
        eprintln!("No logs found for tunnel '{name}'");
        return Ok(());
    }

    let content = std::fs::read_to_string(&log_file)?;
    print!("{content}");
    Ok(())
}

/// Test connectivity of a named tunnel.
pub async fn test_tunnel(name: &str, config: &Config) -> Result<()> {
    let tunnel_def = config
        .get_tunnel(name)
        .ok_or_else(|| format!("tunnel '{name}' not found in config"))?;

    let ssh_host = tunnel_def
        .ssh
        .host
        .as_deref()
        .ok_or_else(|| format!("tunnel '{name}' missing ssh.host"))?;
    let ssh_port = tunnel_def.ssh.port.unwrap_or(config.ssh_port);
    let user = get_ssh_user(config, tunnel_def.ssh.user.as_deref());

    eprintln!("Testing SSH connection to {ssh_host}:{ssh_port}...");

    let identity = tunnel_def
        .ssh
        .identity_file
        .as_ref()
        .map(|s| PathBuf::from(crate::config::shellexpand_pub(s)));

    match ssh::connect(
        ssh_host,
        ssh_port,
        &user,
        identity.as_deref().or(config.identity_file.as_deref()),
        config.keepalive_secs,
    )
    .await
    {
        Ok(_) => {
            eprintln!("✓ SSH connection successful");
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ SSH connection failed: {e}");
            Err(e)
        }
    }
}

// --- Helper functions ---

fn get_ssh_user(config: &Config, override_user: Option<&str>) -> String {
    override_user
        .map(|s| s.to_string())
        .or_else(|| config.ssh_user.clone())
        .unwrap_or_else(|| std::env::var("USER").unwrap_or_else(|_| "root".to_string()))
}

/// Connect to SSH with retry on failure.
async fn connect_with_retry(
    host: &str,
    port: u16,
    user: &str,
    identity: Option<&std::path::Path>,
    keepalive: u64,
    config: &Config,
) -> Result<ssh::SshSession> {
    #[allow(unused_assignments)]
    let mut delay_secs = config.reconnect_delay_secs;

    loop {
        match ssh::connect(host, port, user, identity, keepalive).await {
            Ok(session) => return Ok(session),
            Err(e) => {
                if !config.reconnect {
                    return Err(e);
                }
                eprintln!("Connection failed: {e}. Retrying in {delay_secs}s...");
                tokio::time::sleep(Duration::from_secs(delay_secs)).await;
                delay_secs = (delay_secs * 2).min(config.reconnect_max_delay_secs);
            }
        }
    }
}

/// Run a forwarding function with automatic reconnection.
async fn run_with_reconnect<F, Fut>(
    config: &Config,
    ssh_host: &str,
    ssh_port: u16,
    user: &str,
    make_future: F,
) -> Result<()>
where
    F: Fn(ssh::SshSession) -> Fut + Clone,
    Fut: std::future::Future<Output = Result<()>>,
{
    #[allow(unused_assignments)]
    let mut backoff = config.reconnect_delay_secs;

    loop {
        let session = connect_with_retry(
            ssh_host,
            ssh_port,
            user,
            config.identity_file.as_deref(),
            config.keepalive_secs,
            config,
        )
        .await?;

        backoff = config.reconnect_delay_secs;

        let result = tokio::select! {
            r = (make_future.clone())(session) => r,
            _ = signal::ctrl_c() => {
                if !config.quiet {
                    eprintln!("\nShutting down...");
                }
                return Ok(());
            }
        };

        match result {
            Ok(()) => return Ok(()),
            Err(e) => {
                if !config.reconnect {
                    return Err(e);
                }
                eprintln!("Connection lost: {e}. Reconnecting in {backoff}s...");
                tokio::time::sleep(Duration::from_secs(backoff)).await;
                #[allow(unused_assignments)]
                {
                    backoff = (backoff * 2).min(config.reconnect_max_delay_secs);
                }
            }
        }
    }
}

fn write_pid_file(name: &str, config: &Config) -> io::Result<()> {
    std::fs::create_dir_all(&config.pid_dir)?;
    let pid_file = config.pid_dir.join(format!("{name}.pid"));
    let pid = std::process::id();
    std::fs::write(&pid_file, pid.to_string())?;
    Ok(())
}

fn is_tunnel_running(name: &str, config: &Config) -> bool {
    let pid_file = config.pid_dir.join(format!("{name}.pid"));
    if !pid_file.exists() {
        return false;
    }
    if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            return std::path::Path::new(&format!("/proc/{pid}")).exists();
        }
    }
    false
}
