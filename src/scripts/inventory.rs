use eframe::egui;

pub fn show_inventory(_app: &mut crate::CsApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
                ui.heading("Inventory (placeholder)");
                ui.add_space(10.0);
                ui.label("This screen will contain the user's inventory grid. (TODO)");
                ui.add_space(10.0);

                // Simple placeholder grid so the inventory screen demonstrates the layout
                let cols = 6;
                let rows = 3;

                egui::Grid::new("inventory_screen_grid").spacing(egui::vec2(8.0, 8.0)).show(ui, |ui| {
                    for _r in 0..rows {
                        for _c in 0..cols {
                            if ui.add_sized([90.0, 90.0], egui::Button::new("Empty")).clicked() {
                                _app.message = String::from("Clicked inventory slot (placeholder)");
                            }
                        }
                        ui.end_row();
                    }
                });

                ui.add_space(12.0);
                if ui.button("Back").clicked() {
                    _app.screen = crate::Screen::LoggedIn(_app.username.clone());
                }
            },
        );
    });
}
