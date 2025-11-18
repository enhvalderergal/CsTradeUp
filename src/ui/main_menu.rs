use eframe::egui;
use crate::{AuthMode, Screen};

pub fn show_main_menu(app: &mut crate::CsApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // Use available size to center manually both horizontally and vertically
        let avail_size = ui.available_size();

        // Layout constants
        let button_h = 44.0;
        let spacing = 8.0;
        let header_h = 36.0;
        let message_h = if app.message.is_empty() { 0.0 } else { 20.0 };

        // Total content height (header + buttons + spacing + message)
        let content_h = header_h + button_h * 2.0 + spacing * 3.0 + message_h;
        let top_padding = (avail_size.y - content_h).max(0.0) / 2.0;

        ui.add_space(top_padding);

        // All content centered horizontally
        ui.vertical_centered(|ui| {
            // --- Header ---
            ui.label(
                egui::RichText::new("SKIN HUB")
                    .heading()
                    .strong()
                    .size(32.0)
            );

            ui.add_space(spacing * 2.0);

            // --- Buttons ---
            ui.spacing_mut().item_spacing = egui::vec2(0.0, spacing);

            if ui.add_sized([220.0, button_h], egui::Button::new("Login")).clicked() {
                app.screen = Screen::Auth(AuthMode::Login);
                app.message.clear();
            }

            if ui.add_sized([220.0, button_h], egui::Button::new("Register")).clicked() {
                app.screen = Screen::Auth(AuthMode::Register);
                app.message.clear();
            }

            // --- Optional message ---
            if !app.message.is_empty() {
                ui.add_space(spacing * 2.0);
                ui.label(&app.message);
            }
        });
    });
}


pub fn show_splash(_app: &mut crate::CsApp, ctx: &egui::Context) {
    // Forward to the dedicated splash animation module which keeps the
    // animation code separated from the main menu.
    crate::ui::splash::show_splash(_app, ctx);
}

pub fn show_logged_in(app: &mut crate::CsApp, ctx: &egui::Context, username: String) {
    // Top-left small user badge
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(egui::RichText::new("🔷").size(18.0));
            ui.add_space(6.0);
            // Show username and balance (if available)
            let mut label = app.username.clone();
            if let Some(uid) = app.current_user_id {
                if let Ok(Some(user)) = crate::db::get_user_by_id(&app.db_path, uid) {
                    label = format!("{} — ${:.2}", user.username, user.balance);
                }
            }
            ui.label(egui::RichText::new(label).strong());
        });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
              

                // Main actions after login: vertical stacked buttons with
                // separators between them to ensure consistent spacing.
                let button_h = 44.0;
                let btn_w = 220.0_f32;
                let spacing = 8.0_f32;

                ui.spacing_mut().item_spacing = egui::vec2(0.0, spacing);

                let labels = ["Buy", "Sell", "Tradeup", "Open Skins", "Inventory"];

                // Stack buttons vertically, centered horizontally. Show a
                // bold header above the buttons. Use consistent spacing but
                // no separator lines between buttons (user requested no lines).
                ui.vertical_centered(|ui| {
                    // Header for logged-in screen
                    ui.label(
                        egui::RichText::new("TRADE HUB")
                            .heading()
                            .strong()
                            .size(32.0),
                    );
                    ui.add_space(spacing * 1.25);

                    for (i, label) in labels.iter().enumerate() {
                        if ui.add_sized([btn_w, button_h], egui::Button::new(*label)).clicked() {
                            match *label {
                                "Buy" => { app.screen = Screen::Buy; app.message.clear(); }
                                "Sell" => { app.screen = Screen::Sell; app.message.clear(); }
                                "Tradeup" => { app.screen = Screen::Tradeup; app.message.clear(); }
                                "Open Skins" => { app.screen = Screen::OpenSkins; app.message.clear(); }
                                "Inventory" => { app.screen = Screen::Inventory; app.message.clear(); }
                                _ => {}
                            }
                        }

                        // Uniform spacing between buttons (no visual separator)
                        if i < labels.len() - 1 {
                            ui.add_space(spacing);
                        }
                    }

                    ui.add_space(spacing * 1.5);
                    // Center Logout under the stacked buttons
                    let logout_w = 140.0;
                    if ui.add_sized([logout_w, 32.0], egui::Button::new("Logout")).clicked() {
                        app.screen = Screen::MainMenu;
                        app.username.clear();
                        app.password.clear();
                        app.message.clear();
                    }
                });

            

                if !app.message.is_empty() {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.label(&app.message);
                }
            },
        );
    });
}
