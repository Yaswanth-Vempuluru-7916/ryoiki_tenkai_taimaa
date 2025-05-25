use axum::{
    extract::{Path, State},
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

#[derive(Debug)]
enum AppError {
    NotFound(String),
    Internal(String),
    Conflict(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Domain {
    id: i32,
    name: String,
    duration: i32,
}

#[derive(Debug,Serialize,Deserialize)]
struct DomainStatus{
    id : i32,
    name : String,
    duration : i32,
    remaining_seconds : u64
}

#[derive(Clone,Debug)]
struct AppState {
    domains: Arc<Mutex<HashMap<i32, (Domain, Instant)>>>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
}


impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status,message) = match self {
            AppError::Internal(msg)=>(StatusCode::INTERNAL_SERVER_ERROR,msg),
            AppError::NotFound(msg)=>(StatusCode::NOT_FOUND,msg),
            AppError::Conflict(msg)=>(StatusCode::CONFLICT,msg)
        };
         let body = Json(serde_json::json!({"error":message}));
         (status,body).into_response()
    }
}

async fn check_health() -> Json<HealthResponse> {
   Json(HealthResponse{
    status :  "I am healthier than you BROther".to_string(),
   })
}

async fn add_domains(
    State(state): State<AppState>,
    Json(payload): Json<Domain>,
) -> Result<(StatusCode, Json<Domain>), AppError> {
    let mut domains = state.domains.lock().map_err(|_| AppError::Internal("Mutex Poisoned".into()))?;
    println!("Adding domain: {:?}", payload);
    if domains.contains_key(&payload.id) {
        return Err(AppError::Conflict("Domain already exists".to_string()));
    }
    domains.insert(payload.id, (payload.clone(), Instant::now()));
    println!("Domain added");
    Ok((StatusCode::CREATED, Json(payload)))
}

async fn get_domains(State(state): State<AppState>) ->Result<Json<Vec<Domain>>, AppError> {
    let domains = state.domains.lock().map_err(|_| AppError::Internal("Mutex Poisoned".into()))?;
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
    Ok(Json(active_domains))
}

async fn get_domain(State(state): State<AppState>,Path(id):Path<i32>)->Result<(StatusCode, Json<DomainStatus>), AppError>{

    let domains = state.domains.lock().map_err(|_| AppError::Internal("Mutex Poisoned".into()))?;

    if let Some(( domain , instant)) = domains.get(&id){
       let now = Instant::now();

       if now.duration_since(*instant)>=Duration::from_secs(domain.duration as u64){
        return Err(AppError::NotFound(format!("Domain ID {} expired", id)));
       }
        
       let expires_at = *instant + Duration::from_secs(domain.duration as u64);
       let remaining_duration = expires_at.saturating_duration_since(now);
       
      
        let domain_status = DomainStatus{
            id : domain.id,
            name : domain.name.clone(),
            duration : domain.duration,
            remaining_seconds  : remaining_duration.as_secs()
        };


        Ok((StatusCode::OK,Json(domain_status)))
       
    }else{
        Err(AppError::NotFound(format!("Domain ID {} not found",id)))
    }
   


}

async fn cleanup_expired_domains(domains: Arc<Mutex<HashMap<i32, (Domain, Instant)>>>) {
    let mut interval = interval(Duration::from_secs(1)); // Use 1s for testing, revert to 5s for production
    loop {
        interval.tick().await;
        let mut domains = match domains.lock().map_err(|_| AppError::Internal("Mutex Poisoned".into())) {
            Ok(domains) => domains,
            Err(e) => {
                println!("Cleanup error: {:?}", e);
                continue;
            }
        };
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
        .route("/domains/{id}",get(get_domain))
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