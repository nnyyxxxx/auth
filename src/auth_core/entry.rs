use crate::{
    auth_core::totp::generate_totp,
    utils::constants::{INVALID_LABEL, INVALID_REMAINING_TIME},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Entries {
    pub entries: Vec<Entry>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Entry {
    pub name: String,
    pub secret: String,
}

impl Entry {
    pub fn generate_totp_with_time(&self) -> (String, u64) {
        match generate_totp(&self.secret) {
            Ok(result) => result,
            Err(_) => (INVALID_LABEL.to_string(), INVALID_REMAINING_TIME),
        }
    }
}
