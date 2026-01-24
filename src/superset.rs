//! Superset server management

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, error, warn};

use crate::python::PythonEnv;

const PID_FILE: &str = "superset.pid";

/// Superset server process manager
pub struct SupersetServer {
    root: PathBuf,
    python_env: PythonEnv,
    port: u16,
    process: Option<Child>,
    running: Arc<AtomicBool>,
}

impl SupersetServer {
    /// Create a new Superset server manager
    pub fn new(root: &Path, python_env: &PythonEnv, port: u16) -> Self {
        Self {
            root: root.to_path_buf(),
            python_env: PythonEnv::new(root).unwrap(),
            port,
            process: None,
            running: Arc::new(AtomicBool::new(false)),
        }
    }
    
    /// Start Superset server
    pub async fn start(&mut self) -> Result<()> {
        if self.running.load(Ordering::SeqCst) {
            warn!("Superset is already running");
            return Ok(());
        }
        
        let superset_home = self.root.join("superset_home");
        let logs_dir = self.root.join("logs");
        
        // Ensure directories exist
        std::fs::create_dir_all(&superset_home)?;
        std::fs::create_dir_all(&logs_dir)?;
        
        // Build command
        let mut cmd = Command::new(self.python_env.python_path());
        
        // Set environment variables
        for (key, value) in self.python_env.get_env_vars() {
            cmd.env(&key, &value);
        }
        cmd.env("PATH", self.python_env.get_path_env());
        
        // Run superset
        cmd.args([
            "-m", "superset.cli.main",
            "run",
            "-h", "127.0.0.1",
            "-p", &self.port.to_string(),
            "--with-threads",
            "--reload",  // Remove in production
        ]);
        
        cmd.current_dir(&self.root);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        info!("Starting Superset with command: {:?}", cmd);
        
        let child = cmd.spawn()
            .context("Failed to start Superset. Is it installed?")?;
        
        let pid = child.id();
        info!("Superset started with PID: {}", pid);
        
        // Save PID file
        let pid_path = self.root.join(PID_FILE);
        std::fs::write(&pid_path, pid.to_string())?;
        
        self.process = Some(child);
        self.running.store(true, Ordering::SeqCst);
        
        // Wait a bit and check if still running
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        if let Some(ref mut process) = self.process {
            match process.try_wait() {
                Ok(Some(status)) => {
                    error!("Superset exited immediately with status: {}", status);
                    self.running.store(false, Ordering::SeqCst);
                    return Err(anyhow::anyhow!("Superset failed to start"));
                }
                Ok(None) => {
                    info!("Superset is running on http://127.0.0.1:{}", self.port);
                }
                Err(e) => {
                    error!("Error checking process status: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Wait for server to finish
    pub async fn wait(&mut self) -> Result<()> {
        if let Some(ref mut process) = self.process {
            let status = process.wait()?;
            info!("Superset exited with status: {}", status);
            self.running.store(false, Ordering::SeqCst);
            
            // Clean up PID file
            let pid_path = self.root.join(PID_FILE);
            let _ = std::fs::remove_file(&pid_path);
        }
        Ok(())
    }
    
    /// Stop the running process
    pub fn stop(&mut self) -> Result<()> {
        if let Some(ref mut process) = self.process {
            info!("Stopping Superset...");
            process.kill()?;
            self.running.store(false, Ordering::SeqCst);
            
            // Clean up PID file
            let pid_path = self.root.join(PID_FILE);
            let _ = std::fs::remove_file(&pid_path);
        }
        Ok(())
    }
    
    /// Check if running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
    
    /// Stop any running Superset instance (static method)
    pub fn stop_running() -> Result<()> {
        let root = crate::get_portable_root()?;
        let pid_path = root.join(PID_FILE);
        
        if pid_path.exists() {
            let pid_str = std::fs::read_to_string(&pid_path)?;
            let pid: u32 = pid_str.trim().parse()?;
            
            info!("Found running Superset with PID: {}", pid);
            
            #[cfg(windows)]
            {
                // Kill process on Windows
                let _ = Command::new("taskkill")
                    .args(["/F", "/PID", &pid.to_string()])
                    .output();
            }
            
            std::fs::remove_file(&pid_path)?;
            info!("Superset stopped");
        } else {
            info!("No running Superset instance found");
        }
        
        Ok(())
    }
    
    /// Get status of Superset
    pub fn get_status() -> Result<String> {
        let root = crate::get_portable_root()?;
        let pid_path = root.join(PID_FILE);
        
        if pid_path.exists() {
            let pid_str = std::fs::read_to_string(&pid_path)?;
            Ok(format!("Superset is running (PID: {})", pid_str.trim()))
        } else {
            Ok("Superset is not running".to_string())
        }
    }
}

/// Initialize Superset (first-time setup)
pub async fn initialize(root: &Path, python_env: &PythonEnv, username: &str, password: &str) -> Result<()> {
    let superset_home = root.join("superset_home");
    std::fs::create_dir_all(&superset_home)?;
    
    // Create superset_config.py if not exists
    let config_path = superset_home.join("superset_config.py");
    if !config_path.exists() {
        let secret_key = generate_secret_key();
        let config_content = format!(r#"
# Superset Portable Configuration
import os

# Secret key for session signing
SECRET_KEY = "{}"

# SQLite database (portable)
SQLALCHEMY_DATABASE_URI = "sqlite:///" + os.path.join(os.path.dirname(__file__), "superset.db")

# Disable CSRF for simplicity (enable in production)
WTF_CSRF_ENABLED = False

# Disable async queries (simplifies portable setup)
SUPERSET_WEBSERVER_TIMEOUT = 300

# Disable feature flags that require Redis
FEATURE_FLAGS = {{
    "ALERT_REPORTS": False,
}}

# Simple cache (no Redis required)
CACHE_CONFIG = {{
    'CACHE_TYPE': 'SimpleCache',
    'CACHE_DEFAULT_TIMEOUT': 300,
}}
"#, secret_key);
        
        std::fs::write(&config_path, config_content)?;
        info!("Created superset_config.py");
    }
    
    info!("Running database migrations...");
    let output = python_env.run_python(&["-m", "superset", "db", "upgrade"])?;
    if !output.status.success() {
        error!("Database migration failed: {}", String::from_utf8_lossy(&output.stderr));
        return Err(anyhow::anyhow!("Database migration failed"));
    }
    
    info!("Creating admin user...");
    let output = python_env.run_python(&[
        "-m", "superset", "fab", "create-admin",
        "--username", username,
        "--password", password,
        "--firstname", "Admin",
        "--lastname", "User", 
        "--email", "admin@localhost",
    ])?;
    if !output.status.success() {
        // User might already exist, not a fatal error
        warn!("Admin creation output: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    info!("Initializing Superset...");
    let output = python_env.run_python(&["-m", "superset", "init"])?;
    if !output.status.success() {
        error!("Superset init failed: {}", String::from_utf8_lossy(&output.stderr));
        return Err(anyhow::anyhow!("Superset init failed"));
    }
    
    info!("Superset initialization complete!");
    Ok(())
}

/// Generate a random secret key
fn generate_secret_key() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("portable-superset-{:x}", timestamp)
}
