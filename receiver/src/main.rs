use anyhow::{Context, Result};
use clap::Parser;
use common::{mavlink_utils, tlog, MAX_MAVLINK_PACKET_SIZE};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs::{self, File};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::UdpSocket;
use tracing::{error, info, warn};

const DEFAULT_MAX_FILE_SIZE_MB: u64 = 100;
const FLUSH_INTERVAL_PACKETS: usize = 100;
const RETRY_BACKOFF_MS: u64 = 1000;

#[derive(Parser, Debug)]
#[command(author, version, about = "MAVLink telemetry receiver and tlog writer")]
struct Args {
    /// Address to listen on for MAVLink packets
    #[arg(short, long, default_value = "0.0.0.0:14550")]
    listen: SocketAddr,

    /// Output directory for tlog files
    #[arg(short, long, default_value = "./logs")]
    output: PathBuf,

    /// Maximum file size in MB before rotation
    #[arg(short, long, default_value_t = DEFAULT_MAX_FILE_SIZE_MB)]
    max_file_size: u64,
}

struct TlogWriter {
    current_file: BufWriter<File>,
    bytes_written: u64,
    max_file_size: u64,
    output_dir: PathBuf,
    packets_since_flush: usize,
}

impl TlogWriter {
    async fn new(output_dir: PathBuf, max_file_size_mb: u64) -> Result<Self> {
        // Create output directory if it doesn't exist
        fs::create_dir_all(&output_dir)
            .await
            .context("Failed to create output directory")?;

        let max_file_size = max_file_size_mb * 1024 * 1024;
        let file = Self::create_new_file(&output_dir).await?;

        Ok(Self {
            current_file: BufWriter::new(file),
            bytes_written: 0,
            max_file_size,
            output_dir,
            packets_since_flush: 0,
        })
    }

    async fn create_new_file(output_dir: &Path) -> Result<File> {
        let now = chrono::Local::now();
        let filename = format!(
            "telemetry_{}_{:03}.tlog",
            now.format("%Y%m%d_%H%M%S"),
            now.timestamp_subsec_millis()
        );
        let path = output_dir.join(&filename);

        let file = File::create(&path)
            .await
            .context("Failed to create tlog file")?;

        info!("Created new tlog file: {}", path.display());
        Ok(file)
    }

    async fn write_packet(&mut self, packet: &[u8]) -> Result<()> {
        // Format packet with timestamp
        let entry = tlog::format_tlog_entry(packet);

        // Write to file
        self.current_file
            .write_all(&entry)
            .await
            .context("Failed to write to tlog file")?;

        self.bytes_written += entry.len() as u64;
        self.packets_since_flush += 1;

        // Periodic flush to ensure data is written
        if self.packets_since_flush >= FLUSH_INTERVAL_PACKETS {
            self.current_file
                .flush()
                .await
                .context("Failed to flush tlog file")?;
            self.packets_since_flush = 0;
        }

        // Check if rotation is needed
        self.check_rotation().await?;

        Ok(())
    }

    async fn check_rotation(&mut self) -> Result<()> {
        if self.bytes_written >= self.max_file_size {
            self.rotate_file().await?;
        }
        Ok(())
    }

    async fn rotate_file(&mut self) -> Result<()> {
        info!(
            "Rotating tlog file (size: {} bytes)",
            self.bytes_written
        );

        // Flush and close current file
        self.current_file
            .flush()
            .await
            .context("Failed to flush before rotation")?;

        // Create new file
        let file = Self::create_new_file(&self.output_dir).await?;
        self.current_file = BufWriter::new(file);
        self.bytes_written = 0;
        self.packets_since_flush = 0;

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down tlog writer");
        self.current_file
            .flush()
            .await
            .context("Failed to flush on shutdown")?;
        Ok(())
    }
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

    info!("MAVLink Receiver starting");
    info!("Listen address: {}", args.listen);
    info!("Output directory: {}", args.output.display());
    info!("Max file size: {} MB", args.max_file_size);

    // Bind socket
    let socket = UdpSocket::bind(args.listen)
        .await
        .context("Failed to bind UDP socket")?;

    info!("Successfully bound to {}", args.listen);

    // Initialize tlog writer
    let mut writer = TlogWriter::new(args.output, args.max_file_size)
        .await
        .context("Failed to initialize tlog writer")?;

    // Set up graceful shutdown
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .context("Failed to setup SIGTERM handler")?;
    let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
        .context("Failed to setup SIGINT handler")?;

    // Main receive loop
    let mut buffer = [0u8; MAX_MAVLINK_PACKET_SIZE];
    loop {
        tokio::select! {
            result = socket.recv_from(&mut buffer) => {
                match result {
                    Ok((len, _src)) => {
                        let packet = &buffer[..len];

                        // Validate MAVLink packet
                        if !mavlink_utils::is_valid_mavlink(packet) {
                            warn!("Received invalid MAVLink packet (validation failed)");
                            continue;
                        }

                        // Write to tlog
                        if let Err(e) = writer.write_packet(packet).await {
                            error!("Failed to write packet to tlog: {}", e);
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

    // Graceful shutdown
    writer.shutdown().await?;
    info!("Shutdown complete");

    Ok(())
}
