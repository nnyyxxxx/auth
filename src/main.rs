use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Button, Entry, Label, Orientation};
use totp_rs::{Secret, TOTP};

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
        .default_width(300)
        .default_height(200)
        .build();

    let vbox = Box::new(Orientation::Vertical, 5);

    let secret_entry = Entry::new();
    secret_entry.set_placeholder_text(Some("Enter secret key"));
    vbox.append(&secret_entry);

    let generate_button = Button::with_label("Generate TOTP");
    vbox.append(&generate_button);

    let totp_label = Label::new(None);
    vbox.append(&totp_label);

    generate_button.connect_clicked(move |_| {
        let secret = secret_entry.text().to_string();
        if !secret.is_empty() {
            let totp = TOTP::new(
                totp_rs::Algorithm::SHA1,
                6,
                1,
                30,
                Secret::Raw(secret.into_bytes()).to_bytes().unwrap(),
            )
            .unwrap();
            let token = totp.generate_current().unwrap();
            totp_label.set_text(&token);
        }
    });

    window.set_child(Some(&vbox));
    window.present();
}
