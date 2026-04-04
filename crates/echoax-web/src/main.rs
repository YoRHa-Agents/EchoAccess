mod routes;

use axum::routing::get;
use axum::Router;

fn create_router() -> Router {
    Router::new()
        .route("/api/health", get(routes::health::health_check))
        .route("/api/status", get(routes::status::get_status))
}

#[tokio::main]
async fn main() {
    let app = create_router();
    let addr = "127.0.0.1:9876";
    println!("EchoAccess Web API listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");
    axum::serve(listener, app)
        .await
        .expect("Server error");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn health_check_returns_ok() {
        let app = create_router();
        let req = Request::builder()
            .uri("/api/health")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn status_returns_json() {
        let app = create_router();
        let req = Request::builder()
            .uri("/api/status")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
