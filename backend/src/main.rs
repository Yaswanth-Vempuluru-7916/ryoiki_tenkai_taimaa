use std::{collections::HashMap, sync::{Arc, Mutex}};
use axum::{
    extract::State,
    http::StatusCode,
    routing::{post,get},
    Json, 
    Router};
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

async fn check_health() -> axum::Json<HealthResponse> {
    let health_status = String::from("I am healthier than you BROther");
    axum::Json(HealthResponse {
        status: health_status,
    })
}

async fn add_domains( state : State<AppState>,
    Json(payload) : Json<Domain>
   )
    ->Result<(StatusCode,Json<Domain>),StatusCode>{
        let domain = payload;
        println!("Adding domain: {:?}", domain);
       let  mut domains = state.domains.lock().unwrap();
       if domains.contains_key(&domain.id){
        println!("Domain Already Exists");
        return Err(StatusCode::CONFLICT);
       }
       domains.insert(domain.id,domain.clone());
       println!("Domain Added");
       Ok((StatusCode::CREATED, Json(domain)))
}
#[tokio::main]
async fn main() {
    let config = Config::from_env();
    let state = AppState{
        domains : Arc::new(Mutex::new(HashMap::new()))
    };
    let app = Router::new()
        .route("/domains",post(add_domains))
        .route("/health",get(check_health))
        .with_state(state);
    let addr : SocketAddr = format!("{}:{}",config.host,config.port)
        .parse()
        .expect("Invalid address");
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("Failed to bind address");
    println!("Server running on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
