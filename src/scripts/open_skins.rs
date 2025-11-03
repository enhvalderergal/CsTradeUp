use eframe::egui;

pub fn show_open_skins(_app: &mut crate::CsApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
                ui.heading("Open Skins (placeholder)");
                ui.add_space(10.0);
                ui.label("This screen will handle opening skin cases. (TODO)");
                ui.add_space(10.0);
                if ui.button("Back").clicked() {
                    _app.screen = crate::Screen::LoggedIn(_app.username.clone());
                }
            },
        );
    });
}
