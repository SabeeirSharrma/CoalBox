use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, Query, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use coalbox_core::{
    generate_passphrase, generate_password, Entry, EntryId, PassphraseConfig, PasswordConfig,
    TotpConfig, Vault,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

// ── CLI ──────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "coalbox-web", version = "0.6.0", about = "Coalbox WebUI")]
struct Cli {
    /// Path to the vault file
    #[arg(short, long, default_value = "~/.local/share/coalbox/vault.emberkeys")]
    vault: String,

    /// Port to listen on (0 = random)
    #[arg(short, long, default_value = "0")]
    port: u16,

    /// Don't open browser automatically
    #[arg(long)]
    no_open: bool,
}

// ── Shared state ─────────────────────────────────────────────────────

type SharedVault = Arc<RwLock<Option<VaultState>>>;

struct VaultState {
    vault: Vault,
    password: String,
    vault_path: String,
}

#[derive(Clone)]
struct AppState {
    vault: SharedVault,
    default_vault_path: String,
    tx: broadcast::Sender<String>,
}

// ── API types ────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ApiResponse<T: Serialize> {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl ApiResponse<()> {
    fn err(msg: &str) -> Json<ApiResponse<()>> {
        Json(ApiResponse {
            ok: false,
            data: None,
            error: Some(msg.to_string()),
        })
    }
}

impl<T: Serialize> ApiResponse<T> {
    fn ok(data: T) -> Json<ApiResponse<T>> {
        Json(ApiResponse {
            ok: true,
            data: Some(data),
            error: None,
        })
    }
}

#[derive(Deserialize)]
struct UnlockRequest {
    password: String,
}

#[derive(Serialize)]
struct StatusResponse {
    locked: bool,
    entry_count: usize,
    vault_path: String,
}

#[derive(Deserialize)]
struct GenerateRequest {
    #[serde(default)]
    passphrase: bool,
    #[serde(default = "default_password_length")]
    length: usize,
    #[serde(default = "default_word_count")]
    words: usize,
    #[serde(default = "default_separator")]
    separator: String,
    #[serde(default = "default_true")]
    capitalize: bool,
    #[serde(default)]
    number: bool,
}

fn default_password_length() -> usize { 20 }
fn default_word_count() -> usize { 6 }
fn default_separator() -> String { " ".to_string() }
fn default_true() -> bool { true }

#[derive(Serialize)]
struct GeneratedPassword {
    password: String,
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

#[derive(Deserialize)]
struct CreateEntryRequest {
    title: String,
    #[serde(default)]
    username: String,
    #[serde(default)]
    password: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    notes: String,
    #[serde(default)]
    totp_secret: String,
    #[serde(default = "default_entry_type")]
    entry_type: String,
}

fn default_entry_type() -> String { "login".to_string() }

#[derive(Deserialize)]
struct UpdateEntryRequest {
    title: Option<String>,
    username: Option<String>,
    password: Option<String>,
    url: Option<String>,
    notes: Option<String>,
    totp_secret: Option<String>,
}

// ── Handlers ─────────────────────────────────────────────────────────

async fn get_status(State(state): State<AppState>) -> Json<ApiResponse<StatusResponse>> {
    let vault = state.vault.read().await;
    let (locked, entry_count, vault_path) = match vault.as_ref() {
        Some(v) => (false, v.vault.entry_count(), v.vault_path.clone()),
        None => (true, 0, state.default_vault_path.clone()),
    };
    ApiResponse::ok(StatusResponse {
        locked,
        entry_count,
        vault_path,
    })
}

async fn unlock(
    State(state): State<AppState>,
    Json(req): Json<UnlockRequest>,
) -> Result<Json<ApiResponse<StatusResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let vault_path = {
        let v = state.vault.read().await;
        v.as_ref()
            .map(|v| v.vault_path.clone())
            .unwrap_or_else(|| state.default_vault_path.clone())
    };

    let path = std::path::PathBuf::from(shellexpand::tilde(&vault_path).to_string());

    match Vault::unlock(&path, &req.password) {
        Ok(vault) => {
            let entry_count = vault.entry_count();
            let mut v = state.vault.write().await;
            *v = Some(VaultState {
                vault,
                password: req.password,
                vault_path: vault_path.clone(),
            });
            let _ = state.tx.send("unlock".to_string());
            Ok(ApiResponse::ok(StatusResponse {
                locked: false,
                entry_count,
                vault_path,
            }))
        }
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            ApiResponse::err(&format!("Failed to unlock: {}", e)),
        )),
    }
}

async fn lock(State(state): State<AppState>) -> Json<ApiResponse<StatusResponse>> {
    let mut vault = state.vault.write().await;
    *vault = None;
    let _ = state.tx.send("lock".to_string());
    ApiResponse::ok(StatusResponse {
        locked: true,
        entry_count: 0,
        vault_path: state.default_vault_path.clone(),
    })
}

