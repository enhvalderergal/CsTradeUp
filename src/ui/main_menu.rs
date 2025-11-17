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
            ui.label(egui::RichText::new(&app.username).strong());
        });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
              

                // Main actions after login. The actual app :O
                let button_h = 44.0;
                // Slightly tighter vertical spacing between rows for a compact
                // game-like layout and remove the manual add_space that caused
                // an extra gap before the first row.
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 4.0);
                let btn_w = 160.0_f32;
                let gap = 8.0_f32;

                let labels = ["Buy", "Sell", "Tradeup", "Open Skins", "Inventory"];
                let avail_w = ui.available_size().x;

                // How many columns can we fit? Ensure at least 1.
                let mut columns = ((avail_w + gap) / (btn_w + gap)).floor() as usize;
                if columns == 0 {
                    columns = 1;
                }

                // Clamp columns to the number of buttons available
                columns = columns.min(labels.len());

                let mut idx = 0;
                while idx < labels.len() {
                    let remaining = labels.len() - idx;
                    let cols = columns.min(remaining);

                    let row_total = btn_w * (cols as f32) + gap * ((cols - 1) as f32);
                    let left_pad = ((avail_w - row_total) / 2.0).max(0.0);

                    ui.horizontal(|ui| {
                        ui.add_space(left_pad);
                        for j in 0..cols {
                            let label = labels[idx + j];
                            if ui.add_sized([btn_w, button_h], egui::Button::new(label)).clicked() {
                                match label {
                                    "Buy" => { app.screen = Screen::Buy; app.message.clear(); }
                                    "Sell" => { app.screen = Screen::Sell; app.message.clear(); }
                                    "Tradeup" => { app.screen = Screen::Tradeup; app.message.clear(); }
                                    "Open Skins" => { app.screen = Screen::OpenSkins; app.message.clear(); }
                                    "Inventory" => { app.screen = Screen::Inventory; app.message.clear(); }
                                    _ => {}
                                }
                            }
                            if j < cols - 1 {
                                ui.add_space(gap);
                            }
                        }
                    });

                    idx += cols;
                }

                ui.add_space(10.0);
                // Center Logout under the row
                let logout_w = 120.0;
                let logout_left = ((avail_w - logout_w) / 2.0).max(0.0);
                ui.horizontal(|ui| {
                    ui.add_space(logout_left);
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
