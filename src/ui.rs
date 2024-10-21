use crate::app_state::{AppState, TOTPEntry};
use crate::storage;
use crate::totp;
use gtk::{
    gdk::Display,
    glib::{self, clone},
    prelude::*,
    Application, ApplicationWindow, Box as GtkBox, Button, Entry, Label, ListBox, Orientation,
    ScrolledWindow,
};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

pub fn build_ui(app: &Application, state: Arc<Mutex<AppState>>) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Authenticator")
        .default_width(400)
        .default_height(600)
        .build();

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

    let editing = Arc::new(Mutex::new(HashSet::new()));

    let editing_clone_add = Arc::clone(&editing);
    let editing_clone_import = Arc::clone(&editing);
    let state_clone_update = Arc::clone(&state);
    let editing_clone_update = Arc::clone(&editing);
    let state_clone_timeout = Arc::clone(&state);
    let editing_clone_timeout = Arc::clone(&editing);

    let list_box_clone_add = list_box.clone();
    let state_clone_add = Arc::clone(&state);
    add_button.connect_clicked(move |_| {
        let name = name_entry.text().to_string();
        let secret = secret_entry.text().to_string().replace(" ", "");
        if !name.is_empty() && !secret.is_empty() {
            let mut state = state_clone_add.lock().unwrap();
            if !state.entries.contains_key(&name) {
                state.entries.insert(
                    name.clone(),
                    TOTPEntry {
                        name: name.clone(),
                        secret: secret.clone(),
                    },
                );
                if let Err(e) = storage::save_entries(&state.entries) {
                    eprintln!("Failed to save entries: {}", e);
                }
                name_entry.set_text("");
                secret_entry.set_text("");
                update_list_box(
                    &list_box_clone_add,
                    &state.entries,
                    Arc::clone(&state_clone_add),
                    &editing_clone_add,
                );
            } else {
                eprintln!("Entry with name '{}' already exists", name);
            }
        }
    });

    let window_clone_backup = window.clone();
    let state_clone_backup = Arc::clone(&state);
    backup_button.connect_clicked(move |_| {
        let file_chooser = gtk::FileChooserDialog::new(
            Some("Save Backup"),
            Some(&window_clone_backup),
            gtk::FileChooserAction::Save,
            &[
                ("Cancel", gtk::ResponseType::Cancel),
                ("Save", gtk::ResponseType::Accept),
            ],
        );
        let state_clone_inner = Arc::clone(&state_clone_backup);
        file_chooser.set_current_name("authenticator_backup.json");
        file_chooser.connect_response(move |dialog, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(path) = dialog.file().and_then(|f| f.path()) {
                    let state = state_clone_inner.lock().unwrap();
                    if let Err(e) = storage::backup_entries(&state.entries, path) {
                        eprintln!("Failed to backup entries: {}", e);
                    }
                }
            }
            dialog.close();
        });
        file_chooser.show();
    });

    let list_box_clone_import = list_box.clone();
    let state_clone_import = Arc::clone(&state);
    let window_clone_import = window.clone();
    import_button.connect_clicked(move |_| {
        let file_chooser = gtk::FileChooserDialog::new(
            Some("Import Backup"),
            Some(&window_clone_import),
            gtk::FileChooserAction::Open,
            &[
                ("Cancel", gtk::ResponseType::Cancel),
                ("Open", gtk::ResponseType::Accept),
            ],
        );
        let state_clone_inner = Arc::clone(&state_clone_import);
        let list_box_clone_inner = list_box_clone_import.clone();
        let editing_clone_inner = Arc::clone(&editing_clone_import);
        file_chooser.connect_response(move |dialog, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(path) = dialog.file().and_then(|f| f.path()) {
                    match storage::import_entries(path) {
                        Ok(imported_entries) => {
                            let mut state = state_clone_inner.lock().unwrap();
                            state.entries.extend(imported_entries);
                            update_list_box(
                                &list_box_clone_inner,
                                &state.entries,
                                Arc::clone(&state_clone_inner),
                                &editing_clone_inner,
                            );
                            if let Err(e) = storage::save_entries(&state.entries) {
                                eprintln!("Failed to save entries: {}", e);
                            }
                        }
                        Err(e) => eprintln!("Failed to import entries: {}", e),
                    }
                }
            }
            dialog.close();
        });
        file_chooser.show();
    });

    update_list_box(
        &list_box,
        &state_clone_update.lock().unwrap().entries,
        Arc::clone(&state_clone_update),
        &editing_clone_update,
    );

    let list_box_clone_timeout = list_box.clone();
    glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
        let state = state_clone_timeout.lock().unwrap();
        update_list_box(
            &list_box_clone_timeout,
            &state.entries,
            Arc::clone(&state_clone_timeout),
            &editing_clone_timeout,
        );
        glib::Continue(true)
    });

    window.set_child(Some(&main_box));
    window.present();
}

