use eframe::egui;

pub fn show_buy(_app: &mut crate::CsApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
                ui.heading("Buy Skins (placeholder)");
                ui.add_space(10.0);
                ui.label("This screen will show available skins to buy. (TODO)");
                ui.add_space(10.0);
                if ui.button("Back").clicked() {
                    _app.screen = crate::Screen::LoggedIn(_app.username.clone());
                }
            },
        );
    });
}
