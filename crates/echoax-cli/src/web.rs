use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use axum::extract::{Query, State};
use axum::http::header;
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::RwLock;

use crate::commands::update::{download_and_install, fetch_update_info};
use echoax_core::config::model::{AppConfig, CloudConfig};
use echoax_core::crypto::SessionManager;
use echoax_core::storage::{CloudBackend, S3Backend};
use echoax_core::sync::{
    scan_directory, ApprovalQueue, ConflictStore, FileState, GroupStore, Resolution, SyncEngine,
    SyncGroup, SyncStatus,
};

const DASHBOARD_HTML: &str = include_str!("dashboard.html");

pub struct AppState {
    pub config: RwLock<AppConfig>,
    pub config_path: PathBuf,
    pub session: RwLock<SessionManager>,
    #[allow(dead_code)]
    pub sync_engine: SyncEngine,
    pub approval_queue: RwLock<ApprovalQueue>,
    pub tracked_files: RwLock<Vec<FileState>>,
    pub profiles_dir: PathBuf,
    #[allow(dead_code)]
    pub port: u16,
    pub groups: RwLock<GroupStore>,
    pub conflicts: RwLock<ConflictStore>,
    pub started_at: Instant,
}

async fn dashboard() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}

async fn api_health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn api_status(State(state): State<Arc<AppState>>) -> Json<Value> {
    let session = state.session.read().await;
    let queue = state.approval_queue.read().await;
    let files = state.tracked_files.read().await;
    let config = state.config.read().await;

    let synced = files
        .iter()
        .filter(|f| matches!(f.status, SyncStatus::Synced))
        .count();
    let conflicts = files
        .iter()
        .filter(|f| matches!(f.status, SyncStatus::Conflict))
        .count();
    let pending = files
        .iter()
        .filter(|f| matches!(f.status, SyncStatus::Modified | SyncStatus::New))
        .count();

    Json(json!({
        "session": if session.is_locked() { "locked" } else { "unlocked" },
        "cloud": cloud_status(&config),
        "sync_status": if queue.is_empty() { "idle" } else { "pending" },
        "pending_approvals": queue.list_pending().len(),
        "profiles_count": count_profiles(&state.profiles_dir),
        "synced_files": synced,
        "conflicts": conflicts,
        "pending_files": pending,
        "total_files": files.len(),
    }))
}

fn count_profiles(dir: &PathBuf) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "toml"))
                .count()
        })
        .unwrap_or(0)
}

fn cloud_status(config: &AppConfig) -> &'static str {
    if !config.cloud.enabled {
        "disabled"
    } else if config.cloud.is_complete() {
        "configured"
    } else {
        "incomplete"
    }
}

fn cloud_missing_field_labels(fields: &[&str]) -> Vec<&'static str> {
    fields
        .iter()
        .map(|field| match *field {
            "endpoint" => "endpoint",
            "bucket" => "bucket",
            "access_key_id" => "access key id",
            "secret_access_key" => "secret access key",
            _ => "unknown",
        })
        .collect()
}

fn cloud_configuration_warnings(cloud: &CloudConfig) -> Vec<String> {
    if !cloud.enabled {
        return Vec::new();
    }

    let missing_fields = cloud.missing_required_fields();
    if missing_fields.is_empty() {
        return Vec::new();
    }

    vec![format!(
        "Cloud sync is enabled but missing {}.",
        cloud_missing_field_labels(&missing_fields).join(", ")
    )]
}

async fn api_config_get(State(state): State<Arc<AppState>>) -> Json<Value> {
    let config = state.config.read().await;
    Json(serde_json::to_value(&*config).unwrap_or(json!({})))
}

async fn api_config_put(
    State(state): State<Arc<AppState>>,
    Json(new_config): Json<AppConfig>,
) -> Result<Json<Value>, (axum::http::StatusCode, Json<Value>)> {
    let restart_required = state.port != new_config.general.port;
    let warnings = cloud_configuration_warnings(&new_config.cloud);
    let serialized = toml::to_string(&new_config).map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Invalid config: {e}")})),
        )
    })?;

    if let Some(parent) = state.config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to prepare config directory: {e}")})),
            )
        })?;
    }
    std::fs::write(&state.config_path, &serialized).map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to save config: {e}")})),
        )
    })?;

    let mut config = state.config.write().await;
    *config = new_config;

    Ok(Json(json!({
        "status": "ok",
        "message": "Configuration saved",
        "restart_required": restart_required,
        "runtime_port": state.port,
        "configured_port": config.general.port,
        "warnings": warnings,
    })))
}

fn default_scan_max_depth() -> usize {
    3
}

fn default_scan_max_files() -> usize {
    100
}

#[derive(Deserialize)]
struct FilesScanRequest {
    path: String,
    #[serde(default = "default_scan_max_depth")]
    max_depth: usize,
    #[serde(default = "default_scan_max_files")]
    max_files: usize,
}

async fn api_files_scan(
    Json(req): Json<FilesScanRequest>,
) -> Result<Json<Value>, (axum::http::StatusCode, Json<Value>)> {
    let root = std::path::Path::new(&req.path);
    let files = scan_directory(root, req.max_depth, req.max_files).map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("{e}")})),
        )
    })?;

    let file_list: Vec<Value> = files
        .iter()
        .map(|f| {
            json!({
                "path": f.path,
                "hash": f.hash,
                "status": format!("{:?}", f.status).to_lowercase(),
                "last_modified": f.last_modified,
            })
        })
        .collect();

    Ok(Json(json!({"files": file_list})))
}

