use axum::{
    routing::{get, post},
    Router,
};
use config::Config;
use handlers::{add_domains, check_health, get_domain, get_domains};
use state::AppState;
use std::net::SocketAddr;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tasks::cleanup_expired_domains;
use tokio::task;
mod config;
mod errors;
mod handlers;
mod models;
mod state;
mod tasks;

#[tokio::main]
async fn main() {
    let config = Config::from_env();
    let state = AppState {
        domains: Arc::new(Mutex::new(HashMap::new())),
    };

    let domains = state.domains.clone();

    let app = Router::new()
        .route("/domains", post(add_domains))
        .route("/domains/active", get(get_domains))
        .route("/health", get(check_health))
        .route("/domains/{id}", get(get_domain))
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .expect("Invalid address");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    println!("Server running on http://{addr}");

    task::spawn(cleanup_expired_domains(domains));

    axum::serve(listener, app)
        .await
        .expect("Failed to start the server");
}
