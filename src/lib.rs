pub mod auth_core {
    pub mod app;
    pub mod crypto;
    pub mod entry;
    pub mod totp;
}

pub mod utils {
    pub mod cli;
    pub mod clipboard;
    pub mod command;
    pub mod constants;
    pub mod error;
}

pub mod input {
    pub mod event;
    pub mod linux;
    pub mod mouse;
    pub mod root;
}

pub mod ui {
    pub mod core;
    pub mod layout;
    pub mod notification;
    pub mod size;
}

pub use auth_core::app::App;
pub use utils::error::{AuthError, AuthResult};