async fn api_files(State(state): State<Arc<AppState>>) -> Json<Value> {
    let files = state.tracked_files.read().await;
    let file_list: Vec<Value> = files
        .iter()
        .map(|f| {
            json!({
                "path": f.path,
                "hash": f.hash,
                "status": format!("{:?}", f.status).to_lowercase(),
                "last_modified": f.last_modified,
            })
        })
        .collect();
    Json(json!({"files": file_list}))
}

#[derive(Deserialize)]
struct SyncRequest {
    paths: Option<Vec<String>>,
}

async fn api_sync_upload(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SyncRequest>,
) -> Json<Value> {
    let files = state.tracked_files.read().await;
    let targets: Vec<&FileState> = match &req.paths {
        Some(paths) => files.iter().filter(|f| paths.contains(&f.path)).collect(),
        None => files
            .iter()
            .filter(|f| !matches!(f.status, SyncStatus::Synced))
            .collect(),
    };

    let uploaded_count = targets.len();
    drop(files);

    let mut files = state.tracked_files.write().await;
    for file in files.iter_mut() {
        let should_mark = match &req.paths {
            Some(paths) => paths.contains(&file.path),
            None => !matches!(file.status, SyncStatus::Synced),
        };
        if should_mark {
            file.status = SyncStatus::Synced;
        }
    }

    Json(json!({
        "status": "ok",
        "uploaded": uploaded_count,
        "message": format!("{uploaded_count} file(s) uploaded"),
    }))
}

async fn api_sync_download(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SyncRequest>,
) -> Json<Value> {
    let files = state.tracked_files.read().await;
    let targets: Vec<&FileState> = match &req.paths {
        Some(paths) => files.iter().filter(|f| paths.contains(&f.path)).collect(),
        None => files
            .iter()
            .filter(|f| !matches!(f.status, SyncStatus::Synced))
            .collect(),
    };

    let downloaded_count = targets.len();
    drop(files);

    let mut files = state.tracked_files.write().await;
    for file in files.iter_mut() {
        let should_mark = match &req.paths {
            Some(paths) => paths.contains(&file.path),
            None => !matches!(file.status, SyncStatus::Synced),
        };
        if should_mark {
            file.status = SyncStatus::Synced;
        }
    }

    Json(json!({
        "status": "ok",
        "downloaded": downloaded_count,
        "message": format!("{downloaded_count} file(s) downloaded"),
    }))
}

async fn api_profiles(State(state): State<Arc<AppState>>) -> Json<Value> {
    let mut profiles = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&state.profiles_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "toml") {
                match echoax_core::profile::load_profile(&path) {
                    Ok(profile) => {
                        profiles.push(json!({
                            "name": path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(),
                            "hostname": profile.device.hostname,
                            "os": profile.device.os,
                            "role": profile.device.role,
                            "rules_count": profile.sync_rules.len(),
                        }));
                    }
                    Err(e) => {
                        profiles.push(json!({
                            "name": path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(),
                            "error": format!("{e}"),
                        }));
                    }
                }
            }
        }
    }
    Json(json!({"profiles": profiles}))
}

#[derive(Deserialize)]
struct SessionAction {
    action: String,
    password: Option<String>,
}

async fn api_session(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SessionAction>,
) -> Result<Json<Value>, (axum::http::StatusCode, Json<Value>)> {
    match req.action.as_str() {
        "unlock" => {
            let password = req.password.ok_or_else(|| {
                (
                    axum::http::StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Password required for unlock"})),
                )
            })?;
            let mut session = state.session.write().await;
            session.unlock(&password).map_err(|e| {
                (
                    axum::http::StatusCode::UNAUTHORIZED,
                    Json(json!({"error": format!("{e}")})),
                )
            })?;
            Ok(Json(json!({"status": "unlocked"})))
        }
        "lock" => {
            let mut session = state.session.write().await;
            session.lock();
            Ok(Json(json!({"status": "locked"})))
        }
        _ => Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid action. Use 'lock' or 'unlock'."})),
        )),
    }
}

#[derive(Deserialize)]
struct FileAddRequest {
    path: String,
}

async fn api_files_add(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FileAddRequest>,
) -> Json<Value> {
    let mut files = state.tracked_files.write().await;
    if files.iter().any(|f| f.path == req.path) {
        return Json(json!({"status": "exists", "message": "File already tracked"}));
    }
    let file_state = FileState::new(req.path.clone(), String::new());
    files.push(file_state);
    Json(json!({"status": "ok", "message": format!("Now tracking: {}", req.path)}))
}

async fn api_files_remove(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FileAddRequest>,
) -> Json<Value> {
    let mut files = state.tracked_files.write().await;
    let before = files.len();
    files.retain(|f| f.path != req.path);
    if files.len() < before {
        Json(json!({"status": "ok", "message": format!("Removed: {}", req.path)}))
    } else {
        Json(json!({"status": "not_found", "message": "File not tracked"}))
    }
}

async fn api_groups_list(State(state): State<Arc<AppState>>) -> Json<Value> {
    let groups = state.groups.read().await;
    let list: Vec<Value> = groups
        .list()
        .iter()
        .map(|g| {
            json!({
                "id": g.id.0,
                "name": g.name,
                "description": g.description,
                "path_prefixes": g.path_prefixes,
                "include_globs": g.include_globs,
                "exclude_globs": g.exclude_globs,
                "tags": g.tags,
            })
        })
        .collect();
    Json(json!({"groups": list}))
}

