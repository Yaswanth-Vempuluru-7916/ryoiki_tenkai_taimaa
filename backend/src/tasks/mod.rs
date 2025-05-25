use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use tokio::time::interval;

use crate::{errors::AppError, models::Domain};

pub async fn cleanup_expired_domains(domains: Arc<Mutex<HashMap<i32, (Domain, Instant)>>>) {
    let mut interval = interval(Duration::from_secs(1)); // Use 1s for testing, revert to 5s for production
    loop {
        interval.tick().await;
        let mut domains = match domains
            .lock()
            .map_err(|_| AppError::Internal("Mutex Poisoned".into()))
        {
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
