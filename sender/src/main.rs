use anyhow::{Context, Result};
use clap::Parser;
use common::MAX_MAVLINK_PACKET_SIZE;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tracing::{error, info, warn};

const RETRY_BACKOFF_MS: u64 = 1000;

#[derive(Parser, Debug)]
#[command(author, version, about = "MAVLink UDP forwarder for aircraft telemetry")]
struct Args {
    /// Address to listen on for MAVLink packets from mavlink-router
    #[arg(short, long, default_value = "0.0.0.0:14551")]
    listen: SocketAddr,

    /// Address to forward packets to (ground station)
    #[arg(short, long, env = "GROUND_ADDR")]
    forward: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    info!("MAVLink Sender starting");
    info!("Listen address: {}", args.listen);
    info!("Forward address: {}", args.forward);

    // Bind socket - this is fatal if it fails
    let socket = UdpSocket::bind(args.listen)
        .await
        .context("Failed to bind UDP socket")?;

    info!("Successfully bound to {}", args.listen);

    // Set up graceful shutdown
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .context("Failed to setup SIGTERM handler")?;
    let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
        .context("Failed to setup SIGINT handler")?;

    // Main forwarding loop
    let mut buffer = [0u8; MAX_MAVLINK_PACKET_SIZE];
    loop {
        tokio::select! {
            result = socket.recv_from(&mut buffer) => {
                match result {
                    Ok((len, _src)) => {
                        let packet = &buffer[..len];

                        if let Err(e) = socket.send_to(packet, args.forward).await {
                            warn!("Failed to forward packet to {}: {}", args.forward, e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to receive packet: {}", e);
                        tokio::time::sleep(Duration::from_millis(RETRY_BACKOFF_MS)).await;
                    }
                }
            }
            _ = sigterm.recv() => {
                info!("Received SIGTERM, shutting down");
                break;
            }
            _ = sigint.recv() => {
                info!("Received SIGINT, shutting down");
                break;
            }
        }
    }

    info!("Shutdown complete");
    Ok(())
}
