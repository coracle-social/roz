use axum::{
    routing::get,
    Router,
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use lmdb;
use std::error::Error;
use std::path::Path as FilePath;
use lmdb::Transaction;
use std::sync::Arc;

#[derive(Deserialize)]
struct TimestampParam {
    key: String,
}

struct AppState {
    env: Arc<lmdb::Environment>,
    db: lmdb::Database,
}

async fn get_timestamp(
    Path(params): Path<TimestampParam>,
    State(state): State<Arc<AppState>>
) -> impl IntoResponse {
    let txn = state.env.begin_ro_txn().unwrap();

    match txn.get(state.db, &params.key.as_bytes()) {
        Ok(value) => {
            if !value.is_empty() {
                (StatusCode::OK, Json(json!({ "seen": String::from_utf8_lossy(value) }))).into_response()
            } else {
                (StatusCode::NOT_FOUND, Json(json!({ "error": "Key not found" }))).into_response()
            }
        },
        Err(e) => {
            match e {
                lmdb::Error::NotFound => {
                    (StatusCode::NOT_FOUND, Json(json!({ "error": "Key not found" }))).into_response()
                },
                _ => {
                    println!("ERROR: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Internal server error" }))).into_response()
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Open our database
    let env = Arc::new(lmdb::Environment::new()
        .set_map_size(1024 * 1024 * 1024)
        .open(FilePath::new("data/db"))?);

    // Make sure our database exists and get a handle to it
    env.create_db(None, lmdb::DatabaseFlags::empty())?;
    let db = env.open_db(None)?;

    let state = Arc::new(AppState {
        env,
        db,
    });

    // Build our application with a route
    let app = Router::new()
        .route("/notary/:key", get(get_timestamp))
        .with_state(state);

    // Run it on localhost:3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("Server running on http://127.0.0.1:3000");

    Ok(axum::serve(listener, app).await?)
}
