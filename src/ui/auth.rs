use eframe::egui;
use crate::AuthMode;

pub fn show_auth(app: &mut crate::CsApp, ctx: &egui::Context, mode: AuthMode) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // Use available size to center manually both horizontally and vertically
        let avail_size = ui.available_size();

        // Layout constants
        let input_h = 30.0;
        let button_h = 44.0;
        let spacing = 8.0;
        let header_h = 36.0;
        let message_h = if app.message.is_empty() { 0.0 } else { 20.0 };

        // Total content height: header + inputs + buttons + spacing
        // Two inputs, two buttons, and spacing between elements
        let content_h = header_h + input_h * 2.0 + button_h * 2.0 + spacing * 5.0 + message_h;
        let top_padding = (avail_size.y - content_h).max(0.0) / 2.0;

        ui.add_space(top_padding);

        ui.vertical_centered(|ui| {
            // --- Header ---
            let header_text = match mode {
                AuthMode::Login => "Login",
                AuthMode::Register => "Register",
            };

            ui.label(
                egui::RichText::new(header_text)
                    .heading()
                    .strong()
                    .size(32.0),
            );

            ui.add_space(spacing * 2.0);

            // --- Inputs ---
            ui.label("Username");
            ui.add_sized([220.0, input_h], egui::TextEdit::singleline(&mut app.username));

            ui.add_space(spacing);

            ui.label("Password");
            ui.add_sized([220.0, input_h], egui::TextEdit::singleline(&mut app.password).password(true));

            ui.add_space(spacing * 1.5);

            // --- Buttons ---
            ui.spacing_mut().item_spacing = egui::vec2(0.0, spacing);

            if ui.add_sized([220.0, button_h], egui::Button::new("Submit")).clicked() {
                match mode {
                    AuthMode::Login => {
                        match crate::db::authenticate(&app.db_path, &app.username, &app.password) {
                            Ok(Some(user)) => {
                                // Save logged-in username to the app state so other screens can reference it
                                app.username = user.username.clone();
                                app.screen = crate::Screen::LoggedIn(user.username.clone());
                                app.message.clear();
                                app.password.clear();
                            }
                            Ok(None) => {
                                app.message = String::from("Invalid username or password");
                            }
                            Err(e) => {
                                app.message = format!("DB error: {}", e);
                            }
                        }
                    }
                    AuthMode::Register => {
                        match crate::db::create_user(&app.db_path, &app.username, &app.password) {
                            Ok(_user) => {
                                app.message = String::from("Registration successful. You can now log in.");
                                app.screen = crate::Screen::MainMenu;
                                app.password.clear();
                            }
                            Err(e) => {
                                app.message = format!("Failed to register: {}", e);
                            }
                        }
                    }
                }
            }

            if ui.add_sized([220.0, button_h], egui::Button::new("Back")).clicked() {
                app.screen = crate::Screen::MainMenu;
                app.message.clear();
                app.password.clear();
            }

            // --- Optional message ---
            if !app.message.is_empty() {
                ui.add_space(spacing * 2.0);
                ui.label(&app.message);
            }
        });
    });
}