async fn list_entries(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Entry>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let v = state.vault.read().await;
    match v.as_ref() {
        Some(vs) => match vs.vault.list_entries() {
            Ok(entries) => Ok(ApiResponse::ok(entries)),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiResponse::err(&format!("Failed to list entries: {}", e)),
            )),
        },
        None => Err((StatusCode::UNAUTHORIZED, ApiResponse::err("Vault is locked"))),
    }
}

async fn get_entry(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Entry>>, (StatusCode, Json<ApiResponse<()>>)> {
    let entry_id = match EntryId::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, ApiResponse::err("Invalid entry ID"))),
    };
    let v = state.vault.read().await;
    match v.as_ref() {
        Some(vs) => match vs.vault.get_entry(&entry_id) {
            Ok(entry) => Ok(ApiResponse::ok(entry)),
            Err(e) => Err((StatusCode::NOT_FOUND, ApiResponse::err(&format!("Entry not found: {}", e)))),
        },
        None => Err((StatusCode::UNAUTHORIZED, ApiResponse::err("Vault is locked"))),
    }
}

async fn create_entry(
    State(state): State<AppState>,
    Json(req): Json<CreateEntryRequest>,
) -> Result<Json<ApiResponse<Entry>>, (StatusCode, Json<ApiResponse<()>>)> {
    let mut v = state.vault.write().await;
    match v.as_mut() {
        Some(vs) => {
            let entry = match req.entry_type.as_str() {
                "note" => Entry::new_note(req.title, req.notes),
                "authenticator" => Entry::new_authenticator(req.title, req.totp_secret.clone()),
                _ => Entry::new_login(req.title, req.username, req.password),
            };
            let entry = if !req.url.is_empty() {
                entry.with_url(req.url)
            } else {
                entry
            };
            let entry = if !req.totp_secret.is_empty() {
                entry.with_totp(req.totp_secret)
            } else {
                entry
            };
            let _ = entry.id;
            match vs.vault.add_entry(entry.clone()) {
                Ok(_) => {
                    if let Err(e) = vs.vault.save(&vs.password) {
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            ApiResponse::err(&format!("Failed to save: {}", e)),
                        ));
                    }
                    let _ = state.tx.send("entries_changed".to_string());
                    Ok(ApiResponse::ok(entry))
                }
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiResponse::err(&format!("Failed to create: {}", e)),
                )),
            }
        }
        None => Err((StatusCode::UNAUTHORIZED, ApiResponse::err("Vault is locked"))),
    }
}

async fn update_entry(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateEntryRequest>,
) -> Result<Json<ApiResponse<Entry>>, (StatusCode, Json<ApiResponse<()>>)> {
    let entry_id = match EntryId::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, ApiResponse::err("Invalid entry ID"))),
    };
    let mut v = state.vault.write().await;
    match v.as_mut() {
        Some(vs) => {
            let _ = vs.vault.update_entry(&entry_id, |entry| {
                if let Some(title) = req.title {
                    entry.title = title;
                }
                if let Some(username) = req.username {
                    entry.username = Some(username);
                }
                if let Some(password) = req.password {
                    entry.password = Some(password);
                }
                if let Some(url) = req.url {
                    entry.url = Some(url);
                }
                if let Some(notes) = req.notes {
                    entry.notes = Some(notes);
                }
                if let Some(totp_secret) = req.totp_secret {
                    entry.totp_secret = if totp_secret.is_empty() {
                        None
                    } else {
                        Some(totp_secret)
                    };
                }
                entry.modified = chrono::Utc::now();
            });

            match vs.vault.save(&vs.password) {
                Ok(_) => {
                    let _ = state.tx.send("entries_changed".to_string());
                    match vs.vault.get_entry(&entry_id) {
                        Ok(entry) => Ok(ApiResponse::ok(entry)),
                        Err(e) => Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            ApiResponse::err(&format!("Failed to read updated entry: {}", e)),
                        )),
                    }
                }
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiResponse::err(&format!("Failed to save: {}", e)),
                )),
            }
        }
        None => Err((StatusCode::UNAUTHORIZED, ApiResponse::err("Vault is locked"))),
    }
}

async fn delete_entry(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let entry_id = match EntryId::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, ApiResponse::err("Invalid entry ID"))),
    };
    let mut v = state.vault.write().await;
    match v.as_mut() {
        Some(vs) => match vs.vault.delete_entry(&entry_id) {
            Ok(_) => {
                if let Err(e) = vs.vault.save(&vs.password) {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ApiResponse::err(&format!("Failed to save: {}", e)),
                    ));
                }
                let _ = state.tx.send("entries_changed".to_string());
                Ok(ApiResponse::ok(()))
            }
            Err(e) => Err((StatusCode::NOT_FOUND, ApiResponse::err(&format!("Entry not found: {}", e)))),
        },
        None => Err((StatusCode::UNAUTHORIZED, ApiResponse::err("Vault is locked"))),
    }
}

