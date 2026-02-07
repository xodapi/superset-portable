//! LightDocs local server for serving static site

use std::path::{Path, PathBuf};
use std::net::SocketAddr;
use anyhow::Result;
use axum::Router;
use tower_http::services::ServeDir;
use tracing::info;

/// LightDocs development server
pub struct LightDocsServer {
    root: PathBuf,
    output_dir: PathBuf,
    port: u16,
}

impl LightDocsServer {
    /// Create new server instance
    pub fn new(root: &Path, output_dir: &Path, port: u16) -> Self {
        Self {
            root: root.to_path_buf(),
            output_dir: output_dir.to_path_buf(),
            port,
        }
    }
    
    /// Start the server
    pub async fn start(&self) -> Result<()> {
        // Ensure output directory exists
        if !self.output_dir.exists() {
            std::fs::create_dir_all(&self.output_dir)?;
        }
        
        // Serve static files from output directory
        let serve_dir = ServeDir::new(&self.output_dir)
            .append_index_html_on_directories(true);
        
        let app = Router::new()
            .fallback_service(serve_dir);
        
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        info!("📚 LightDocs server at http://localhost:{}", self.port);
        
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        
        Ok(())
    }
    
    /// Start server in background
    pub fn start_background(self) -> tokio::task::JoinHandle<Result<()>> {
        tokio::spawn(async move {
            self.start().await
        })
    }
}
