//! Unified Web Launcher UI
//!
//! Web interface for managing Superset and LightDocs services.
//! Runs on port 3000 by default.

use anyhow::Result;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{info, error};

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

/// Default port for launcher UI
pub const LAUNCHER_PORT: u16 = 3000;

/// Service status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Running,
    Stopped,
    Starting,
    Stopping,
    Error,
}

/// Status of all services
#[derive(Debug, Clone, Serialize)]
pub struct SystemStatus {
    pub superset: ServiceInfo,
    pub lightdocs: ServiceInfo,
    pub uptime_seconds: u64,
}

/// Individual service info
#[derive(Debug, Clone, Serialize)]
pub struct ServiceInfo {
    pub status: ServiceStatus,
    pub port: u16,
    pub url: String,
}

/// Shared application state
pub struct AppState {
    pub root: PathBuf,
    pub start_time: std::time::Instant,
    pub superset_status: RwLock<ServiceStatus>,
    pub lightdocs_status: RwLock<ServiceStatus>,
    pub superset_port: u16,
    pub lightdocs_port: u16,
    pub shutdown_tx: mpsc::Sender<()>,
}

impl AppState {
    pub fn new(root: &PathBuf, superset_port: u16, lightdocs_port: u16, shutdown_tx: mpsc::Sender<()>) -> Self {
        Self {
            root: root.clone(),
            start_time: std::time::Instant::now(),
            superset_status: RwLock::new(ServiceStatus::Stopped),
            lightdocs_status: RwLock::new(ServiceStatus::Stopped),
            superset_port,
            lightdocs_port,
            shutdown_tx,
        }
    }
}

/// Launcher UI server
pub struct LauncherUI {
    root: PathBuf,
    port: u16,
    superset_port: u16,
    lightdocs_port: u16,
}

impl LauncherUI {
    pub fn new(root: &PathBuf, port: u16, superset_port: u16, lightdocs_port: u16) -> Self {
        Self {
            root: root.clone(),
            port,
            superset_port,
            lightdocs_port,
        }
    }

    /// Start the launcher UI server
    pub async fn start(&self) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(1);
        let state = Arc::new(AppState::new(&self.root, self.superset_port, self.lightdocs_port, tx));
        
        let app = Router::new()
            .route("/", get(index_handler))
            .route("/api/status", get(status_handler))
            .route("/api/superset/start", post(superset_start_handler))
            .route("/api/superset/stop", post(superset_stop_handler))
            .route("/api/lightdocs/start", post(lightdocs_start_handler))
            .route("/api/lightdocs/stop", post(lightdocs_stop_handler))
            .route("/api/lightdocs/search", get(search_handler))
            .route("/api/shutdown", post(shutdown_handler))
            .with_state(state);

        let addr = format!("127.0.0.1:{}", self.port);
        info!("🚀 Launcher UI starting at http://{}", addr);
        
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = rx.recv().await;
                info!("Shutdown signal received");
            })
            .await?;
            
        // Cleanup on exit
        info!("Cleaning up services...");
        let _ = kill_process_on_port(self.superset_port).await;
        let _ = kill_process_on_port(self.lightdocs_port).await;
        
        Ok(())
    }
}

// Handler: Main HTML page
async fn index_handler() -> Html<&'static str> {
    Html(LAUNCHER_HTML)
}

// Handler: Get system status
async fn status_handler(
    State(state): State<Arc<AppState>>,
) -> Json<SystemStatus> {
    let superset_status = state.superset_status.read().await.clone();
    let lightdocs_status = state.lightdocs_status.read().await.clone();
    
    // Check actual port availability
    let superset_running = check_port(state.superset_port).await;
    let lightdocs_running = check_port(state.lightdocs_port).await;
    
    Json(SystemStatus {
        superset: ServiceInfo {
            status: if superset_running { ServiceStatus::Running } else { superset_status },
            port: state.superset_port,
            url: format!("http://localhost:{}", state.superset_port),
        },
        lightdocs: ServiceInfo {
            status: if lightdocs_running { ServiceStatus::Running } else { lightdocs_status },
            port: state.lightdocs_port,
            url: format!("http://localhost:{}", state.lightdocs_port),
        },
        uptime_seconds: state.start_time.elapsed().as_secs(),
    })
}

