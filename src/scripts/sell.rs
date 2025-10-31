use eframe::egui;

pub fn show_sell(_app: &mut crate::CsApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
                ui.heading("Sell Skins (placeholder)");
                ui.add_space(10.0);
                ui.label("This screen will let the user select skins to sell. (TODO)");
                ui.add_space(10.0);
                if ui.button("Back").clicked() {
                    _app.screen = crate::Screen::LoggedIn(_app.username.clone());
                }
            },
        );
    });
}
