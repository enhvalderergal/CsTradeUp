use eframe::egui;

pub fn show_sell(app: &mut crate::CsApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
                ui.heading("SELL SCREEN (UI placeholder)");
                ui.add_space(8.0);
                ui.label("This is the Sell screen placeholder. Implement selling flow here.");
                ui.add_space(12.0);
                if ui.button("Back").clicked() {
                    app.screen = crate::Screen::LoggedIn(app.username.clone());
                }
            },
        );
    });
}