async fn search_entries(
    State(state): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<ApiResponse<Vec<Entry>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let v = state.vault.read().await;
    match v.as_ref() {
        Some(vs) => {
            let results = vs.vault.search(&q.q);
            Ok(ApiResponse::ok(results))
        }
        None => Err((StatusCode::UNAUTHORIZED, ApiResponse::err("Vault is locked"))),
    }
}

async fn generate_password_endpoint(
    Json(req): Json<GenerateRequest>,
) -> Json<ApiResponse<GeneratedPassword>> {
    let password = if req.passphrase {
        let config = PassphraseConfig {
            word_count: req.words,
            separator: req.separator,
            capitalize: req.capitalize,
            include_number: req.number,
        };
        generate_passphrase(&config)
    } else {
        let config = PasswordConfig {
            length: req.length,
            uppercase: true,
            lowercase: true,
            numbers: true,
            symbols: true,
            custom_symbols: None,
            exclude_chars: None,
        };
        generate_password(&config)
    };
    ApiResponse::ok(GeneratedPassword { password })
}

async fn get_totp(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<HashMap<String, String>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let entry_id = match EntryId::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, ApiResponse::err("Invalid entry ID"))),
    };
    let v = state.vault.read().await;
    match v.as_ref() {
        Some(vs) => {
            let entry = match vs.vault.get_entry(&entry_id) {
                Ok(e) => e,
                Err(e) => {
                    return Err((StatusCode::NOT_FOUND, ApiResponse::err(&format!("Entry not found: {}", e))))
                }
            };

            match &entry.totp_secret {
                Some(secret) => match TotpConfig::from_secret(secret) {
                    Ok(config) => {
                        let code = config.generate_current();
                        let mut map = HashMap::new();
                        map.insert("code".to_string(), code.code);
                        map.insert("remaining".to_string(), code.remaining.to_string());
                        Ok(ApiResponse::ok(map))
                    }
                    Err(e) => Err((StatusCode::BAD_REQUEST, ApiResponse::err(&format!("Invalid TOTP: {}", e)))),
                },
                None => Err((StatusCode::NOT_FOUND, ApiResponse::err("No TOTP configured"))),
            }
        }
        None => Err((StatusCode::UNAUTHORIZED, ApiResponse::err("Vault is locked"))),
    }
}

// ── WebSocket ────────────────────────────────────────────────────────

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> axum::response::Response {
    ws.on_upgrade(|socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();

    let status = {
        let v = state.vault.read().await;
        match v.as_ref() {
            Some(_) => "unlock",
            None => "lock",
        }
    };
    let _ = socket.send(Message::Text(status.into())).await;

    loop {
        tokio::select! {
            Ok(msg) = rx.recv() => {
                let _ = socket.send(Message::Text(msg.into())).await;
            }
            Some(Ok(msg)) = socket.recv() => {
                match msg {
                    Message::Text(t) => {
                        if t == "ping" {
                            let _ = socket.send(Message::Text("pong".into())).await;
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
            else => break,
        }
    }
}

// ── Frontend ─────────────────────────────────────────────────────────

const INDEX_HTML: &str = include_str!("index.html");

async fn serve_index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

// ── Main ─────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(false)
        .init();

    let (tx, _) = broadcast::channel(32);

    let vault_path = shellexpand::tilde(&cli.vault).to_string();

    let state = AppState {
        vault: Arc::new(RwLock::new(None)),
        default_vault_path: vault_path.clone(),
        tx,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/status", get(get_status))
        .route("/api/unlock", post(unlock))
        .route("/api/lock", post(lock))
        .route("/api/entries", get(list_entries).post(create_entry))
        .route("/api/entries/{id}", get(get_entry).put(update_entry).delete(delete_entry))
        .route("/api/search", get(search_entries))
        .route("/api/generate", post(generate_password_endpoint))
        .route("/api/entries/{id}/totp", get(get_totp))
        .route("/ws", get(ws_handler))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], cli.port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");

    let local_addr = listener.local_addr().expect("Failed to get local addr");

    println!("╔══════════════════════════════════════════════╗");
    println!("║           Coalbox WebUI v0.6.0              ║");
    println!("╠══════════════════════════════════════════════╣");
    println!("║  URL:   http://{}", local_addr);
    println!("║  Vault: {}", vault_path);
    println!("║  Press Ctrl+C to stop                       ║");
    println!("╚══════════════════════════════════════════════╝");

    if !cli.no_open {
        let url = format!("http://{}", local_addr);
        let _ = open::that(&url);
    }

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}
