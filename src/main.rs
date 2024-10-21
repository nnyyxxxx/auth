use gtk::gdk::Display;
use gtk::glib::clone;
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

    let button_box = GtkBox::new(Orientation::Horizontal, 5);
    button_box.set_halign(gtk::Align::End);
    button_box.set_margin_top(10);

    let import_button = Button::with_label("Import");
    let backup_button = Button::with_label("Backup");
    button_box.append(&import_button);
    button_box.append(&backup_button);
    main_box.append(&button_box);

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
            update_list_box(&list_box_clone, &state.entries, Arc::clone(&state_clone));
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
    let window_clone = window.clone();
    let list_box_clone = list_box.clone();
    import_button.connect_clicked(move |_| {
        let file_chooser = gtk::FileChooserDialog::new(
            Some("Import Backup"),
            Some(&window_clone),
            gtk::FileChooserAction::Open,
            &[
                ("Cancel", gtk::ResponseType::Cancel),
                ("Open", gtk::ResponseType::Accept),
            ],
        );
        let state_clone = Arc::clone(&state_clone);
        let list_box_clone = list_box_clone.clone();
        file_chooser.connect_response(move |dialog, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(path) = dialog.file().and_then(|f| f.path()) {
                    match import_entries(path) {
                        Ok(imported_entries) => {
                            let mut state = state_clone.lock().unwrap();
                            state.entries.extend(imported_entries);
                            save_entries(&state.entries).unwrap();
                            update_list_box(
                                &list_box_clone,
                                &state.entries,
                                Arc::clone(&state_clone),
                            );
                        }
                        Err(e) => eprintln!("Failed to import entries: {}", e),
                    }
                }
            }
            dialog.close();
        });
        file_chooser.show();
    });

    let state_clone = Arc::clone(&state);
    update_list_box(
        &list_box,
        &state_clone.lock().unwrap().entries,
        Arc::clone(&state_clone),
    );

    let state_clone = Arc::clone(&state);
    let list_box_clone = list_box.clone();
    glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
        let state = state_clone.lock().unwrap();
        update_list_box(&list_box_clone, &state.entries, Arc::clone(&state_clone));
        Continue(true)
    });

    window.set_child(Some(&main_box));
    window.present();
}

fn update_list_box(
    list_box: &ListBox,
    entries: &HashMap<String, TOTPEntry>,
    state: Arc<Mutex<AppState>>,
) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }
    for entry in entries.values() {
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

        let current_token = totp.generate_current().unwrap();
        let next_token = totp.generate(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                / 30
                + 1,
        );

        let remaining = 30
            - (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                % 30);

        let row = GtkBox::new(Orientation::Horizontal, 5);
        row.set_hexpand(true);

        let name_label = Label::new(Some(&entry.name));
        let token_label = Label::new(Some(&current_token));
        let remaining_label = Label::new(Some(&format!("{}s", remaining)));

        let spacer = GtkBox::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);

        let remove_button = Button::with_label("X");
        remove_button.set_valign(gtk::Align::Center);

        let state_clone = Arc::clone(&state);
        let entry_name = entry.name.clone();
        let list_box_clone = list_box.clone();
        remove_button.connect_clicked(move |_| {
            let mut state = state_clone.lock().unwrap();
            state.entries.remove(&entry_name);
            save_entries(&state.entries).unwrap();
            update_list_box(&list_box_clone, &state.entries, Arc::clone(&state_clone));
        });

        row.append(&name_label);
        row.append(&token_label);
        row.append(&remaining_label);
        row.append(&spacer);
        row.append(&remove_button);

        let gesture = gtk::GestureClick::new();
        gesture.set_button(1);
        gesture.connect_released(
            clone!(@strong current_token, @strong next_token => move |_, _, _, _| {
                if let Some(display) = Display::default() {
                    let clipboard = display.clipboard();
                    clipboard.set_text(&format!("{} {}", current_token, next_token));
                }
            }),
        );
        row.add_controller(&gesture);

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

fn import_entries(
    path: std::path::PathBuf,
) -> Result<HashMap<String, TOTPEntry>, Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data)?)
}