// Handler: Start Superset
async fn superset_start_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!("Starting Superset...");
    
    {
        let mut status = state.superset_status.write().await;
        *status = ServiceStatus::Starting;
    }
    
    // Spawn Superset process
    let root = state.root.clone();
    let port = state.superset_port;
    
    tokio::spawn(async move {
        // Prepare paths
        let logs_dir = root.join("logs");
        let _ = std::fs::create_dir_all(&logs_dir);
        let stdout_file = std::fs::File::create(logs_dir.join("superset.stdout.log")).unwrap_or_else(|_| 
            std::fs::File::create("superset.stdout.log").unwrap() // Fallback
        );
        let stderr_file = std::fs::File::create(logs_dir.join("superset.stderr.log")).unwrap_or_else(|_| 
            std::fs::File::create("superset.stderr.log").unwrap() // Fallback
        );
        
        let python_env = crate::python::PythonEnv::new(&root).unwrap();
        let python_path = python_env.python_path();
        
        // Build command with correct environment from PythonEnv
        let mut cmd = tokio::process::Command::new(python_path);
        
        cmd.args([
            "-m", "flask",
            "--app", "superset.app:create_app()",
            "run",
            "--host", "127.0.0.1",
            "--port", &port.to_string(),
        ]);
        
        cmd.current_dir(&root);
        
        // Apply all environment variables from PythonEnv (includes PYTHONHOME, PATH)
        for (key, val) in python_env.get_env_vars() {
            cmd.env(key, val);
        }
        cmd.env("PATH", python_env.get_path_env());
        
        // Redirect output
        cmd.stdout(std::process::Stdio::from(stdout_file));
        cmd.stderr(std::process::Stdio::from(stderr_file));
            
        match cmd.spawn() {
            Ok(_) => info!("Superset process started via UI"),
            Err(e) => error!("Failed to start Superset: {}", e),
        }
    });
    
    Json(serde_json::json!({"status": "starting", "port": state.superset_port}))
}

// Handler: Stop Superset
async fn superset_stop_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!("Stopping Superset...");
    
    {
        let mut status = state.superset_status.write().await;
        *status = ServiceStatus::Stopping;
    }
    
    // Kill process on port
    let port = state.superset_port;
    let _ = kill_process_on_port(port).await;
    
    {
        let mut status = state.superset_status.write().await;
        *status = ServiceStatus::Stopped;
    }
    
    Json(serde_json::json!({"status": "stopped"}))
}

// Handler: Start LightDocs
async fn lightdocs_start_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!("Starting LightDocs...");
    
    {
        let mut status = state.lightdocs_status.write().await;
        *status = ServiceStatus::Starting;
    }
    
    let root = state.root.clone();
    let port = state.lightdocs_port;
    
    tokio::spawn(async move {
        // Build and serve LightDocs
        if let Ok(lightdocs) = crate::lightdocs::LightDocs::new(&root) {
            let _ = lightdocs.build();
            
            if let Ok(config) = crate::lightdocs::LightDocsConfig::load(&root) {
                let output_dir = config.output_dir_abs(&root);
                let server = crate::lightdocs::LightDocsServer::new(&root, &output_dir, port);
                let _ = server.start().await;
            }
        }
    });
    
    {
        let mut status = state.lightdocs_status.write().await;
        *status = ServiceStatus::Running;
    }
    
    Json(serde_json::json!({"status": "starting", "port": state.lightdocs_port}))
}

// Handler: Stop LightDocs
async fn lightdocs_stop_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!("Stopping LightDocs...");
    
    let port = state.lightdocs_port;
    let _ = kill_process_on_port(port).await;
    
    {
        let mut status = state.lightdocs_status.write().await;
        *status = ServiceStatus::Stopped;
    }
    
    Json(serde_json::json!({"status": "stopped"}))
}

// Handler: Shutdown entire launcher
async fn shutdown_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!("Shutdown requested via API");
    
    // Send shutdown signal
    let _ = state.shutdown_tx.send(()).await;
    
    Json(serde_json::json!({"status": "shutting_down"}))
}

