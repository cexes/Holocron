mod app;
mod config;
mod error;
mod headless;
mod ipc;
mod mcp;
mod terminal;
mod tui;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Parser)]
#[command(
    name = "holocron",
    version,
    about = "Cross-platform TUI terminal multiplexer with Claude MCP integration"
)]
struct Cli {
    /// Run as MCP bridge (stdio → IPC → TUI). Used by Claude Code.
    #[arg(long)]
    mcp: bool,

    /// Run the terminal manager + IPC server with no TUI attached.
    /// Auto-spawned by `--mcp` when no session is already running.
    #[arg(long)]
    headless: bool,

    /// Path to config file (default: ~/.config/holocron/config.toml)
    #[arg(long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Session name to attach to (for --mcp mode)
    #[arg(long, value_name = "SESSION")]
    session: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = config::Config::load()?;

    setup_logging()?;
    info!("holocron starting (mcp={})", cli.mcp);

    if cli.mcp {
        mcp::run_bridge(cli.session).await
    } else if cli.headless {
        headless::run(config).await
    } else {
        tui::run(config).await
    }
}

fn setup_logging() -> Result<()> {
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("holocron");

    std::fs::create_dir_all(&log_dir)?;

    let file_appender = tracing_appender::rolling::daily(&log_dir, "holocron.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Leak the guard so the log worker lives for the full process lifetime
    Box::leak(Box::new(guard));

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    Ok(())
}
