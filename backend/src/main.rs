use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use config::Config;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{task, time::interval};
mod config;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Domain {
    id: i32,
    name: String,
    duration: i32,
}

#[derive(Clone)]
struct AppState {
    domains: Arc<Mutex<HashMap<i32, (Domain, Instant)>>>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        (StatusCode::CONFLICT, Json(self)).into_response()
    }
}

async fn check_health() -> axum::Json<HealthResponse> {
    let health_status = String::from("I am healthier than you BROther");
    axum::Json(HealthResponse {
        status: health_status,
    })
}

async fn add_domains(
    State(state): State<AppState>,
    Json(payload): Json<Domain>,
) -> Result<(StatusCode, Json<Domain>), ErrorResponse> {
    let mut domains = state.domains.lock().expect("Mutex poisoned");
    println!("Adding domain: {:?}", payload);
    if domains.contains_key(&payload.id) {
        println!("Domain already exists");
        return Err(ErrorResponse {
            error: "Domain already exists".to_string(),
        });
    }
    domains.insert(payload.id, (payload.clone(), Instant::now()));
    println!("Domain added");
    Ok((StatusCode::CREATED, Json(payload)))
}

async fn get_domains(State(state): State<AppState>) -> Json<Vec<Domain>> {
    let domains = state.domains.lock().expect("Mutex poisoned");
    let now = Instant::now();

    let active_domains = domains
        .iter()
        .filter_map(|(_key, (domain, instant))| {
            if now.duration_since(*instant) < Duration::from_secs(domain.duration as u64) {
                Some(domain.clone())
            } else {
                None
            }
        })
        .collect();
    println!("Returning domains: {:?}", active_domains);
    Json(active_domains)
}

async fn cleanup_expired_domains(domains: Arc<Mutex<HashMap<i32, (Domain, Instant)>>>) {
    let mut interval = interval(Duration::from_secs(1)); // Use 1s for testing, revert to 5s for production
    loop {
        interval.tick().await;
        let mut domains = domains.lock().expect("Mutex poisoned");
        let before = domains.len();
        let now = Instant::now();

        domains.retain(|id, (domain, instant)| {
            let keep = now.duration_since(*instant) < Duration::from_secs(domain.duration as u64);
            if !keep {
                println!("Removed domain ID: {}", id);
            }
            keep
        });
        let after = domains.len();
        println!("Cleanup ran. Removed {} expired domain(s).", before - after);
    }
}

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