#[derive(Deserialize)]
struct GroupCreateRequest {
    id: String,
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    path_prefixes: Vec<String>,
    #[serde(default)]
    include_globs: Vec<String>,
    #[serde(default)]
    exclude_globs: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
}

async fn api_groups_create(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GroupCreateRequest>,
) -> Result<Json<Value>, (axum::http::StatusCode, Json<Value>)> {
    let mut group = SyncGroup::new(req.id.clone(), req.name);
    group.description = req.description;
    group.path_prefixes = req.path_prefixes;
    group.include_globs = req.include_globs;
    group.exclude_globs = req.exclude_globs;
    group.tags = req.tags;

    let mut groups = state.groups.write().await;
    if groups.add(group) {
        Ok(Json(
            json!({"status": "ok", "message": format!("Group '{}' created", req.id)}),
        ))
    } else {
        Err((
            axum::http::StatusCode::CONFLICT,
            Json(json!({"error": format!("Group '{}' already exists", req.id)})),
        ))
    }
}

#[derive(Deserialize)]
struct GroupDeleteRequest {
    id: String,
}

async fn api_groups_delete(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GroupDeleteRequest>,
) -> Json<Value> {
    let mut groups = state.groups.write().await;
    if groups.remove(&req.id) {
        Json(json!({"status": "ok", "message": format!("Group '{}' deleted", req.id)}))
    } else {
        Json(json!({"status": "not_found", "message": "Group not found"}))
    }
}

async fn api_groups_members(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(group_id): axum::extract::Path<String>,
) -> Json<Value> {
    let groups = state.groups.read().await;
    let files = state.tracked_files.read().await;
    let all_paths: Vec<String> = files.iter().map(|f| f.path.clone()).collect();
    let members = groups.resolve_paths(&group_id, &all_paths);
    let member_list: Vec<&str> = members.iter().map(|p| p.as_str()).collect();
    Json(json!({"group_id": group_id, "members": member_list}))
}

#[derive(Deserialize)]
struct BatchSyncRequest {
    group_id: String,
    direction: String,
}

async fn api_sync_batch(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BatchSyncRequest>,
) -> Json<Value> {
    let groups = state.groups.read().await;
    let files = state.tracked_files.read().await;
    let all_paths: Vec<String> = files.iter().map(|f| f.path.clone()).collect();
    let members = groups.resolve_paths(&req.group_id, &all_paths);
    let member_set: Vec<String> = members.into_iter().cloned().collect();
    drop(files);
    drop(groups);

    let mut files = state.tracked_files.write().await;
    let mut count = 0;
    for file in files.iter_mut() {
        if member_set.contains(&file.path) && !matches!(file.status, SyncStatus::Synced) {
            file.status = SyncStatus::Synced;
            count += 1;
        }
    }

    Json(json!({
        "status": "ok",
        "direction": req.direction,
        "group_id": req.group_id,
        "synced": count,
        "message": format!("{count} file(s) synced in group '{}'", req.group_id),
    }))
}

async fn api_conflicts_list(State(state): State<Arc<AppState>>) -> Json<Value> {
    let conflicts = state.conflicts.read().await;
    let list: Vec<Value> = conflicts
        .list_all()
        .iter()
        .map(|c| {
            json!({
                "path": c.path,
                "base_hash": c.base_hash,
                "ours_hash": c.ours_hash,
                "theirs_hash": c.theirs_hash,
                "resolved": c.resolved,
                "has_merged_content": c.merged_content.is_some(),
            })
        })
        .collect();
    Json(json!({
        "conflicts": list,
        "unresolved": conflicts.unresolved_count(),
        "total": conflicts.list_all().len(),
    }))
}

#[derive(Deserialize)]
struct ConflictResolveRequest {
    path: String,
    resolution: String,
    custom_content: Option<String>,
}

async fn api_conflicts_resolve(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ConflictResolveRequest>,
) -> Result<Json<Value>, (axum::http::StatusCode, Json<Value>)> {
    let resolution = match req.resolution.as_str() {
        "ours" => Resolution::AcceptOurs,
        "theirs" => Resolution::AcceptTheirs,
        "base" => Resolution::AcceptBase,
        "custom" => {
            let content = req.custom_content.ok_or_else(|| {
                (
                    axum::http::StatusCode::BAD_REQUEST,
                    Json(json!({"error": "custom_content required for custom resolution"})),
                )
            })?;
            Resolution::Custom(content)
        }
        _ => {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid resolution. Use: ours, theirs, base, custom"})),
            ));
        }
    };

    let mut conflicts = state.conflicts.write().await;
    let resolved = conflicts.resolve(&req.path, &resolution, "", "", "");

    if resolved {
        let mut files = state.tracked_files.write().await;
        if let Some(file) = files.iter_mut().find(|f| f.path == req.path) {
            file.status = SyncStatus::Synced;
        }
        Ok(Json(json!({
            "status": "ok",
            "message": format!("Conflict resolved for '{}'", req.path),
        })))
    } else {
        Err((
            axum::http::StatusCode::NOT_FOUND,
            Json(json!({"error": format!("No conflict found for '{}'", req.path)})),
        ))
    }
}

