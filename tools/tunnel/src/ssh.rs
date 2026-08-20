//! SSH connection management using russh (pure Rust, async).
//! Supports authentication via SSH agent and identity files.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use russh::client;
use russh_keys::key::PrivateKeyWithHashAlg;
use tokio::sync::Mutex;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// An established SSH session, wrapped in Arc<Mutex> so it can be shared.
pub struct SshSession {
    pub handle: Arc<Mutex<client::Handle<SshHandler>>>,
}

/// Minimal client handler for russh.
pub struct SshHandler;

#[async_trait::async_trait]
impl client::Handler for SshHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &ssh_key::PublicKey,
    ) -> std::result::Result<bool, Self::Error> {
        // Accept all host keys (like StrictHostKeyChecking=no).
        // Future enhancement: verify against known_hosts.
        Ok(true)
    }
}

/// Connect to an SSH server and authenticate.
pub async fn connect(
    host: &str,
    port: u16,
    user: &str,
    identity_file: Option<&Path>,
    keepalive_secs: u64,
) -> Result<SshSession> {
    let mut ssh_config = client::Config::default();
    ssh_config.keepalive_interval = if keepalive_secs > 0 {
        Some(std::time::Duration::from_secs(keepalive_secs))
    } else {
        None
    };
    ssh_config.keepalive_max = 3;

    let config = Arc::new(ssh_config);
    let handler = SshHandler;

    let addr = format!("{host}:{port}");
    log::info!("Connecting to {addr} as {user}");

    let mut handle = client::connect(config, &addr, handler).await?;

    // Try authentication methods in order
    let authenticated = try_auth(&mut handle, user, identity_file).await?;

    if !authenticated {
        return Err("authentication failed: no valid method".into());
    }

    log::info!("Connected and authenticated to {addr}");
    Ok(SshSession {
        handle: Arc::new(Mutex::new(handle)),
    })
}

/// Try authentication methods: agent first, then identity file.
async fn try_auth(
    handle: &mut client::Handle<SshHandler>,
    user: &str,
    identity_file: Option<&Path>,
) -> Result<bool> {
    // 1. Try SSH agent
    if let Ok(true) = try_agent_auth(handle, user).await {
        log::debug!("Authenticated via SSH agent");
        return Ok(true);
    }

    // 2. Try identity files
    let key_paths = collect_identity_paths(identity_file);
    for path in &key_paths {
        if let Ok(true) = try_key_auth(handle, user, path).await {
            log::debug!("Authenticated via key: {}", path.display());
            return Ok(true);
        }
    }

    Ok(false)
}

/// Attempt authentication via SSH agent.
async fn try_agent_auth(
    handle: &mut client::Handle<SshHandler>,
    user: &str,
) -> Result<bool> {
    let mut agent = match russh_keys::agent::client::AgentClient::connect_env().await {
        Ok(a) => a,
        Err(e) => {
            log::debug!("Cannot connect to SSH agent: {e}");
            return Ok(false);
        }
    };

    let identities = agent.request_identities().await?;

    for identity in identities {
        let pub_key = identity.clone();
        let auth_result = handle
            .authenticate_publickey_with(user, pub_key, &mut agent)
            .await;
        match auth_result {
            Ok(true) => return Ok(true),
            Ok(false) => continue,
            Err(_) => continue,
        }
    }

    Ok(false)
}

/// Attempt authentication with a private key file.
async fn try_key_auth(
    handle: &mut client::Handle<SshHandler>,
    user: &str,
    key_path: &Path,
) -> Result<bool> {
    if !key_path.exists() {
        return Ok(false);
    }

    let key_pair = match russh_keys::load_secret_key(key_path, None) {
        Ok(k) => k,
        Err(e) => {
            log::debug!("Cannot load key {}: {e}", key_path.display());
            return Ok(false);
        }
    };

    let key_with_alg = PrivateKeyWithHashAlg::new(Arc::new(key_pair), None)
        .map_err(|e| format!("key error: {e}"))?;

    let auth_result = handle
        .authenticate_publickey(user, key_with_alg)
        .await;

    match auth_result {
        Ok(ok) => Ok(ok),
        Err(e) => {
            log::debug!("Key auth failed for {}: {e}", key_path.display());
            Ok(false)
        }
    }
}

/// Collect identity file paths to try: explicit path + defaults.
fn collect_identity_paths(explicit: Option<&Path>) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(p) = explicit {
        paths.push(p.to_path_buf());
    }

    if let Some(home) = dirs::home_dir() {
        let ssh_dir = home.join(".ssh");
        for name in &["id_ed25519", "id_rsa", "id_ecdsa"] {
            let p = ssh_dir.join(name);
            if !paths.contains(&p) {
                paths.push(p);
            }
        }
    }

    paths
}

/// Parse a "host:port" or just "host" string.
pub fn parse_host_port(s: &str, default_port: u16) -> (String, u16) {
    if let Some(idx) = s.rfind(':') {
        let host = &s[..idx];
        if let Ok(port) = s[idx + 1..].parse::<u16>() {
            return (host.to_string(), port);
        }
    }
    (s.to_string(), default_port)
}
