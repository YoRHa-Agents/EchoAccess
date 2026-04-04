use axum::Json;
use serde_json::{json, Value};

pub async fn get_status() -> Json<Value> {
    Json(json!({
        "session": "locked",
        "cloud": "disconnected",
        "sync_status": "idle",
        "pending_approvals": 0,
    }))
}