// Handler: Search LightDocs
async fn search_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<SearchQuery>,
) -> impl IntoResponse {
    let index_res = crate::lightdocs::search::SearchIndex::open(&state.root);
    match index_res {
        Ok(index) => {
            match index.search(&params.q) {
                Ok(results) => Json(serde_json::to_value(results).unwrap()),
                Err(e) => Json(serde_json::json!({"error": e.to_string()})),
            }
        },
        Err(e) => Json(serde_json::json!({"error": e.to_string()})),
    }
}

/// Check if a port is in use
async fn check_port(port: u16) -> bool {
    tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port))
        .await
        .is_ok()
}

/// Kill process on port (Windows)
async fn kill_process_on_port(port: u16) -> Result<()> {
    #[cfg(windows)]
    {
        let output = tokio::process::Command::new("cmd")
            .args(["/C", &format!("for /f \"tokens=5\" %a in ('netstat -ano ^| findstr :{} ^| findstr LISTENING') do taskkill /PID %a /F", port)])
            .output()
            .await?;
        
        if !output.status.success() {
            // Try alternative method
            let _ = tokio::process::Command::new("powershell")
                .args(["-Command", &format!(
                    "Get-NetTCPConnection -LocalPort {} -ErrorAction SilentlyContinue | ForEach-Object {{ Stop-Process -Id $_.OwningProcess -Force -ErrorAction SilentlyContinue }}",
                    port
                )])
                .output()
                .await;
        }
    }
    
    Ok(())
}

