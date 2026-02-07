//! System tray integration for Portable Superset Launcher

use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tray_item::{IconSource, TrayItem};
use tracing::{info, error};

use crate::config::Config;
use crate::python::PythonEnv;
use crate::superset::SupersetServer;
use crate::gateway;

/// Run the application with system tray
pub async fn run_tray(root: &Path, python_env: &PythonEnv, config: &Config) -> Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    // Create tray item
    let mut tray = TrayItem::new("Superset Launcher", IconSource::Resource("icon"))?;
    
    // Add menu items
    let port = config.port;
    tray.add_label("Portable Superset")?;
    tray.add_menu_item("Open Dashboard", move || {
        let url = format!("http://localhost:{}", port);
        let _ = open::that(&url);
    })?;
    
    tray.add_menu_item("Start Server", || {
        info!("Start requested from tray");
        // TODO: Implement proper async communication
    })?;
    
    tray.add_menu_item("Stop Server", || {
        info!("Stop requested from tray");
        let _ = SupersetServer::stop_running();
    })?;
    
    let r2 = running.clone();
    tray.add_menu_item("Exit", move || {
        info!("Exit requested from tray");
        let _ = SupersetServer::stop_running();
        r2.store(false, Ordering::SeqCst);
    })?;
    
    // Start Superset automatically
    let mut server = SupersetServer::new(root, python_env, config.port);
    
    // Check if Superset is installed
    if !python_env.is_superset_installed() {
        error!("Superset is not installed!");
        error!("Please run setup/install_superset.bat first");
        return Err(anyhow::anyhow!("Superset not installed"));
    }
    
    server.start().await?;
    
    // Open browser if configured
    if config.open_browser {
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        let url = format!("http://localhost:{}", config.port);
        let _ = open::that(&url);
    }
    
    info!("Superset Launcher running in system tray");
    info!("Right-click the tray icon for options");
    
    // Keep running until exit
    while running.load(Ordering::SeqCst) {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    // Cleanup
    server.stop()?;
    
    Ok(())
}