fn update_list_box(
    list_box: &ListBox,
    entries: &HashMap<String, TOTPEntry>,
    state: Arc<Mutex<AppState>>,
    editing: &Arc<Mutex<HashSet<String>>>,
) {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let remaining = 30 - (current_time % 30);

    let mut processed_entries = std::collections::HashSet::new();
    let editing_set = editing.lock().unwrap();

    for entry in entries.values() {
        if processed_entries.contains(&entry.name) || editing_set.contains(&entry.name) {
            continue;
        }
        processed_entries.insert(entry.name.clone());

        let row = if let Some(existing_row) = find_row_by_name(list_box, &entry.name) {
            existing_row
        } else {
            let new_row = GtkBox::new(Orientation::Horizontal, 5);
            new_row.set_hexpand(true);
            list_box.append(&new_row);
            new_row
        };

        if row.first_child().is_none() {
            let name_box = GtkBox::new(Orientation::Horizontal, 0);
            let name_label = Label::new(Some(&entry.name));
            name_box.append(&name_label);

            let token_label = Label::new(None);
            let remaining_label = Label::new(None);

            let spacer = GtkBox::new(Orientation::Horizontal, 0);
            spacer.set_hexpand(true);

            let edit_button = Button::with_label("Edit");
            edit_button.set_valign(gtk::Align::Center);

            let remove_button = Button::with_label("X");
            remove_button.set_valign(gtk::Align::Center);

            row.append(&name_box);
            row.append(&token_label);
            row.append(&remaining_label);
            row.append(&spacer);
            row.append(&edit_button);
            row.append(&remove_button);

            let state_clone = Arc::clone(&state);
            let entry_name = entry.name.clone();
            let list_box_clone = list_box.clone();
            let editing_clone = Arc::clone(&editing);
            remove_button.connect_clicked(move |_| {
                let mut state = state_clone.lock().unwrap();
                state.entries.remove(&entry_name);
                if let Err(e) = storage::save_entries(&state.entries) {
                    eprintln!("Failed to save entries: {}", e);
                }
                drop(state);
                editing_clone.lock().unwrap().remove(&entry_name);
                update_list_box(
                    &list_box_clone,
                    &state_clone.lock().unwrap().entries,
                    Arc::clone(&state_clone),
                    &editing_clone,
                );
            });

            let state_clone = Arc::clone(&state);
            let entry_name = entry.name.clone();
            let list_box_clone = list_box.clone();
            let name_box_clone = name_box.clone();
            let editing_clone = Arc::clone(&editing);
            edit_button.connect_clicked(move |_| {
                editing_clone.lock().unwrap().insert(entry_name.clone());
                let edit_entry = Entry::new();
                edit_entry.set_text(&entry_name);
                name_box_clone.remove(&name_label);
                name_box_clone.append(&edit_entry);

                let edit_entry_clone = edit_entry.clone();
                let editing_clone_inner = Arc::clone(&editing_clone);
                edit_entry.connect_activate(clone!(@strong state_clone, @strong list_box_clone, @strong entry_name, @strong name_box_clone, @strong edit_entry_clone => move |_| {
                    let new_name = edit_entry_clone.text().to_string();
                    if !new_name.is_empty() && new_name != entry_name {
                        let mut state = state_clone.lock().unwrap();
                        if let Some(entry) = state.entries.remove(&entry_name) {
                            let updated_entry = TOTPEntry { name: new_name.clone(), secret: entry.secret };
                            state.entries.insert(new_name.clone(), updated_entry);
                            if let Err(e) = storage::save_entries(&state.entries) {
                                eprintln!("Failed to save entries: {}", e);
                            }
                        }
                        drop(state);

                        name_box_clone.remove(&edit_entry_clone);
                        let new_label = Label::new(Some(&new_name));
                        name_box_clone.append(&new_label);
                    } else {
                        name_box_clone.remove(&edit_entry_clone);
                        let original_label = Label::new(Some(&entry_name));
                        name_box_clone.append(&original_label);
                    }
                    editing_clone_inner.lock().unwrap().remove(&entry_name);
                    update_list_box(&list_box_clone, &state_clone.lock().unwrap().entries, Arc::clone(&state_clone), &editing_clone_inner);
                }));

                edit_entry.grab_focus();
            });

            let gesture = gtk::GestureClick::new();
            gesture.set_button(1);
            gesture.connect_released(
                clone!(@strong entry => move |_, _, _, _| {
                    if let Some(display) = Display::default() {
                        let clipboard = display.clipboard();
                        let current_time = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        let current_token = totp::generate_totp(&entry.secret, current_time).unwrap_or_else(|e| {
                            eprintln!("Error generating TOTP for {}: {}", entry.name, e);
                            "Error".to_string()
                        });
                        clipboard.set_text(&current_token);
                    }
                }),
            );
            row.add_controller(&gesture);
        }

        let current_token = match totp::generate_totp(&entry.secret, current_time) {
            Ok(token) => token,
            Err(e) => {
                eprintln!("Error generating TOTP for {}: {}", entry.name, e);
                "Invalid".to_string()
            }
        };

        if let Some(token_label) = row.first_child().and_then(|c| c.next_sibling()) {
            if let Ok(label) = token_label.downcast::<Label>() {
                label.set_label(&current_token);
            }
        }

        if let Some(remaining_label) = row
            .first_child()
            .and_then(|c| c.next_sibling())
            .and_then(|c| c.next_sibling())
        {
            if let Ok(label) = remaining_label.downcast::<Label>() {
                label.set_label(&format!("{}s", remaining));
            }
        }
    }

    remove_unused_rows(list_box, &processed_entries);
}

fn find_row_by_name(list_box: &ListBox, name: &str) -> Option<GtkBox> {
    let mut index = 0;
    while let Some(row) = list_box.row_at_index(index) {
        if let Some(box_widget) = row.child() {
            if let Some(label_widget) = box_widget.first_child().and_then(|c| c.first_child()) {
                if let Some(label) = label_widget.downcast_ref::<gtk::Label>() {
                    if label.text() == name {
                        return box_widget.downcast::<GtkBox>().ok();
                    }
                }
            }
        }
        index += 1;
    }
    None
}

fn remove_unused_rows(list_box: &ListBox, processed_entries: &HashSet<String>) {
    let mut index = 0;
    while let Some(row) = list_box.row_at_index(index) {
        if let Some(box_widget) = row.child() {
            if let Some(label_widget) = box_widget.first_child().and_then(|c| c.first_child()) {
                if let Some(label) = label_widget.downcast_ref::<gtk::Label>() {
                    if !processed_entries.contains(&label.text().to_string()) {
                        list_box.remove(&row);
                        continue;
                    }
                }
            }
        }
        index += 1;
    }
}
