use std::{collections::HashMap, sync::{Arc, Mutex}};
use axum::{
    extract::State, http::{StatusCode}, response::{IntoResponse, Response}, routing::{get, post}, Json, Router};
use config::Config;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
mod config;

#[derive(Serialize,Deserialize,Clone,Debug)]
struct Domain {
    id : i32,
    name : String,
    duration : i32
}
#[derive(Clone)]
struct AppState{
    domains : Arc<Mutex<HashMap<i32,Domain>>>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for ErrorResponse{
    fn into_response(self)->Response{
        (StatusCode::CONFLICT,Json(self)).into_response()
    }
}

async fn check_health() -> axum::Json<HealthResponse> {
    let health_status = String::from("I am healthier than you BROther");
    axum::Json(HealthResponse {
        status: health_status,
    })
}

async fn add_domains( 
    State(state) : State<AppState>,
    Json(payload) : Json<Domain>,
   )
    ->Result<(StatusCode, Json<Domain>), ErrorResponse>{
        let  mut domains = state.domains.lock().expect("Mutex Poisoned");
        if domains.contains_key(&payload.id){
            println!("Domain Already Exists");
            return Err(ErrorResponse { error: "Domain already exists".to_string() });
        }
       println!("Adding domain: {:?}", payload);
       domains.insert(payload.id,payload.clone());
       println!("Domain Added");
       Ok((StatusCode::CREATED, Json(payload)))
}
async fn get_domains(State(state):State<AppState>)->Json<Vec<Domain>>{
    let domains = state.domains.lock().expect("Mutex Poisoned");
    let domains_vec = domains.values().cloned().collect();
    //why cloned ??
    //.collect() -> consumes an iterator and  transforms it into a collection, like a Vec, HashMap, HashSet, etc.
    println!("Returning domains: {:?}", domains_vec);
    Json(domains_vec)
}
#[tokio::main]
async fn main() {
    let config = Config::from_env();
    let state = AppState{
        domains : Arc::new(Mutex::new(HashMap::new()))
    };
    let app = Router::new()
        .route("/domains",post(add_domains))
        .route("/domains/active", get(get_domains))
        .route("/health",get(check_health))
        .with_state(state);
    let addr : SocketAddr = format!("{}:{}",config.host,config.port)
        .parse()
        .expect("Invalid address");
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("Failed to bind address");
    println!("Server running on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