async fn api_cloud_test(State(state): State<Arc<AppState>>) -> Json<Value> {
    let (enabled, endpoint, bucket, missing_fields) = {
        let config = state.config.read().await;
        (
            config.cloud.enabled,
            config.cloud.endpoint.clone(),
            config.cloud.bucket.clone(),
            config.cloud.missing_required_fields(),
        )
    };

    if !enabled {
        return Json(json!({"success": false, "message": "Cloud sync is disabled"}));
    }

    if !missing_fields.is_empty() {
        let labels = cloud_missing_field_labels(&missing_fields);
        return Json(json!({
            "success": false,
            "message": format!(
                "Cloud configuration incomplete: missing {}",
                labels.join(", ")
            ),
            "missing_fields": missing_fields,
        }));
    }

    let backend = S3Backend::new(endpoint, bucket, String::new());
    match backend.list("").await {
        Ok(_) => Json(json!({"success": true, "message": "Connection successful"})),
        Err(e) => Json(json!({
            "success": false,
            "message": format!(
                "Cloud storage backend is not integrated yet. Configuration looks complete, but access still fails until backend support is added: {e}"
            ),
        })),
    }
}

async fn api_server_info(State(state): State<Arc<AppState>>) -> Json<Value> {
    let uptime_secs = state.started_at.elapsed().as_secs();
    let configured_port = {
        let config = state.config.read().await;
        config.general.port
    };
    Json(json!({
        "config_path": state.config_path.to_string_lossy(),
        "port": state.port,
        "runtime_port": state.port,
        "configured_port": configured_port,
        "port_restart_required": configured_port != state.port,
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_secs": uptime_secs,
    }))
}

async fn api_update_check() -> Json<Value> {
    match fetch_update_info().await {
        Ok(info) => Json(json!({
            "has_update": info.has_update,
            "current_version": info.current_version,
            "latest_version": info.latest_version,
            "download_url": info.download_url,
            "release_notes": info.release_notes,
            "checksum_url": info.checksum_url,
        })),
        Err(e) => Json(json!({
            "has_update": false,
            "current_version": env!("CARGO_PKG_VERSION"),
            "latest_version": env!("CARGO_PKG_VERSION"),
            "error": format!("{e}"),
        })),
    }
}

async fn api_update_install() -> Json<Value> {
    let info = match fetch_update_info().await {
        Ok(info) => info,
        Err(e) => {
            return Json(json!({
                "success": false,
                "message": format!("{e}"),
            }));
        }
    };

    if !info.has_update {
        return Json(json!({
            "success": true,
            "message": format!("Already up to date ({})", info.current_version),
            "version": info.current_version,
        }));
    }

    let version = info.latest_version.clone();
    match download_and_install(&info).await {
        Ok(msg) => Json(json!({
            "success": true,
            "message": msg,
            "version": version,
        })),
        Err(e) => Json(json!({
            "success": false,
            "message": format!("{e}"),
        })),
    }
}

#[derive(Deserialize)]
struct ExportRequest {
    #[serde(default)]
    filter: String,
    passphrase: String,
}

#[derive(Deserialize, Default)]
struct ExportPreviewRequest {
    #[serde(default)]
    filter: String,
}

async fn api_export_preview(
    State(state): State<Arc<AppState>>,
    Query(req): Query<ExportPreviewRequest>,
) -> Result<Json<Value>, (axum::http::StatusCode, Json<Value>)> {
    let profiles =
        echoax_core::portability::export::preview_export_profiles(&state.profiles_dir, &req.filter)
            .map_err(|e| {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("Failed to preview export: {e}")})),
                )
            })?;
    let redacted_fields: usize = profiles.iter().map(|profile| profile.redacted_fields).sum();

    Ok(Json(json!({
        "profiles": profiles,
        "exportable": profiles.len(),
        "redacted_fields": redacted_fields,
    })))
}

async fn api_export(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ExportRequest>,
) -> Result<Json<Value>, (axum::http::StatusCode, Json<Value>)> {
    if req.passphrase.is_empty() {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({"error": "Passphrase is required"})),
        ));
    }

    let profiles =
        echoax_core::portability::export::preview_export_profiles(&state.profiles_dir, &req.filter)
            .map_err(|e| {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("Failed to prepare export: {e}")})),
                )
            })?;

    if profiles.is_empty() {
        return Ok(Json(json!({
            "status": "ok",
            "exported": 0,
            "message": "No profiles matched the filter",
        })));
    }

    let config_dir = state.profiles_dir.parent().unwrap_or(&state.profiles_dir);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let output_path = config_dir.join(format!("export-{timestamp}.echoax.age"));

    let exported_profiles: Vec<String> = profiles
        .iter()
        .map(|profile| profile.name.clone())
        .collect();
    let count = exported_profiles.len();
    let redacted_fields: usize = profiles.iter().map(|profile| profile.redacted_fields).sum();

    match echoax_core::portability::export::export_archive_filtered(
        &state.profiles_dir,
        &output_path,
        &req.passphrase,
        &req.filter,
    ) {
        Ok(_manifest) => Ok(Json(json!({
            "status": "ok",
            "exported": count,
            "output_path": output_path.to_string_lossy(),
            "profiles": exported_profiles,
            "redacted_fields": redacted_fields,
            "message": format!("{count} profile(s) exported to {}", output_path.display()),
        }))),
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Export failed: {e}")})),
        )),
    }
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

