mod app_state;
mod storage;
mod totp;
mod ui;

use app_state::AppState;
use gtk::prelude::*;
use gtk::Application;
use std::sync::{Arc, Mutex};

fn main() {
    let app = Application::builder()
        .application_id("nnyyxxxx.auth")
        .build();

    let state = Arc::new(Mutex::new(AppState::new()));

    app.connect_activate(move |app| {
        ui::build_ui(app, Arc::clone(&state));
    });

    app.run();
}
