use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::models::Domain;
#[derive(Clone, Debug)]
pub struct AppState {
    pub domains: Arc<Mutex<HashMap<i32, (Domain, Instant)>>>,
}
