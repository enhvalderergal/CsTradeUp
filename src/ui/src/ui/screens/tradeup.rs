use eframe::egui;

pub fn show_tradeup(app: &mut crate::CsApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
                ui.heading("TRADEUP SCREEN (UI placeholder)");
                ui.add_space(8.0);
                ui.label("This is the Tradeup screen placeholder. Implement tradeup composition here.");
                ui.add_space(12.0);
            },
        );
    });

    // Standard bottom-left back button
    crate::ui::bottom_left_back(ctx, app, crate::Screen::LoggedIn(app.username.clone()));
}
