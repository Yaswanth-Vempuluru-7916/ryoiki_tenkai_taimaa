use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Domain {
    pub id: i32,
    pub name: String,
    pub duration: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DomainStatus {
    pub id: i32,
    pub name: String,
    pub duration: i32,
    pub remaining_seconds: u64,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
}
