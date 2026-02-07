//! Health check module for Superset
//! 
//! Provides fast, native health checking without spawning Python processes.

use anyhow::Result;
use std::time::Duration;

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub superset_ok: bool,
    pub docs_ok: bool,
    pub superset_url: String,
    pub docs_url: String,
    pub response_time_ms: u64,
}

impl HealthStatus {
    /// Human-readable status
    pub fn summary(&self) -> String {
        let superset = if self.superset_ok { "✅" } else { "❌" };
        let docs = if self.docs_ok { "✅" } else { "❌" };
        
        format!(
            "Superset: {} ({}) | Docs: {} ({}) | Response: {}ms",
            superset, self.superset_url,
            docs, self.docs_url,
            self.response_time_ms
        )
    }
}

/// Perform a quick health check on Superset
pub async fn check_superset(port: u16) -> Result<bool> {
    let url = format!("http://127.0.0.1:{}/health", port);
    check_endpoint(&url).await
}

/// Perform a quick health check on docs server
pub async fn check_docs(port: u16) -> Result<bool> {
    let url = format!("http://127.0.0.1:{}/health", port);
    check_endpoint(&url).await
}

/// Check a specific endpoint
async fn check_endpoint(url: &str) -> Result<bool> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;
    
    match client.get(url).send().await {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false),
    }
}

/// Full health check for all services
pub async fn full_health_check(superset_port: u16, docs_port: u16) -> HealthStatus {
    let start = std::time::Instant::now();
    
    let superset_ok = check_superset(superset_port).await.unwrap_or(false);
    let docs_ok = check_docs(docs_port).await.unwrap_or(false);
    
    let response_time_ms = start.elapsed().as_millis() as u64;
    
    HealthStatus {
        superset_ok,
        docs_ok,
        superset_url: format!("http://127.0.0.1:{}", superset_port),
        docs_url: format!("http://127.0.0.1:{}", docs_port),
        response_time_ms,
    }
}

/// Print health status to console
pub async fn print_health_status(superset_port: u16, docs_port: u16) {
    let status = full_health_check(superset_port, docs_port).await;
    
    println!();
    println!("╔════════════════════════════════════════════╗");
    println!("║       Superset Portable Health Check       ║");
    println!("╠════════════════════════════════════════════╣");
    
    let superset_icon = if status.superset_ok { "✅" } else { "❌" };
    let docs_icon = if status.docs_ok { "✅" } else { "❌" };
    
    println!("║ Superset:  {} {}  ║", superset_icon, pad_right(&status.superset_url, 25));
    println!("║ Docs:      {} {}  ║", docs_icon, pad_right(&status.docs_url, 25));
    println!("║ Response:  {} ms                          ║", pad_right(&status.response_time_ms.to_string(), 4));
    println!("╚════════════════════════════════════════════╝");
    println!();
}

/// Pad string to the right
fn pad_right(s: &str, width: usize) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - s.len()))
    }
}
