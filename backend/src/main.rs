use axum::{
    routing::{get, post},
    Router,
};
use anyhow::Result;
use config::Config;
use handlers::{add_domains, check_health, get_domain, get_domains,db_test};
use sqlx::PgPool;
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
async fn main() -> Result<()>{
    let config = Config::from_env();
    let pool = match PgPool::connect(&config.database_url).await{
        Ok(pool)=>pool,
        Err(e)=>{
            eprintln!("Failed to connect to database: {:?}", e);
            return Err(e.into());
        }
    };
    let state = AppState {
        domains: Arc::new(Mutex::new(HashMap::new())),
        pool,
    };

    match sqlx::query!("CREATE TABLE IF NOT EXISTS domains (
               id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                duration INTEGER NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT now()
            )")
    .execute(&state.pool)
    .await{
        Ok(_)=>println!("Domains table created or already exists"),
        Err(e)=>{
            eprintln!("Failed to create domains table: {:?}", e);
            return Err(e.into());
        }
    };

    let domains = state.domains.clone();

    let app = Router::new()
        .route("/domains", post(add_domains))
        .route("/domains/active", get(get_domains))
        .route("/health", get(check_health))
        .route("/domains/{id}", get(get_domain))
        .route("/db-test",get(db_test))
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
    Ok(())
}
