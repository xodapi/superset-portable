//! Portable Apache Superset Launcher
//! 
//! A Rust-based launcher for running Apache Superset from a USB flash drive
//! without requiring installation or admin privileges.

mod config;
mod cache;
mod demo_data;
mod docs_server;
mod gateway;
mod health_check;
mod launcher_ui;
mod lightdocs;
mod packer;
mod python;
mod superset;
mod tray;
mod validator;
mod data_loader;
mod watcher;

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
        
        /// Also start docs server
        #[arg(short, long, default_value = "true")]
        docs: bool,
    },
    /// Stop running Superset server
    Stop,
    /// Show server status and health check
    Status,
    /// Fast health check (no Python needed)
    Health,
    /// Start documentation server only
    Docs {
        /// Port for docs server (default: 8089)
        #[arg(short, long, default_value = "8089")]
        port: u16,
    },
    /// Initialize Superset (first-time setup)
    Init {
        /// Admin username
        #[arg(short, long, default_value = "admin")]
        username: String,
        
        /// Admin password
        #[arg(short, long, default_value = "admin")]
        password: String,
    },
    /// Pack release for distribution
    Pack {
        /// Use zstd compression (faster) instead of ZIP
        #[arg(short, long)]
        zstd: bool,
    },
    /// Run with system tray GUI
    Tray,
    /// Validate environment
    Validate,
    /// Import RZD demo data into examples.db
    ImportDemo,
    /// Manage cache (stats, clear)
    Cache {
        #[command(subcommand)]
        action: CacheAction,
    },
    /// LightDocs - Knowledge Base commands
    Lightdocs {
        #[command(subcommand)]
        action: LightDocsAction,
    },
    /// Start unified launcher UI (web interface)
    Launcher {
        /// Port for launcher UI (default: 3000)
        #[arg(short, long, default_value = "3000")]
        port: u16,
        /// Superset port (default: 8088)
        #[arg(long, default_value = "8088")]
        superset_port: u16,
        /// LightDocs port (default: 3030)
        #[arg(long, default_value = "3030")]
        lightdocs_port: u16,
    },
    /// High-performance data loader (Excel/CSV)
    LoadData {
        /// Path to input file
        file: PathBuf,
        /// Target table name (optional, defaults to filename)
        #[arg(short, long)]
        table: Option<String>,
        /// Database path (optional, defaults to examples.db)
        #[arg(short, long)]
        db: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum CacheAction {
    /// Show cache statistics
    Stats,
    /// Clear all cached data
    Clear,
    /// Test cache operations
    Test,
}

#[derive(Subcommand)]
enum LightDocsAction {
    /// Initialize LightDocs in current directory
    Init,
    /// Build static site from markdown files
    Build,
    /// Start development server with live reload
    Serve {
        /// Port for server (default: 8090)
        #[arg(short, long, default_value = "8090")]
        port: u16,
        /// Open browser after start
        #[arg(short, long, default_value = "true")]
        browser: bool,
    },
    /// Search documents
    Search {
        /// Search query
        query: String,
    },
}

/// Get the portable root directory (where the exe is located)
fn get_portable_root() -> Result<PathBuf> {
    let exe_path = std::env::current_exe()?;
    let root = exe_path.parent()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine executable directory"))?;
        
    // Check if we are running in development (cargo run)
    // The executable is in target/debug, but assets are in project root
    if !root.join("python").exists() {
        let cwd = std::env::current_dir()?;
        if cwd.join("python").exists() || cwd.join("Cargo.toml").exists() {
            info!("Development mode detected, using CWD as root: {}", cwd.display());
            return Ok(cwd);
        }
    }
    
    Ok(root.to_path_buf())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let _subscriber = FmtSubscriber::builder()
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
    
    match cli.command {
        Some(Commands::Start { port, browser, docs }) => {
            if !python_env.is_valid() {
                error!("Python environment not found at: {}", python_env.python_path().display());
                std::process::exit(1);
            }
            info!("Starting Superset on port {}...", port);
            config.port = port;
            config.open_browser = browser;
            config.save(&root)?;
            
            // Start docs server if requested
            if docs {
                let mut docs_server = docs_server::DocsServer::new(&root, docs_server::DOCS_DEFAULT_PORT);
                docs_server.start().await?;
            }
            
            let mut server = superset::SupersetServer::new(&root, &python_env, port);
            server.start().await?;
            
            if browser {
                let url = format!("http://localhost:{}", port);
                info!("Opening browser: {}", url);
                let _ = open::that(&url);
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
            // Also show health check
            health_check::print_health_status(config.port, docs_server::DOCS_DEFAULT_PORT).await;
        }
        Some(Commands::Health) => {
            // Fast health check - no Python needed
            health_check::print_health_status(config.port, docs_server::DOCS_DEFAULT_PORT).await;
        }
        Some(Commands::Docs { port }) => {
            info!("Starting documentation server on port {}...", port);
            let mut docs_server = docs_server::DocsServer::new(&root, port);
            docs_server.start().await?;
            
            let url = format!("http://localhost:{}", port);
            info!("📚 Documentation available at: {}", url);
            open::that(&url)?;
            
            // Keep running
            info!("Press Ctrl+C to stop.");
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
        Some(Commands::Init { username, password }) => {
            if !python_env.is_valid() {
                error!("Python environment not found at: {}", python_env.python_path().display());
                std::process::exit(1);
            }
            info!("Initializing Superset...");
            superset::initialize(&root, &python_env, &username, &password).await?;
            info!("Superset initialized successfully!");
            info!("You can now run: superset-launcher start");
        }
        Some(Commands::Pack { zstd }) => {
            info!("📦 Packing release for distribution...");
            let packer = packer::ReleasePacker::new(&root);
            
            if zstd {
                info!("Using Zstd compression (faster)");
                packer.pack_zstd()?;
            } else {
                info!("Using ZIP compression (more compatible)");
                packer.pack_zip()?;
            }
        }
        Some(Commands::Tray) => {
            info!("Starting with system tray...");
            tray::run_tray(&root, &python_env, &config).await?;
        }
        Some(Commands::Validate) => {
            info!("Validating environment...");
            let validator = validator::Validator::new(&root);
            let results = validator.validate_all();
            validator::print_validation_report(&results);
        }
        Some(Commands::ImportDemo) => {
            info!("Importing RZD demo data...");
            demo_data::import_demo_data(&root)?;
        }
        Some(Commands::Cache { action }) => {
            match action {
                CacheAction::Stats => {
                    info!("📊 Cache statistics:");
                    let cache_result = cache::Cache::open(&root);
                    match cache_result {
                        Ok(cache) => {
                            let stats = cache.stats();
                            println!("{}", stats);
                        }
                        Err(e) => {
                            println!("Cache not initialized: {}", e);
                        }
                    }
                }
                CacheAction::Clear => {
                    info!("🗑️ Clearing cache...");
                    let cache = cache::Cache::open(&root)?;
                    cache.clear()?;
                    println!("✅ Cache cleared successfully!");
                }
                CacheAction::Test => {
                    info!("🧪 Testing cache...");
                    let cache = cache::Cache::open(&root)?;
                    
                    // Test write
                    cache.set_string("test_key", "Привет, мир!")?;
                    println!("✅ Write test passed");
                    
                    // Test read
                    if let Some(value) = cache.get_string("test_key") {
                        println!("✅ Read test passed: {}", value);
                    } else {
                        println!("❌ Read test failed");
                    }
                    
                    // Clean up
                    cache.remove("test_key")?;
                    println!("✅ Delete test passed");
                    
                    let stats = cache.stats();
                    println!("\n{}", stats);
                }
            }
        }
        Some(Commands::Lightdocs { action }) => {
            match action {
                LightDocsAction::Init => {
                    info!("📚 Initializing LightDocs...");
                    let lightdocs = lightdocs::LightDocs::new(&root)?;
                    lightdocs.init()?;
                    info!("✅ LightDocs initialized!");
                    info!("📁 Documents folder: {}", root.join("knowledge").display());
                    info!("🚀 Run: superset-launcher lightdocs serve");
                }
                LightDocsAction::Build => {
                    info!("🔨 Building static site...");
                    let lightdocs = lightdocs::LightDocs::new(&root)?;
                    let docs = lightdocs.build()?;
                    let public_count = docs.iter()
                        .filter(|d| d.status == lightdocs::DocumentStatus::Public)
                        .count();
                    info!("✅ Built {} public documents (of {} total)", public_count, docs.len());
                }
                LightDocsAction::Serve { port, browser } => {
                    info!("📚 Starting LightDocs server...");
                    
                    // Build first
                    let lightdocs = lightdocs::LightDocs::new(&root)?;
                    let config = lightdocs::LightDocsConfig::load(&root)?;
                    lightdocs.build()?;
                    
                    // Index documents for search
                    let search_index = lightdocs::search::SearchIndex::open(&root)?;
                    for doc in lightdocs.list_documents()? {
                        search_index.index_document(&doc.slug(), &doc.title, &doc.content)?;
                    }
                    
                    // Start watcher in background
                    if config.live_reload {
                        let watcher_root = root.clone();
                        std::thread::spawn(move || {
                            if let Ok(lightdocs) = lightdocs::LightDocs::new(&watcher_root) {
                                if let Err(e) = lightdocs.watch() {
                                    tracing::error!("Watcher error: {}", e);
                                }
                            }
                        });
                    }
                    
                    // Start server
                    let output_dir = config.output_dir_abs(&root);
                    let server = lightdocs::LightDocsServer::new(&root, &output_dir, port);
                    
                    if browser {
                        let url = format!("http://localhost:{}", port);
                        info!("🌐 Opening: {}", url);
                        let _ = open::that(&url);
                    }
                    
                    info!("Press Ctrl+C to stop.");
                    server.start().await?;
                }
                LightDocsAction::Search { query } => {
                    info!("🔍 Searching: {}", query);
                    let search_index = lightdocs::search::SearchIndex::open(&root)?;
                    let results = search_index.search(&query)?;
                    
                    if results.is_empty() {
                        println!("Ничего не найдено.");
                    } else {
                        println!("\n📚 Результаты ({}):\n", results.len());
                        for (i, entry) in results.iter().enumerate() {
                            println!("{}. {} ({})", i + 1, entry.title, entry.slug);
                            println!("   {}\n", entry.excerpt);
                        }
                    }
                }
            }
        }
        Some(Commands::Launcher { port, superset_port, lightdocs_port }) => {
            info!("🚀 Starting unified launcher UI...");
            
            // Start Data Watcher
            let watcher = std::sync::Arc::new(watcher::DataWatcher::new(&root));
            watcher.start().await;
            
            let launcher = launcher_ui::LauncherUI::new(&root, port, superset_port, lightdocs_port, watcher);
            
            let url = format!("http://localhost:{}", port);
            info!("🌐 Opening: {}", url);
            let _ = open::that(&url);
            
            launcher.start().await?;
        }
        Some(Commands::LoadData { file, table, db }) => {
            let table_name = table.unwrap_or_else(|| {
                file.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            });
            
            let db_path = db.unwrap_or_else(|| root.join("examples.db"));
            
            match data_loader::load_file(&file, &table_name, &db_path) {
                Ok(msg) => info!("{}", msg),
                Err(e) => error!("Failed to load data: {}", e),
            }
        }
        None => {
            // Default: start with launcher UI
            info!("🚀 Starting unified launcher UI (default mode)...");
            
            // Start Data Watcher
            let watcher = std::sync::Arc::new(watcher::DataWatcher::new(&root));
            watcher.start().await;
            
            let launcher = launcher_ui::LauncherUI::new(&root, 3000, 8088, 3030, watcher);
            
            let url = "http://localhost:3000";
            info!("🌐 Opening: {}", url);
            let _ = open::that(url);
            
            launcher.start().await?;
        }
    }
    
    Ok(())
}
