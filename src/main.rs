use glib::Continue;
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Button, Entry, Label, ListBox, Orientation,
    ScrolledWindow,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use totp_rs::{Secret, TOTP};

#[derive(Serialize, Deserialize, Clone)]
struct TOTPEntry {
    name: String,
    secret: String,
}

struct AppState {
    entries: HashMap<String, TOTPEntry>,
}

fn main() {
    let app = Application::builder()
        .application_id("nnyyxxxx.auth")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Authenticator")
        .default_width(400)
        .default_height(600)
        .build();

    let state = Arc::new(Mutex::new(AppState {
        entries: load_entries().unwrap_or_default(),
    }));

    let main_box = GtkBox::new(Orientation::Vertical, 10);
    main_box.set_margin_top(10);
    main_box.set_margin_bottom(10);
    main_box.set_margin_start(10);
    main_box.set_margin_end(10);

    let input_box = GtkBox::new(Orientation::Horizontal, 5);
    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some("Name"));
    name_entry.set_hexpand(true);
    let secret_entry = Entry::new();
    secret_entry.set_placeholder_text(Some("Secret"));
    secret_entry.set_hexpand(true);
    let add_button = Button::with_label("Add");
    input_box.append(&name_entry);
    input_box.append(&secret_entry);
    input_box.append(&add_button);
    main_box.append(&input_box);

    let scroll = ScrolledWindow::new();
    scroll.set_vexpand(true);
    let list_box = ListBox::new();
    list_box.set_selection_mode(gtk::SelectionMode::None);
    scroll.set_child(Some(&list_box));
    main_box.append(&scroll);

    let backup_button = Button::with_label("Backup");
    backup_button.set_halign(gtk::Align::End);
    backup_button.set_margin_top(10);
    main_box.append(&backup_button);

    let state_clone = Arc::clone(&state);
    let list_box_clone = list_box.clone();
    add_button.connect_clicked(move |_| {
        let name = name_entry.text().to_string();
        let secret = secret_entry.text().to_string();
        if !name.is_empty() && !secret.is_empty() {
            let mut state = state_clone.lock().unwrap();
            state.entries.insert(
                name.clone(),
                TOTPEntry {
                    name: name.clone(),
                    secret,
                },
            );
            save_entries(&state.entries).unwrap();
            name_entry.set_text("");
            secret_entry.set_text("");
            update_list_box(&list_box_clone, &state.entries);
        }
    });

    let state_clone = Arc::clone(&state);
    let window_clone = window.clone();
    backup_button.connect_clicked(move |_| {
        let file_chooser = gtk::FileChooserDialog::new(
            Some("Save Backup"),
            Some(&window_clone),
            gtk::FileChooserAction::Save,
            &[
                ("Cancel", gtk::ResponseType::Cancel),
                ("Save", gtk::ResponseType::Accept),
            ],
        );
        file_chooser.set_current_name("authenticator_backup.json");
        let state_clone = Arc::clone(&state_clone);
        file_chooser.connect_response(move |dialog, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(path) = dialog.file().and_then(|f| f.path()) {
                    let state = state_clone.lock().unwrap();
                    if let Err(e) = backup_entries(&state.entries, path) {
                        eprintln!("Failed to backup entries: {}", e);
                    }
                }
            }
            dialog.close();
        });
        file_chooser.show();
    });

    let state_clone = Arc::clone(&state);
    update_list_box(&list_box, &state_clone.lock().unwrap().entries);

    let state_clone = Arc::clone(&state);
    let list_box_clone = list_box.clone();
    glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
        let state = state_clone.lock().unwrap();
        update_list_box(&list_box_clone, &state.entries);
        Continue(true)
    });

    window.set_child(Some(&main_box));
    window.present();
}

fn update_list_box(list_box: &ListBox, entries: &HashMap<String, TOTPEntry>) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }
    for (_, entry) in entries {
        let totp = TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Raw(entry.secret.clone().into_bytes())
                .to_bytes()
                .unwrap(),
        )
        .unwrap();
        let token = totp.generate_current().unwrap();
        let remaining = 30
            - (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                % 30);
        let row = GtkBox::new(Orientation::Horizontal, 5);
        row.append(&Label::new(Some(&entry.name)));
        row.append(&Label::new(Some(&token)));
        row.append(&Label::new(Some(&format!("{}s", remaining))));
        list_box.append(&row);
    }
}

fn load_entries() -> Result<HashMap<String, TOTPEntry>, Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string("entries.json")?;
    Ok(serde_json::from_str(&data)?)
}

fn save_entries(entries: &HashMap<String, TOTPEntry>) -> Result<(), Box<dyn std::error::Error>> {
    let data = serde_json::to_string(entries)?;
    std::fs::write("entries.json", data)?;
    Ok(())
}

fn backup_entries(
    entries: &HashMap<String, TOTPEntry>,
    path: std::path::PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let data = serde_json::to_string(entries)?;
    std::fs::write(path, data)?;
    Ok(())
}
