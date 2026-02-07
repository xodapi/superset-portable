//! Gateway module for reverse proxying requests to Superset
//! 
//! Handles routing between:
//! - /docs/* -> Documentation server
//! - /static/assets/* -> Direct static file serving (Fast!)
//! - /api/v1/chart/data -> Cached API requests (Smart!)
//! - /* -> Superset backend

use axum::{
    body::Body,
    extract::{Request, State},
    response::{IntoResponse, Response},
    Router,
    http::{Method, Uri},
};
use hyper::StatusCode;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing::{info, error};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Gateway configuration state
#[derive(Clone)]
struct GatewayState {
    superset_port: u16,
    client: Client<hyper_util::client::legacy::connect::HttpConnector, Body>,
    cache: sled::Db,
}

/// Start the gateway server
pub async fn start_gateway(
    public_port: u16, 
    superset_port: u16, 
    root_path: &std::path::Path
) -> anyhow::Result<()> {
    info!("🚀 Starting Gateway on port {}", public_port);
    info!("   - /docs -> Documentation");
    info!("   - /static/assets -> Direct file serving");
    info!("   - /*    -> Superset (internal port {})", superset_port);

    // Create HTTP client for proxying
    let client: Client<hyper_util::client::legacy::connect::HttpConnector, Body> = 
        Client::builder(TokioExecutor::new()).build_http();

    // Open/Create Cache
    let cache_path = root_path.join("cache").join("gateway_sled");
    let cache = sled::open(&cache_path)?;
    info!("   - Smart Cache enabled at: {}", cache_path.display());

    let state = GatewayState {
        superset_port,
        client,
        cache,
    };

    // Docs service
    // Served as static for now, or use docs server logic? 
    // Actually docs are served by docs_server.rs on 8089. Gateway proxies /docs to it? 
    // The previous code served directory "docs", let's keep that logic but point to _site if built?
    // User wanted "LightDocs Integration". LightDocs builds to `_site`.
    // Let's point /docs to `_site` if it exists, else `knowledge`.
    let site_path = root_path.join("_site");
    let docs_root = if site_path.exists() { site_path } else { root_path.join("knowledge") };
    let docs_service = ServeDir::new(docs_root).append_index_html_on_directories(true);

    // Static Assets Service (Direct from Python env)
    // Path: python/Lib/site-packages/superset/static/assets
    let static_assets_path = root_path.join("python/Lib/site-packages/superset/static/assets");
    let static_service = ServeDir::new(static_assets_path);

    // Build router
    let app = Router::new()
        .nest_service("/docs", docs_service)
        .nest_service("/static/assets", static_service) // Intercept static assets
        .fallback(proxy_handler) // Smart proxy for everything else
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], public_port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("Gateway listening on http://localhost:{}", public_port);
    axum::serve(listener, app).await?;

    Ok(())
}

/// Handler that proxies requests to Superset with Smart Caching
async fn proxy_handler(
    State(state): State<GatewayState>,
    mut req: Request,
) -> Result<Response, StatusCode> {
    let path = req.uri().path().to_string();
    let method = req.method().clone();
    
    // Check if cacheable (API chart data)
    // /api/v1/chart/data is POST
    if method == Method::POST && path == "/api/v1/chart/data" {
        return handle_cached_request(state, req).await;
    }

    // Standard Proxy
    forward_request(state, req).await
}

async fn handle_cached_request(
    state: GatewayState,
    req: Request,
) -> Result<Response, StatusCode> {
    // 1. Read Body to Hash
    let (parts, body) = req.into_parts();
    let bytes = axum::body::to_bytes(body, usize::MAX).await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // 2. Compute Hash
    let mut hasher = DefaultHasher::new();
    parts.uri.path().hash(&mut hasher);
    bytes.hash(&mut hasher); // Hash the JSON body
    let hash = hasher.finish();
    let key = format!("req_{}", hash);

    // 3. Check Cache
    if let Ok(Some(cached)) = state.cache.get(&key) {
        // Return cached response
        // Note: We need to store headers + status + body.
        // For simplicity v1, assuming 200 OK and application/json.
        // Better: use serde to store struct { status, headers, body }
        // Here we just return body as JSON.
        info!("⚡ CACHE HIT: {}", parts.uri.path());
        
        let body = Body::from(cached.to_vec());
        let mut response = Response::new(body);
        *response.status_mut() = StatusCode::OK;
        response.headers_mut().insert("content-type", "application/json".parse().unwrap());
        response.headers_mut().insert("x-superset-cache", "HIT".parse().unwrap());
        return Ok(response);
    }

    // 4. Cache Miss - Forward Request
    // Reconstruct request
    let body = Body::from(bytes.clone());
    let mut new_req = Request::from_parts(parts, body);
    
    // Helper to modify URI for forwarding
    let path_query = new_req.uri().path_and_query().map(|v| v.as_str()).unwrap_or("/").to_string();
    let uri_string = format!("http://127.0.0.1:{}{}", state.superset_port, path_query);
    let uri = uri_string.parse::<Uri>().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    *new_req.uri_mut() = uri;
    new_req.headers_mut().remove("host");

    // Execute
    match state.client.request(new_req).await {
        Ok(res) => {
            let status = res.status();
            if status.is_success() {
                // Cache the response body
                // We need to read response body to cache it
                let (resp_parts, resp_body) = res.into_parts();
                let resp_bytes = axum::body::to_bytes(Body::new(resp_body), usize::MAX).await
                    .map_err(|_| StatusCode::BAD_GATEWAY)?;
                
                // Save to sled (TTL could be added here)
                let _ = state.cache.insert(&key, resp_bytes.to_vec());
                let _ = state.cache.flush();
                info!("🐢 CACHE MISS: {} (Cached {} bytes)", path_query, resp_bytes.len());

                // Return response
                let mut response = Response::from_parts(resp_parts, Body::from(resp_bytes));
                response.headers_mut().insert("x-superset-cache", "MISS".parse().unwrap());
                Ok(response)
            } else {
                Ok(res.into_response())
            }
        }
        Err(e) => {
            error!("Proxy error: {}", e);
            Err(StatusCode::BAD_GATEWAY)
        }
    }
}

async fn forward_request(state: GatewayState, mut req: Request) -> Result<Response, StatusCode> {
    let path_query = req.uri().path_and_query().map(|v| v.as_str()).unwrap_or("/");
    let uri_string = format!("http://127.0.0.1:{}{}", state.superset_port, path_query);
    
    if let Ok(uri) = uri_string.parse::<Uri>() {
        *req.uri_mut() = uri;
        req.headers_mut().remove("host");
        
        match state.client.request(req).await {
            Ok(res) => Ok(res.into_response()),
            Err(e) => {
                error!("Proxy error: {}", e);
                Err(StatusCode::BAD_GATEWAY)
            }
        }
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
