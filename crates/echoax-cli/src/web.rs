use axum::http::header;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Json;
use axum::Router;
use serde_json::{json, Value};

const DASHBOARD_HTML: &str = include_str!("dashboard.html");

async fn dashboard() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}

async fn api_health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn api_status() -> Json<Value> {
    Json(json!({
        "session": "locked",
        "cloud": "disconnected",
        "sync_status": "idle",
        "pending_approvals": 0,
    }))
}

async fn favicon() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/svg+xml")],
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 32 32\">\
         <rect width=\"32\" height=\"32\" rx=\"4\" fill=\"#2E2A27\"/>\
         <text x=\"16\" y=\"22\" text-anchor=\"middle\" fill=\"#C87941\" \
         font-family=\"monospace\" font-size=\"18\" font-weight=\"bold\">E</text></svg>",
    )
}

pub fn create_router() -> Router {
    Router::new()
        .route("/", get(dashboard))
        .route("/favicon.ico", get(favicon))
        .route("/api/health", get(api_health))
        .route("/api/status", get(api_status))
}

fn try_open_browser(url: &str) {
    match open::that(url) {
        Ok(()) => println!("Browser opened at {url}"),
        Err(e) => {
            eprintln!("Could not open browser automatically: {e}");
            println!("Please visit {url} in your browser");
        }
    }
}

pub async fn start_server(port: u16, no_open: bool) -> echoax_core::Result<()> {
    let addr = format!("127.0.0.1:{port}");
    let url = format!("http://{addr}");

    if let Ok(resp) = reqwest::get(format!("{url}/api/health")).await {
        if resp.status().is_success() {
            println!("EchoAccess is already running at {url}");
            if !no_open {
                try_open_browser(&url);
            }
            return Ok(());
        }
    }

    let app = create_router();

    println!("EchoAccess Web UI starting at {url}");

    let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
        echoax_core::EchoAccessError::Network(format!("Failed to bind {addr}: {e}"))
    })?;

    if !no_open {
        let open_url = url.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            try_open_browser(&open_url);
        });
    }

    println!("Press Ctrl+C to stop");

    axum::serve(listener, app)
        .await
        .map_err(|e| echoax_core::EchoAccessError::Network(format!("Server error: {e}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn dashboard_returns_ok() {
        let app = create_router();
        let req = Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn health_returns_ok_with_version() {
        let app = create_router();
        let req = Request::builder()
            .uri("/api/health")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn status_returns_ok() {
        let app = create_router();
        let req = Request::builder()
            .uri("/api/status")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn favicon_returns_svg() {
        let app = create_router();
        let req = Request::builder()
            .uri("/favicon.ico")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let content_type = resp.headers().get("content-type").unwrap();
        assert_eq!(content_type, "image/svg+xml");
    }
}