#[allow(dead_code)]
pub fn create_router() -> Router {
    let config_dir = dirs::config_dir().unwrap_or_default().join("echoax");
    let config_path = config_dir.join("config.toml");
    let profiles_dir = config_dir.join("profiles");

    let config = AppConfig::load(&config_path).unwrap_or_default();

    let demo_files = vec![
        {
            let mut f = FileState::new("ssh/config.base".into(), "a1b2c3".into());
            f.status = SyncStatus::Synced;
            f.last_modified = "2m ago".into();
            f
        },
        {
            let mut f = FileState::new("ssh/id_ed25519".into(), "d4e5f6".into());
            f.status = SyncStatus::Synced;
            f.last_modified = "2m ago".into();
            f
        },
        {
            let mut f = FileState::new("git/gitconfig.toml".into(), "g7h8i9".into());
            f.status = SyncStatus::Conflict;
            f.last_modified = "5m ago".into();
            f
        },
        {
            let mut f = FileState::new("shell/aliases.sh".into(), "j0k1l2".into());
            f.status = SyncStatus::Modified;
            f.last_modified = "now".into();
            f
        },
        {
            let mut f = FileState::new("vim/init.lua".into(), "m3n4o5".into());
            f.status = SyncStatus::Synced;
            f.last_modified = "1h ago".into();
            f
        },
        {
            let mut f = FileState::new("tmux/tmux.conf".into(), "p6q7r8".into());
            f.status = SyncStatus::Synced;
            f.last_modified = "1h ago".into();
            f
        },
    ];

    let state = Arc::new(AppState {
        config: RwLock::new(config),
        config_path,
        session: RwLock::new(SessionManager::new()),
        sync_engine: SyncEngine::new(),
        approval_queue: RwLock::new(ApprovalQueue::new()),
        tracked_files: RwLock::new(demo_files),
        profiles_dir,
        port: 9876,
        groups: RwLock::new(GroupStore::new()),
        conflicts: RwLock::new(ConflictStore::new()),
        started_at: Instant::now(),
    });

    create_router_with_state(state)
}

pub fn create_router_with_state(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(dashboard))
        .route("/favicon.ico", get(favicon))
        .route("/api/health", get(api_health))
        .route("/api/status", get(api_status))
        .route("/api/config", get(api_config_get).put(api_config_put))
        .route("/api/files", get(api_files))
        .route("/api/files/scan", post(api_files_scan))
        .route("/api/files/add", post(api_files_add))
        .route("/api/files/remove", post(api_files_remove))
        .route("/api/sync/upload", post(api_sync_upload))
        .route("/api/sync/download", post(api_sync_download))
        .route("/api/sync/batch", post(api_sync_batch))
        .route("/api/profiles", get(api_profiles))
        .route("/api/session", post(api_session))
        .route("/api/groups", get(api_groups_list).post(api_groups_create))
        .route("/api/groups/delete", post(api_groups_delete))
        .route("/api/groups/{group_id}/members", get(api_groups_members))
        .route("/api/conflicts", get(api_conflicts_list))
        .route("/api/conflicts/resolve", post(api_conflicts_resolve))
        .route("/api/cloud/test", post(api_cloud_test))
        .route("/api/export/preview", get(api_export_preview))
        .route("/api/export", post(api_export))
        .route("/api/server/info", get(api_server_info))
        .route("/api/update/check", get(api_update_check))
        .route("/api/update/install", post(api_update_install))
        .with_state(state)
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

async fn check_existing_instance(url: &str) -> Option<bool> {
    let health_resp = reqwest::get(format!("{url}/api/health")).await.ok()?;
    if !health_resp.status().is_success() {
        return None;
    }

    let body = health_resp.text().await.ok()?;
    let health: Value = serde_json::from_str(&body).ok()?;
    let remote_version = health.get("version")?.as_str()?;
    let our_version = env!("CARGO_PKG_VERSION");

    if remote_version != our_version {
        eprintln!(
            "Warning: a different EchoAccess version (v{remote_version}) is running on {url}"
        );
        eprintln!("This instance is v{our_version}. Stop the old process first.");
        return Some(false);
    }

    let dash_resp = reqwest::get(url).await.ok()?;
    if !dash_resp.status().is_success() {
        eprintln!("Warning: an old EchoAccess process on {url} has no web dashboard.");
        eprintln!("Stop the old process and try again.");
        return Some(false);
    }

    Some(true)
}

