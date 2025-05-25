use std::time::{Duration, Instant};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::{
    errors::AppError,
    models::{Domain, DomainStatus, HealthResponse},
    state::AppState,
};

pub async fn check_health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "I am healthier than you BROther".to_string(),
    })
}

pub async fn add_domains(
    State(state): State<AppState>,
    Json(payload): Json<Domain>,
) -> Result<(StatusCode, Json<Domain>), AppError> {
    let mut domains = state
        .domains
        .lock()
        .map_err(|_| AppError::Internal("Mutex Poisoned".into()))?;
    println!("Adding domain: {:?}", payload);
    if domains.contains_key(&payload.id) {
        return Err(AppError::Conflict("Domain already exists".to_string()));
    }
    domains.insert(payload.id, (payload.clone(), Instant::now()));
    println!("Domain added");
    Ok((StatusCode::CREATED, Json(payload)))
}

pub async fn get_domains(State(state): State<AppState>) -> Result<Json<Vec<Domain>>, AppError> {
    let domains = state
        .domains
        .lock()
        .map_err(|_| AppError::Internal("Mutex Poisoned".into()))?;
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

pub async fn get_domain(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<(StatusCode, Json<DomainStatus>), AppError> {
    let domains = state
        .domains
        .lock()
        .map_err(|_| AppError::Internal("Mutex Poisoned".into()))?;

    if let Some((domain, instant)) = domains.get(&id) {
        let now = Instant::now();

        if now.duration_since(*instant) >= Duration::from_secs(domain.duration as u64) {
            return Err(AppError::NotFound(format!("Domain ID {} expired", id)));
        }

        let expires_at = *instant + Duration::from_secs(domain.duration as u64);
        let remaining_duration = expires_at.saturating_duration_since(now);

        let domain_status = DomainStatus {
            id: domain.id,
            name: domain.name.clone(),
            duration: domain.duration,
            remaining_seconds: remaining_duration.as_secs(),
        };

        Ok((StatusCode::OK, Json(domain_status)))
    } else {
        Err(AppError::NotFound(format!("Domain ID {} not found", id)))
    }
}
