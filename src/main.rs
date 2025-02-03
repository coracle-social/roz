use axum::{
    routing::get,
    Router,
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::time::Duration;
use nostr_sdk::prelude::*;
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

    match txn.get(state.db, &params.key) {
        Ok(value) => {
            if !value.is_empty() {
                let timestamp = u64::from_le_bytes(value.try_into().unwrap());
                (StatusCode::OK, Json(json!({ "seen": timestamp }))).into_response()
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
                    println!("Database error: {}", e);
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
    let db = env.create_db(None, lmdb::DatabaseFlags::empty())?;

    let state = Arc::new(AppState {
        env,
        db,
    });

    // Build our application with a route
    let app = Router::new()
        .route("/notary/:key", get(get_timestamp))
        .with_state(state.clone());

    // Run it on localhost:3981
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3981").await?;
    println!("Server running on http://127.0.0.1:3981");

    // Clone state for the event handling task
    let event_state = state.clone();

    // Spawn the event handling task
    let event_handle = tokio::spawn(async move {
        loop {
            // Set up a nostr client and listen for events to notarize
            let keys: Keys = Keys::generate();
            let client = Client::new(keys);

            client.add_relay("wss://relay.damus.io").await.unwrap();
            client.add_relay("wss://nos.lol").await.unwrap();
            client.connect().await;

            let filter = Filter::new()
                .since(Timestamp::now() - Duration::from_secs(30));

            let Output { val: sub_id, .. } = client.subscribe(vec![filter], None).await.unwrap();

            println!("INFO: starting subscription");

            client
                .handle_notifications(|notification| async {
                    if let RelayPoolNotification::Event {
                        subscription_id,
                        event,
                        ..
                    } = notification
                    {
                        if subscription_id == sub_id && !EPHEMERAL_RANGE.contains(&event.kind.as_u16()) {
                            let mut txn = event_state.env.begin_rw_txn().unwrap();
                            let timestamp = Timestamp::now();
                            let secs = timestamp.as_u64().to_le_bytes();
                            let key = event.id.to_hex();

                            println!("INFO: Saw {} at {}", key, timestamp);

                            txn.put(event_state.db, &key, &secs, lmdb::WriteFlags::empty()).unwrap();
                            txn.commit().unwrap();
                        }
                    }

                    Ok(false)
                })
                .await.unwrap();
        }
    });

    // Spawn the HTTP server task
    let server_handle = tokio::spawn(async move {
        println!("INFO: Starting HTTP server...");

        match axum::serve(listener, app).await {
            Ok(_) => println!("INFO: Server shut down gracefully"),
            Err(e) => eprintln!("ERROR: Server error: {}", e)
        }
    });

    // Wait for either task to finish (or error)
    tokio::select! {
        event_result = event_handle => {
            if let Err(e) = event_result {
                eprintln!("ERROR: Event handling task failed: {}", e);
            }
        }
        server_result = server_handle => {
            if let Err(e) = server_result {
                eprintln!("ERROR: Server task failed: {}", e);
            }
        }
    }

    Ok(())
}
