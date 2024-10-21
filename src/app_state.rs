use crate::storage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct TOTPEntry {
    pub name: String,
    pub secret: String,
}

pub struct AppState {
    pub entries: HashMap<String, TOTPEntry>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            entries: storage::load_entries().unwrap_or_default(),
        }
    }
}
