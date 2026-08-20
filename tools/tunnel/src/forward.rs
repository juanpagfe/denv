//! Port forwarding implementations: local, remote, and SOCKS proxy.

use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::ssh::{SshHandler, SshSession};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Run a local port forward: listen on local_port, forward to remote_host:remote_port via SSH.
pub async fn local_forward(
    session: &SshSession,
    local_addr: &str,
    local_port: u16,
    remote_host: &str,
    remote_port: u16,
    quiet: bool,
) -> Result<()> {
    let bind_addr = format!("{local_addr}:{local_port}");
    let listener = TcpListener::bind(&bind_addr).await?;

    if !quiet {
        eprintln!(
            "Forwarding {} -> {}:{} via SSH",
            bind_addr, remote_host, remote_port
        );
    }

    loop {
        let (stream, peer) = listener.accept().await?;
        log::debug!("Accepted connection from {peer}");

        let handle = session.handle.clone();
        let rhost = remote_host.to_string();
        let rport = remote_port;

        tokio::spawn(async move {
            if let Err(e) = handle_local_forward(handle, stream, &rhost, rport).await {
                log::warn!("Forward error from {peer}: {e}");
            }
        });
    }
}

async fn handle_local_forward(
    handle: Arc<Mutex<russh::client::Handle<SshHandler>>>,
    mut local_stream: TcpStream,
    remote_host: &str,
    remote_port: u16,
) -> Result<()> {
    let channel = {
        let h = handle.lock().await;
        h.channel_open_direct_tcpip(
            remote_host,
            remote_port as u32,
            "127.0.0.1",
            0,
        )
        .await?
    };

    let mut channel_stream = channel.into_stream();
    tokio::io::copy_bidirectional(&mut local_stream, &mut channel_stream).await?;

    Ok(())
}

/// Run a SOCKS5 proxy: listen on local_port, forward connections via SSH.
pub async fn socks_proxy(
    session: &SshSession,
    port: u16,
    quiet: bool,
) -> Result<()> {
    let bind_addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&bind_addr).await?;

    if !quiet {
        eprintln!("SOCKS5 proxy listening on {bind_addr}");
    }

    loop {
        let (stream, peer) = listener.accept().await?;
        log::debug!("SOCKS connection from {peer}");

        let handle = session.handle.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_socks(handle, stream).await {
                log::debug!("SOCKS error from {peer}: {e}");
            }
        });
    }
}

/// Handle a single SOCKS5 connection.
async fn handle_socks(
    handle: Arc<Mutex<russh::client::Handle<SshHandler>>>,
    mut stream: TcpStream,
) -> Result<()> {
    // SOCKS5 greeting
    let mut buf = [0u8; 256];
    let n = stream.read(&mut buf).await?;
    if n < 2 || buf[0] != 0x05 {
        return Err("not a SOCKS5 request".into());
    }

    // Reply: no authentication required
    stream.write_all(&[0x05, 0x00]).await?;

    // Read connect request
    let mut req = [0u8; 4];
    stream.read_exact(&mut req).await?;

    if req[0] != 0x05 || req[1] != 0x01 {
        // Only support CONNECT command
        stream
            .write_all(&[0x05, 0x07, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
            .await?;
        return Err("unsupported SOCKS5 command".into());
    }

    let (host, port) = match req[3] {
        0x01 => {
            // IPv4
            let mut addr = [0u8; 4];
            stream.read_exact(&mut addr).await?;
            let mut port_buf = [0u8; 2];
            stream.read_exact(&mut port_buf).await?;
            let port = u16::from_be_bytes(port_buf);
            let host = format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3]);
            (host, port)
        }
        0x03 => {
            // Domain name
            let mut len = [0u8; 1];
            stream.read_exact(&mut len).await?;
            let mut domain = vec![0u8; len[0] as usize];
            stream.read_exact(&mut domain).await?;
            let mut port_buf = [0u8; 2];
            stream.read_exact(&mut port_buf).await?;
            let port = u16::from_be_bytes(port_buf);
            let host = String::from_utf8_lossy(&domain).to_string();
            (host, port)
        }
        0x04 => {
            // IPv6
            let mut addr = [0u8; 16];
            stream.read_exact(&mut addr).await?;
            let mut port_buf = [0u8; 2];
            stream.read_exact(&mut port_buf).await?;
            let port = u16::from_be_bytes(port_buf);
            let ip = std::net::Ipv6Addr::from(addr);
            (ip.to_string(), port)
        }
        _ => {
            stream
                .write_all(&[0x05, 0x08, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
                .await?;
            return Err("unsupported address type".into());
        }
    };

    log::debug!("SOCKS CONNECT to {host}:{port}");

    // Open SSH channel to target
    let channel = {
        let h = handle.lock().await;
        match h
            .channel_open_direct_tcpip(&host, port as u32, "127.0.0.1", 0)
            .await
        {
            Ok(c) => c,
            Err(e) => {
                // Connection refused reply
                stream
                    .write_all(&[0x05, 0x05, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
                    .await?;
                return Err(format!("SSH channel error: {e}").into());
            }
        }
    };

    // Success reply
    stream
        .write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
        .await?;

    // Bidirectional copy
    let mut channel_stream = channel.into_stream();
    tokio::io::copy_bidirectional(&mut stream, &mut channel_stream).await?;

    Ok(())
}
