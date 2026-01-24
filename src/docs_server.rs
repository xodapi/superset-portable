//! Fast static file server for documentation
//! 
//! Replaces Python's http.server with a native Rust implementation
//! for 10-20x faster serving of static files.

use anyhow::Result;
use axum::{
    Router,
    routing::get,
    response::{IntoResponse, Response},
    http::{StatusCode, header, HeaderValue},
    body::Body,
};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use std::path::{Path, PathBuf};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::oneshot;
use tracing::{info, error};

/// Documentation server that serves static files
pub struct DocsServer {
    docs_path: PathBuf,
    port: u16,
    running: Arc<AtomicBool>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl DocsServer {
    /// Create a new docs server
    pub fn new(root: &Path, port: u16) -> Self {
        Self {
            docs_path: root.join("docs"),
            port,
            running: Arc::new(AtomicBool::new(false)),
            shutdown_tx: None,
        }
    }
    
    /// Start the documentation server
    pub async fn start(&mut self) -> Result<()> {
        if self.running.load(Ordering::SeqCst) {
            info!("Docs server already running");
            return Ok(());
        }
        
        if !self.docs_path.exists() {
            error!("Docs directory not found: {}", self.docs_path.display());
            return Err(anyhow::anyhow!("Docs directory not found"));
        }
        
        let docs_path = self.docs_path.clone();
        let port = self.port;
        let running = self.running.clone();
        
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        self.shutdown_tx = Some(shutdown_tx);
        
        // Build the router with static file serving and UTF-8 headers
        let serve_dir = ServeDir::new(&docs_path)
            .append_index_html_on_directories(true);
        
        let app = Router::new()
            .route("/health", get(health_handler))
            .nest_service("/", serve_dir)
            .layer(SetResponseHeaderLayer::overriding(
                header::CONTENT_TYPE,
                |response: &Response<Body>| {
                    // Check if it's a text file that needs UTF-8
                    if let Some(ct) = response.headers().get(header::CONTENT_TYPE) {
                        if let Ok(ct_str) = ct.to_str() {
                            // Add charset=utf-8 for text files
                            if ct_str.starts_with("text/") && !ct_str.contains("charset") {
                                return Some(HeaderValue::from_str(&format!("{}; charset=utf-8", ct_str)).ok()?);
                            }
                            // Fix markdown files
                            if ct_str.contains("markdown") || ct_str.contains("octet-stream") {
                                return Some(HeaderValue::from_static("text/markdown; charset=utf-8"));
                            }
                        }
                    }
                    None
                },
            ));
        
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        
        running.store(true, Ordering::SeqCst);
        info!("📚 Docs server starting on http://127.0.0.1:{}", port);
        
        // Spawn the server in a background task
        tokio::spawn(async move {
            let listener = match tokio::net::TcpListener::bind(addr).await {
                Ok(l) => l,
                Err(e) => {
                    error!("Failed to bind docs server: {}", e);
                    running.store(false, Ordering::SeqCst);
                    return;
                }
            };
            
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .ok();
            
            running.store(false, Ordering::SeqCst);
            info!("Docs server stopped");
        });
        
        Ok(())
    }
    
    /// Stop the docs server
    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
            self.running.store(false, Ordering::SeqCst);
            info!("Docs server shutdown requested");
        }
    }
    
    /// Check if running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

/// Health check handler
async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Default port for docs server
pub const DOCS_DEFAULT_PORT: u16 = 8089;