/// Embedded HTML for launcher UI
const LAUNCHER_HTML: &str = r#"<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Apache Superset Portable</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            min-height: 100vh;
            color: #e0e0e0;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        
        .container {
            max-width: 800px;
            width: 100%;
            padding: 20px;
        }
        
        .header {
            text-align: center;
            margin-bottom: 40px;
        }
        
        .header h1 {
            font-size: 2rem;
            color: #fff;
            margin-bottom: 8px;
        }
        
        .header .subtitle {
            color: #888;
            font-size: 0.9rem;
        }
        
        .services {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 24px;
            margin-bottom: 32px;
        }
        
        .service-card {
            background: rgba(255,255,255,0.05);
            border-radius: 16px;
            padding: 24px;
            border: 1px solid rgba(255,255,255,0.1);
            transition: all 0.3s ease;
        }
        
        .service-card:hover {
            transform: translateY(-2px);
            border-color: rgba(255,255,255,0.2);
        }
        
        .service-header {
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 16px;
        }
        
        .service-name {
            font-size: 1.25rem;
            font-weight: 600;
            color: #fff;
        }
        
        .status-badge {
            padding: 4px 12px;
            border-radius: 20px;
            font-size: 0.75rem;
            font-weight: 500;
            text-transform: uppercase;
        }
        
        .status-running { background: #10b981; color: #fff; }
        .status-stopped { background: #6b7280; color: #fff; }
        .status-starting { background: #f59e0b; color: #000; }
        .status-error { background: #ef4444; color: #fff; }
        
        .service-port {
            color: #888;
            font-size: 0.85rem;
            margin-bottom: 20px;
        }
        
        .btn-group {
            display: flex;
            gap: 12px;
        }
        
        .btn {
            flex: 1;
            padding: 12px 16px;
            border: none;
            border-radius: 8px;
            font-size: 0.9rem;
            font-weight: 500;
            cursor: pointer;
            transition: all 0.2s ease;
        }
        
        .btn-primary {
            background: #3b82f6;
            color: #fff;
        }
        
        .btn-primary:hover { background: #2563eb; }
        
        .btn-secondary {
            background: rgba(255,255,255,0.1);
            color: #fff;
        }
        
        .btn-secondary:hover { background: rgba(255,255,255,0.15); }
        
        .btn-danger {
            background: #ef4444;
            color: #fff;
        }
        
        .btn-danger:hover { background: #dc2626; }
        
        .btn:disabled {
            opacity: 0.5;
            cursor: not-allowed;
        }
        
        .footer {
            text-align: center;
            color: #666;
            font-size: 0.8rem;
            padding-top: 20px;
            border-top: 1px solid rgba(255,255,255,0.1);
        }
        
        .footer a {
            color: #3b82f6;
            text-decoration: none;
        }
        
        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
        }
        
        .loading { animation: pulse 1.5s infinite; }

        .btn-text {
            background: none;
            border: none;
            color: #666;
            cursor: pointer;
            font-size: 0.8rem;
            text-decoration: underline;
            margin-top: 8px;
        }
        .btn-text:hover { color: #888; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🚀 Apache Superset Portable</h1>
            <p class="subtitle">Панель управления сервисами</p>
        </div>
        
        <div class="services">
            <div class="service-card" id="superset-card">
                <div class="service-header">
                    <span class="service-name">📊 Superset</span>
                    <span class="status-badge status-stopped" id="superset-status">Остановлен</span>
                </div>
                <div class="service-port" id="superset-port">Порт: 8088</div>
                <div class="btn-group">
                    <button class="btn btn-primary" id="superset-open" onclick="openSuperset()" disabled>Открыть</button>
                    <button class="btn btn-secondary" id="superset-toggle" onclick="toggleSuperset()">Запустить</button>
                </div>
            </div>
            
            <div class="service-card" id="lightdocs-card">
                <div class="service-header">
                    <span class="service-name">📚 База знаний</span>
                    <span class="status-badge status-stopped" id="lightdocs-status">Остановлен</span>
                </div>
                <div class="service-port" id="lightdocs-port">Порт: 3030</div>
                <div class="btn-group">
                    <button class="btn btn-primary" id="lightdocs-open" onclick="openLightdocs()" disabled>Открыть</button>
                    <button class="btn btn-secondary" id="lightdocs-toggle" onclick="toggleLightdocs()">Запустить</button>
                </div>
            </div>
        </div>
        
        <div class="service-card" style="grid-column: 1 / -1;">
            <div class="service-header">
                <span class="service-name">🧠 База знаний (Поиск)</span>
            </div>
            <div style="display: flex; gap: 10px;">
                <input type="text" id="search-input" placeholder="Как исправить ошибку..." style="width: 100%; padding: 10px; border-radius: 8px; border: 1px solid #444; background: rgba(0,0,0,0.2); color: white;">
                <button class="btn btn-primary" onclick="searchDocs()" style="width: auto;">Найти</button>
            </div>
            <div id="search-results" style="margin-top: 15px; max-height: 200px; overflow-y: auto;"></div>
        </div>
        
        <div class="footer">
            <p>Работает автономно • <span id="uptime">0:00</span></p>
            <button class="btn-text" onclick="shutdown()">Выход</button>
        </div>
    </div>
    
    <script>
        let supersetUrl = 'http://localhost:8088';
        let lightdocsUrl = 'http://localhost:3030';
        
        async function fetchStatus() {
            try {
                const res = await fetch('/api/status');
                const data = await res.json();
                updateUI(data);
            } catch (e) {
                console.error('Status fetch failed:', e);
            }
        }

        async function searchDocs() {
            const q = document.getElementById('search-input').value;
            if (!q) return;
            
            const res = document.getElementById('search-results');
            res.innerHTML = '<div class="loading">Поиск...</div>';
            
            try {
                const req = await fetch('/api/lightdocs/search?q=' + encodeURIComponent(q));
                const results = await req.json();
                
                if (results.error) {
                    res.innerHTML = '<div style="color: red;">Ошибка индекса</div>';
                    return;
                }
                
                if (results.length === 0) {
                    res.innerHTML = '<div style="color: #888;">Ничего не найдено</div>';
                    return;
                }
                
                let html = '';
                results.forEach(item => {
                    html += `
                        <div style="margin-bottom: 10px; padding: 10px; background: rgba(255,255,255,0.05); border-radius: 8px;">
                            <a href="${lightdocsUrl}/${item.slug}.html" target="_blank" style="color: #60a5fa; text-decoration: none; font-weight: bold;">${item.title}</a>
                            <div style="color: #ccc; font-size: 0.85rem; margin-top: 5px;">${item.excerpt}</div>
                        </div>
                    `;
                });
                res.innerHTML = html;
            } catch(e) {
                res.innerHTML = 'Ошибка сети';
            }
        }
        
        function updateUI(data) {
            // Superset
            const supersetBadge = document.getElementById('superset-status');
            const supersetOpen = document.getElementById('superset-open');
            const supersetToggle = document.getElementById('superset-toggle');
            
            supersetUrl = data.superset.url;
            document.getElementById('superset-port').textContent = 'Порт: ' + data.superset.port;
            
            if (data.superset.status === 'running') {
                supersetBadge.className = 'status-badge status-running';
                supersetBadge.textContent = 'Работает';
                supersetOpen.disabled = false;
                supersetToggle.textContent = 'Остановить';
                supersetToggle.className = 'btn btn-danger';
            } else if (data.superset.status === 'starting') {
                supersetBadge.className = 'status-badge status-starting loading';
                supersetBadge.textContent = 'Запуск...';
                supersetOpen.disabled = true;
                supersetToggle.disabled = true;
            } else {
                supersetBadge.className = 'status-badge status-stopped';
                supersetBadge.textContent = 'Остановлен';
                supersetOpen.disabled = true;
                supersetToggle.textContent = 'Запустить';
                supersetToggle.className = 'btn btn-secondary';
                supersetToggle.disabled = false;
            }
            
            // LightDocs
            const lightdocsBadge = document.getElementById('lightdocs-status');
            const lightdocsOpen = document.getElementById('lightdocs-open');
            const lightdocsToggle = document.getElementById('lightdocs-toggle');
            
            lightdocsUrl = data.lightdocs.url;
            document.getElementById('lightdocs-port').textContent = 'Порт: ' + data.lightdocs.port;
            
            if (data.lightdocs.status === 'running') {
                lightdocsBadge.className = 'status-badge status-running';
                lightdocsBadge.textContent = 'Работает';
                lightdocsOpen.disabled = false;
                lightdocsToggle.textContent = 'Остановить';
                lightdocsToggle.className = 'btn btn-danger';
            } else if (data.lightdocs.status === 'starting') {
                lightdocsBadge.className = 'status-badge status-starting loading';
                lightdocsBadge.textContent = 'Запуск...';
                lightdocsOpen.disabled = true;
                lightdocsToggle.disabled = true;
            } else {
                lightdocsBadge.className = 'status-badge status-stopped';
                lightdocsBadge.textContent = 'Остановлен';
                lightdocsOpen.disabled = true;
                lightdocsToggle.textContent = 'Запустить';
                lightdocsToggle.className = 'btn btn-secondary';
                lightdocsToggle.disabled = false;
            }
            
            // Uptime
            const mins = Math.floor(data.uptime_seconds / 60);
            const secs = data.uptime_seconds % 60;
            document.getElementById('uptime').textContent = mins + ':' + String(secs).padStart(2, '0');
        }
        
        async function toggleSuperset() {
            const badge = document.getElementById('superset-status');
            const isRunning = badge.classList.contains('status-running');
            
            if (isRunning) {
                await fetch('/api/superset/stop', { method: 'POST' });
            } else {
                await fetch('/api/superset/start', { method: 'POST' });
            }
            setTimeout(fetchStatus, 500);
        }
        
        async function toggleLightdocs() {
            const badge = document.getElementById('lightdocs-status');
            const isRunning = badge.classList.contains('status-running');
            
            if (isRunning) {
                await fetch('/api/lightdocs/stop', { method: 'POST' });
            } else {
                await fetch('/api/lightdocs/start', { method: 'POST' });
            }
            setTimeout(fetchStatus, 500);
        }
        
        function openSuperset() {
            window.open(supersetUrl, '_blank');
        }
        
        function openLightdocs() {
            window.open(lightdocsUrl, '_blank');
        }

        async function shutdown() {
            if (confirm('Выключить все сервисы и закрыть лаунчер?')) {
                try {
                    await fetch('/api/shutdown', { method: 'POST' });
                    document.body.innerHTML = '<div style="color:white;text-align:center"><h1>Лаунчер остановлен</h1><p>Можно закрыть вкладку</p></div>';
                } catch (e) {
                    alert('Ошибка остановки');
                }
            }
        }
        
        // Poll status every 2 seconds
        setInterval(fetchStatus, 2000);
        fetchStatus();
    </script>
</body>
</html>
"#;
