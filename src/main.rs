// #![allow(warnings)]
#![feature(error_generic_member_access)]
use std::{env, net::SocketAddr, sync::Arc};

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::post,
};
use log::LevelFilter;
use serde::Deserialize;
use simple_logger::SimpleLogger;
use tokio::signal;
use tower_http::services::ServeFile;

use crate::{database::DatabaseManager, query::{EngineError, QueryResult}};

mod attribute;
mod bincode;
mod database;
mod file;
mod metagraph;
mod node_id;
mod query;
mod storage;
// mod test;

// const MAIN_DIR: &str = "/var/lib/metagraph-db/data";
const MAIN_DIR: &str = "data";

#[derive(Clone)]
struct AppState {
    database_manager: Arc<DatabaseManager>,
}

#[tokio::main]
async fn main() {
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();
    
    let port: u16 = env::var("PORT").unwrap_or("8080".to_string()).parse().unwrap();
    let address = SocketAddr::from(([0, 0, 0, 0], port));
    
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .unwrap();
    log::info!("Server listening on port {}", port);

    let db_manager = Arc::new(DatabaseManager::new(MAIN_DIR));
    let state = AppState {
        database_manager: db_manager.clone(),
    };

    let router = Router::new()
        .route_service("/", ServeFile::new("ui/index.html"))
        .route("/query", post(handle_query))
        .route("/syntax", post(handle_check_syntax))
        .with_state(state);

    axum::serve(listener, router).with_graceful_shutdown(shutdown_signal(db_manager.clone())).await.unwrap()
}

#[derive(Debug, Deserialize)]
struct QueryDTO {
    db_name: String,
    query: String,
}

async fn handle_query(
    State(state): State<AppState>,
    Json(query): Json<QueryDTO>,
) -> Result<Json<QueryResult>, EngineError> {
    let db = state.database_manager.open(&query.db_name);
    let result = db.execute(&query.query)?;
    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
struct CheckSyntaxDTO {
    query: String,
}

async fn handle_check_syntax(
    State(state): State<AppState>,
    Json(query): Json<CheckSyntaxDTO>,
) -> Result<StatusCode, EngineError> {
    match parser::parse(&query.query) {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Ok(StatusCode::BAD_REQUEST),
    }
}

async fn shutdown_signal(database_manager: Arc<DatabaseManager>) {
    signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C handler");
    
    log::info!("Shutting down...");
    database_manager.close();
}