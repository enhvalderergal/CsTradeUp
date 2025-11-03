use eframe::egui;

pub fn show_tradeup(app: &mut crate::CsApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
                ui.heading("(UI placeholder)");
                ui.add_space(8.0);
                ui.label("Implement things here here.");
                ui.add_space(12.0);
                if ui.button("Back").clicked() {
                    app.screen = crate::Screen::LoggedIn(app.username.clone());
                }
            },
        );
    });
}