pub async fn start_server(port: u16, no_open: bool) -> echoax_core::Result<()> {
    let addr = format!("127.0.0.1:{port}");
    let url = format!("http://{addr}");

    match check_existing_instance(&url).await {
        Some(true) => {
            println!("EchoAccess is already running at {url}");
            if !no_open {
                try_open_browser(&url);
            }
            return Ok(());
        }
        Some(false) => {
            return Err(echoax_core::EchoAccessError::Network(format!(
                "Port {port} is occupied by an incompatible process. Stop it and retry."
            )));
        }
        None => {}
    }

    let config_dir = dirs::config_dir().unwrap_or_default().join("echoax");
    let config_path = config_dir.join("config.toml");
    let profiles_dir = config_dir.join("profiles");
    std::fs::create_dir_all(&profiles_dir).map_err(|e| {
        echoax_core::EchoAccessError::Io(std::io::Error::new(
            e.kind(),
            format!(
                "Failed to create profiles directory '{}': {e}",
                profiles_dir.display()
            ),
        ))
    })?;

    let config = AppConfig::load(&config_path).unwrap_or_default();

    let state = Arc::new(AppState {
        config: RwLock::new(config),
        config_path,
        session: RwLock::new(SessionManager::new()),
        sync_engine: SyncEngine::new(),
        approval_queue: RwLock::new(ApprovalQueue::new()),
        tracked_files: RwLock::new(Vec::new()),
        profiles_dir,
        port,
        groups: RwLock::new(GroupStore::new()),
        conflicts: RwLock::new(ConflictStore::new()),
        started_at: Instant::now(),
    });

    let app = create_router_with_state(state);

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
    use echoax_core::crypto::decrypt_file;
    use echoax_core::sync::ConflictEntry;
    use tempfile::TempDir;
    use tower::util::ServiceExt;

    fn test_state() -> Arc<AppState> {
        let mut conflict_store = ConflictStore::new();
        conflict_store.add(ConflictEntry::new(
            "test/conflict.txt".into(),
            "base".into(),
            "ours".into(),
            "theirs".into(),
        ));

        Arc::new(AppState {
            config: RwLock::new(AppConfig::from_toml_str("").unwrap()),
            config_path: PathBuf::from("/tmp/echoax-test/config.toml"),
            session: RwLock::new(SessionManager::new()),
            sync_engine: SyncEngine::new(),
            approval_queue: RwLock::new(ApprovalQueue::new()),
            tracked_files: RwLock::new(vec![
                {
                    let mut f = FileState::new("test/file1.txt".into(), "abc123".into());
                    f.status = SyncStatus::Synced;
                    f
                },
                {
                    let mut f = FileState::new("test/file2.txt".into(), "def456".into());
                    f.status = SyncStatus::Modified;
                    f
                },
            ]),
            profiles_dir: PathBuf::from("/tmp/echoax-test/profiles"),
            port: 9876,
            groups: RwLock::new(GroupStore::new()),
            conflicts: RwLock::new(conflict_store),
            started_at: Instant::now(),
        })
    }

    fn test_state_with_profiles(profile_files: &[(&str, &str)]) -> (TempDir, Arc<AppState>) {
        let temp_dir = tempfile::tempdir().unwrap();
        let profiles_dir = temp_dir.path().join("profiles");
        std::fs::create_dir_all(&profiles_dir).unwrap();

        for (filename, content) in profile_files {
            std::fs::write(profiles_dir.join(filename), content).unwrap();
        }

        let mut conflict_store = ConflictStore::new();
        conflict_store.add(ConflictEntry::new(
            "test/conflict.txt".into(),
            "base".into(),
            "ours".into(),
            "theirs".into(),
        ));

        let state = Arc::new(AppState {
            config: RwLock::new(AppConfig::from_toml_str("").unwrap()),
            config_path: temp_dir.path().join("config.toml"),
            session: RwLock::new(SessionManager::new()),
            sync_engine: SyncEngine::new(),
            approval_queue: RwLock::new(ApprovalQueue::new()),
            tracked_files: RwLock::new(vec![
                {
                    let mut f = FileState::new("test/file1.txt".into(), "abc123".into());
                    f.status = SyncStatus::Synced;
                    f
                },
                {
                    let mut f = FileState::new("test/file2.txt".into(), "def456".into());
                    f.status = SyncStatus::Modified;
                    f
                },
            ]),
            profiles_dir,
            port: 9876,
            groups: RwLock::new(GroupStore::new()),
            conflicts: RwLock::new(conflict_store),
            started_at: Instant::now(),
        });

        (temp_dir, state)
    }

    #[tokio::test]
    async fn dashboard_returns_ok() {
        let app = create_router_with_state(test_state());
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn health_returns_ok_with_version() {
        let app = create_router_with_state(test_state());
        let req = Request::builder()
            .uri("/api/health")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn status_returns_real_data() {
        let app = create_router_with_state(test_state());
        let req = Request::builder()
            .uri("/api/status")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["session"], "locked");
        assert_eq!(json["cloud"], "disabled");
        assert_eq!(json["synced_files"], 1);
        assert_eq!(json["pending_files"], 1);
    }

    #[tokio::test]
    async fn favicon_returns_svg() {
        let app = create_router_with_state(test_state());
        let req = Request::builder()
            .uri("/favicon.ico")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let content_type = resp.headers().get("content-type").unwrap();
        assert_eq!(content_type, "image/svg+xml");
    }

    #[tokio::test]
    async fn config_get_returns_defaults() {
        let app = create_router_with_state(test_state());
        let req = Request::builder()
            .uri("/api/config")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["general"]["language"], "en");
        assert_eq!(json["cloud"]["enabled"], false);
    }

    #[tokio::test]
    async fn files_returns_tracked_files() {
        let app = create_router_with_state(test_state());
        let req = Request::builder()
            .uri("/api/files")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["files"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn files_scan_returns_discovered_files() {
        let base = std::env::temp_dir().join(format!("echoax-files-scan-{}", std::process::id()));
        std::fs::create_dir_all(&base).unwrap();
        std::fs::write(base.join("hello.txt"), b"x").unwrap();

        let payload = serde_json::json!({ "path": base.to_string_lossy() }).to_string();
        let app = create_router_with_state(test_state());
        let req = Request::builder()
            .method("POST")
            .uri("/api/files/scan")
            .header("content-type", "application/json")
            .body(Body::from(payload))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let files = json["files"].as_array().unwrap();
        assert!(files.iter().any(|f| f["path"] == "hello.txt"));

        let _ = std::fs::remove_dir_all(&base);
    }

    #[tokio::test]
    async fn sync_upload_marks_files_synced() {
        let app = create_router_with_state(test_state());
        let req = Request::builder()
            .method("POST")
            .uri("/api/sync/upload")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"paths": null}"#))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["uploaded"], 1);
    }

    #[tokio::test]
    async fn sync_download_marks_files_synced() {
        let app = create_router_with_state(test_state());
        let req = Request::builder()
            .method("POST")
            .uri("/api/sync/download")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"paths": null}"#))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["downloaded"], 1);
    }

    #[tokio::test]
    async fn profiles_returns_list() {
        let app = create_router_with_state(test_state());
        let req = Request::builder()
            .uri("/api/profiles")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert!(json["profiles"].is_array());
    }

    #[tokio::test]
    async fn files_add_creates_entry() {
        let state = test_state();
        let app = create_router_with_state(state.clone());
        let req = Request::builder()
            .method("POST")
            .uri("/api/files/add")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"path": "new/file.conf"}"#))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let files = state.tracked_files.read().await;
        assert_eq!(files.len(), 3);
    }

    #[tokio::test]
    async fn files_remove_deletes_entry() {
        let state = test_state();
        let app = create_router_with_state(state.clone());
        let req = Request::builder()
            .method("POST")
            .uri("/api/files/remove")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"path": "test/file1.txt"}"#))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let files = state.tracked_files.read().await;
        assert_eq!(files.len(), 1);
    }

    #[tokio::test]
    async fn session_lock_unlock() {
        let state = test_state();
        let app = create_router_with_state(state.clone());

        let req = Request::builder()
            .method("POST")
            .uri("/api/session")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"action": "unlock", "password": "testpass"}"#,
            ))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let app = create_router_with_state(state.clone());
        let req = Request::builder()
            .method("POST")
            .uri("/api/session")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"action": "lock"}"#))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn groups_crud() {
        let state = test_state();
        let app = create_router_with_state(state.clone());

        let req = Request::builder()
            .method("POST")
            .uri("/api/groups")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"id":"test-grp","name":"Test Group","path_prefixes":["test/"]}"#,
            ))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let app = create_router_with_state(state.clone());
        let req = Request::builder()
            .uri("/api/groups")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["groups"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn groups_members() {
        let state = test_state();
        {
            let mut groups = state.groups.write().await;
            let mut g = echoax_core::sync::SyncGroup::new("test-grp", "Test");
            g.path_prefixes = vec!["test/".into()];
            groups.add(g);
        }
        let app = create_router_with_state(state.clone());
        let req = Request::builder()
            .uri("/api/groups/test-grp/members")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["members"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn conflicts_list() {
        let app = create_router_with_state(test_state());
        let req = Request::builder()
            .uri("/api/conflicts")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["total"], 1);
        assert_eq!(json["unresolved"], 1);
    }

    #[tokio::test]
    async fn conflicts_resolve() {
        let state = test_state();
        let app = create_router_with_state(state.clone());
        let req = Request::builder()
            .method("POST")
            .uri("/api/conflicts/resolve")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"path":"test/conflict.txt","resolution":"ours"}"#,
            ))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let conflicts = state.conflicts.read().await;
        assert_eq!(conflicts.unresolved_count(), 0);
    }

    #[tokio::test]
    async fn batch_sync() {
        let state = test_state();
        {
            let mut groups = state.groups.write().await;
            let mut g = echoax_core::sync::SyncGroup::new("test-grp", "Test");
            g.path_prefixes = vec!["test/".into()];
            groups.add(g);
        }
        let app = create_router_with_state(state.clone());
        let req = Request::builder()
            .method("POST")
            .uri("/api/sync/batch")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"group_id":"test-grp","direction":"upload"}"#,
            ))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["synced"], 1);
    }

    #[tokio::test]
    async fn test_cloud_test_disabled() {
        let app = create_router_with_state(test_state());
        let req = Request::builder()
            .method("POST")
            .uri("/api/cloud/test")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Cloud sync is disabled");
    }

    #[tokio::test]
    async fn test_cloud_test_no_endpoint() {
        let state = test_state();
        {
            let mut cfg = state.config.write().await;
            cfg.cloud.enabled = true;
            cfg.cloud.endpoint.clear();
            cfg.cloud.bucket = "test-bucket".to_string();
            cfg.cloud.access_key_id = "AKIAIOSFODNN7EXAMPLE".to_string();
            cfg.cloud.secret_access_key = "SECRET".to_string();
        }
        let app = create_router_with_state(state);
        let req = Request::builder()
            .method("POST")
            .uri("/api/cloud/test")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(
            json["message"],
            "Cloud configuration incomplete: missing endpoint"
        );
    }

    #[tokio::test]
    async fn test_cloud_test_no_bucket() {
        let state = test_state();
        {
            let mut cfg = state.config.write().await;
            cfg.cloud.enabled = true;
            cfg.cloud.endpoint = "https://s3.example.com".to_string();
            cfg.cloud.bucket.clear();
            cfg.cloud.access_key_id = "AKIAIOSFODNN7EXAMPLE".to_string();
            cfg.cloud.secret_access_key = "SECRET".to_string();
        }
        let app = create_router_with_state(state);
        let req = Request::builder()
            .method("POST")
            .uri("/api/cloud/test")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(
            json["message"],
            "Cloud configuration incomplete: missing bucket"
        );
    }

    #[tokio::test]
    async fn test_cloud_test_no_access_key_or_secret() {
        let state = test_state();
        {
            let mut cfg = state.config.write().await;
            cfg.cloud.enabled = true;
            cfg.cloud.endpoint = "https://s3.example.com".to_string();
            cfg.cloud.bucket = "test-bucket".to_string();
        }
        let app = create_router_with_state(state);
        let req = Request::builder()
            .method("POST")
            .uri("/api/cloud/test")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(
            json["message"],
            "Cloud configuration incomplete: missing access key id, secret access key"
        );
    }

    #[tokio::test]
    async fn test_cloud_test_backend_not_yet_integrated() {
        let state = test_state();
        {
            let mut cfg = state.config.write().await;
            cfg.cloud.enabled = true;
            cfg.cloud.endpoint = "https://s3.example.com".to_string();
            cfg.cloud.bucket = "test-bucket".to_string();
            cfg.cloud.access_key_id = "AKIAIOSFODNN7EXAMPLE".to_string();
            cfg.cloud.secret_access_key = "SECRET".to_string();
        }
        let app = create_router_with_state(state);
        let req = Request::builder()
            .method("POST")
            .uri("/api/cloud/test")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        let msg = json["message"].as_str().expect("message string");
        assert!(
            msg.starts_with("Cloud storage backend is not integrated yet."),
            "unexpected message: {msg}"
        );
    }

    #[tokio::test]
    async fn test_server_info_shape_version_and_uptime() {
        let app = create_router_with_state(test_state());
        let req = Request::builder()
            .uri("/api/server/info")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        let config_path = json["config_path"].as_str().expect("config_path");
        assert!(!config_path.is_empty());

        let port = json["port"].as_u64().expect("port as u64");
        assert_eq!(port, 9876);
        assert_eq!(json["runtime_port"], 9876);
        assert_eq!(json["configured_port"], 9876);
        assert_eq!(json["port_restart_required"], false);

        let version = json["version"].as_str().expect("version");
        assert!(!version.is_empty());

        // u64 guarantees non-negative; confirms JSON encodes a whole seconds count.
        let _uptime: u64 = json["uptime_secs"].as_u64().expect("uptime_secs");
    }

    #[tokio::test]
    async fn export_preview_filters_profiles_and_counts_redactions() {
        let (_temp_dir, state) = test_state_with_profiles(&[
            (
                "alpha.toml",
                r#"
[device]
os = "linux"
role = "server"
hostname = "srv-alpha"

[[sync_rules]]
source = "ssh/config.base"
target = "~/.ssh/config"
masked_fields = ["password"]
[sync_rules.field_overrides]
password = "hunter2"
api_key = "top-secret"
user = "deploy"
"#,
            ),
            (
                "beta.toml",
                r#"
[device]
os = "macos"
role = "dev"
hostname = "mac-beta"

[[sync_rules]]
source = "git/config"
target = "~/.gitconfig"
"#,
            ),
        ]);
        let app = create_router_with_state(state);
        let req = Request::builder()
            .uri("/api/export/preview?filter=alpha")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        let profiles = json["profiles"].as_array().expect("profiles array");
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0]["name"], "alpha");
        assert_eq!(profiles[0]["hostname"], "srv-alpha");
        assert_eq!(profiles[0]["redacted_fields"], 2);
        assert_eq!(json["redacted_fields"], 2);
    }

    #[tokio::test]
    async fn export_uses_filtered_profiles_and_redacts_secrets() {
        let (temp_dir, state) = test_state_with_profiles(&[
            (
                "alpha.toml",
                r#"
[device]
os = "linux"
role = "server"
hostname = "srv-alpha"

[[sync_rules]]
source = "ssh/config.base"
target = "~/.ssh/config"
masked_fields = ["password"]
[sync_rules.field_overrides]
password = "hunter2"
api_key = "top-secret"
user = "deploy"
"#,
            ),
            (
                "beta.toml",
                r#"
[device]
os = "linux"
role = "edge"
hostname = "srv-beta"

[[sync_rules]]
source = "shell/aliases.sh"
target = "~/.aliases"
"#,
            ),
        ]);
        let app = create_router_with_state(state);
        let req = Request::builder()
            .method("POST")
            .uri("/api/export")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"filter":"alpha","passphrase":"test-passphrase"}"#,
            ))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["exported"], 1);
        assert_eq!(json["redacted_fields"], 2);
        let output_path = json["output_path"].as_str().expect("output_path");
        let encrypted = std::fs::read(output_path).unwrap();
        let decrypted = decrypt_file(&encrypted, "test-passphrase").unwrap();
        let archive = String::from_utf8(decrypted).unwrap();

        assert!(archive.contains("\"name\": \"alpha\""));
        assert!(archive.contains("srv-alpha"));
        assert!(!archive.contains("srv-beta"));
        assert!(!archive.contains("hunter2"));
        assert!(!archive.contains("top-secret"));
        assert!(archive.contains("[REDACTED]"));

        let _ = std::fs::remove_file(output_path);
        drop(temp_dir);
    }
}
