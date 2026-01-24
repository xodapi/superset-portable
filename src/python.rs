//! Python environment management

use anyhow::Result;
use std::path::{Path, PathBuf};
use tracing::info;

/// Represents the portable Python environment
pub struct PythonEnv {
    root: PathBuf,
    python_exe: PathBuf,
    scripts_dir: PathBuf,
    site_packages: PathBuf,
}

impl PythonEnv {
    /// Create a new Python environment reference
    pub fn new(root: &Path) -> Result<Self> {
        let python_dir = root.join("python");
        let python_exe = python_dir.join("python.exe");
        let scripts_dir = python_dir.join("Scripts");
        let site_packages = python_dir.join("Lib").join("site-packages");
        
        Ok(Self {
            root: root.to_path_buf(),
            python_exe,
            scripts_dir,
            site_packages,
        })
    }
    
    /// Check if Python environment is valid (python.exe exists)
    pub fn is_valid(&self) -> bool {
        self.python_exe.exists()
    }
    
    /// Get path to python.exe
    pub fn python_path(&self) -> &Path {
        &self.python_exe
    }
    
    /// Get path to Scripts directory (where superset CLI is)
    pub fn scripts_path(&self) -> &Path {
        &self.scripts_dir
    }
    
    /// Get path to superset CLI executable
    pub fn superset_cli(&self) -> PathBuf {
        self.scripts_dir.join("superset.exe")
    }
    
    /// Get environment variables for running Python/Superset
    pub fn get_env_vars(&self) -> Vec<(String, String)> {
        let python_dir = self.root.join("python");
        let superset_home = self.root.join("superset_home");
        
        vec![
            // Python paths
            ("PYTHONHOME".to_string(), python_dir.to_string_lossy().to_string()),
            ("PYTHONPATH".to_string(), self.site_packages.to_string_lossy().to_string()),
            // Superset specific
            ("SUPERSET_HOME".to_string(), superset_home.to_string_lossy().to_string()),
            ("SUPERSET_CONFIG_PATH".to_string(), 
             superset_home.join("superset_config.py").to_string_lossy().to_string()),
            // Disable telemetry
            ("SUPERSET_TELEMETRY".to_string(), "false".to_string()),
            // Flask
            ("FLASK_APP".to_string(), "superset".to_string()),
            ("FLASK_ENV".to_string(), "production".to_string()),
        ]
    }
    
    /// Build PATH environment variable including Python directories
    pub fn get_path_env(&self) -> String {
        let python_dir = self.root.join("python");
        let current_path = std::env::var("PATH").unwrap_or_default();
        
        format!(
            "{};{};{}",
            python_dir.to_string_lossy(),
            self.scripts_dir.to_string_lossy(),
            current_path
        )
    }
    
    /// Run a Python command and return output
    pub fn run_python(&self, args: &[&str]) -> Result<std::process::Output> {
        let mut cmd = std::process::Command::new(&self.python_exe);
        
        // Set environment
        for (key, value) in self.get_env_vars() {
            cmd.env(&key, &value);
        }
        cmd.env("PATH", self.get_path_env());
        
        cmd.args(args);
        let output = cmd.output()?;
        Ok(output)
    }
    
    /// Check if Superset is installed
    pub fn is_superset_installed(&self) -> bool {
        self.superset_cli().exists() || {
            // Alternative: check via pip
            if let Ok(output) = self.run_python(&["-m", "pip", "show", "apache-superset"]) {
                output.status.success()
            } else {
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_python_env_paths() {
        let root = PathBuf::from("C:\\test");
        let env = PythonEnv::new(&root).unwrap();
        
        assert_eq!(env.python_path(), PathBuf::from("C:\\test\\python\\python.exe"));
        assert_eq!(env.scripts_path(), PathBuf::from("C:\\test\\python\\Scripts"));
    }
}
