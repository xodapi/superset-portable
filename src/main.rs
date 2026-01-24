//! Portable Apache Superset Launcher
//! 
//! A Rust-based launcher for running Apache Superset from a USB flash drive
//! without requiring installation or admin privileges.

mod config;
mod python;
mod superset;
mod tray;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

/// Portable Apache Superset Launcher
#[derive(Parser)]
#[command(name = "superset-launcher")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start Superset server
    Start {
        /// Port to run on (default: 8088)
        #[arg(short, long, default_value = "8088")]
        port: u16,
        
        /// Open browser after start
        #[arg(short, long, default_value = "true")]
        browser: bool,
    },
    /// Stop running Superset server
    Stop,
    /// Show server status
    Status,
    /// Initialize Superset (first-time setup)
    Init {
        /// Admin username
        #[arg(short, long, default_value = "admin")]
        username: String,
        
        /// Admin password
        #[arg(short, long, default_value = "admin")]
        password: String,
    },
    /// Run with system tray GUI
    Tray,
}

/// Get the portable root directory (where the exe is located)
fn get_portable_root() -> Result<PathBuf> {
    let exe_path = std::env::current_exe()?;
    let root = exe_path.parent()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine executable directory"))?;
    Ok(root.to_path_buf())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .compact()
        .init();
    
    let cli = Cli::parse();
    let root = get_portable_root()?;
    
    info!("Portable Superset Launcher");
    info!("Root directory: {}", root.display());
    
    // Load or create config
    let mut config = config::Config::load_or_create(&root)?;
    
    // Validate Python environment
    let python_env = python::PythonEnv::new(&root)?;
    if !python_env.is_valid() {
        error!("Python environment not found at: {}", python_env.python_path().display());
        error!("Please run setup scripts first or ensure python/ directory exists");
        std::process::exit(1);
    }
    
    match cli.command {
        Some(Commands::Start { port, browser }) => {
            info!("Starting Superset on port {}...", port);
            config.port = port;
            config.open_browser = browser;
            config.save(&root)?;
            
            let mut server = superset::SupersetServer::new(&root, &python_env, port);
            server.start().await?;
            
            if browser {
                let url = format!("http://localhost:{}", port);
                info!("Opening browser: {}", url);
                open::that(&url)?;
            }
            
            info!("Superset is running. Press Ctrl+C to stop.");
            server.wait().await?;
        }
        Some(Commands::Stop) => {
            info!("Stopping Superset...");
            superset::SupersetServer::stop_running()?;
            info!("Superset stopped.");
        }
        Some(Commands::Status) => {
            let status = superset::SupersetServer::get_status()?;
            println!("{}", status);
        }
        Some(Commands::Init { username, password }) => {
            info!("Initializing Superset...");
            superset::initialize(&root, &python_env, &username, &password).await?;
            info!("Superset initialized successfully!");
            info!("You can now run: superset-launcher start");
        }
        Some(Commands::Tray) => {
            info!("Starting with system tray...");
            tray::run_tray(&root, &python_env, &config).await?;
        }
        None => {
            // Default: start with tray
            info!("Starting with system tray (default mode)...");
            tray::run_tray(&root, &python_env, &config).await?;
        }
    }
    
    Ok(())
}
