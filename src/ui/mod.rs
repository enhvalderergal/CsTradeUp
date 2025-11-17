pub mod main_menu;
pub mod auth;
pub mod splash;
pub mod screens;

use eframe::egui;

/// Render a standard bottom-left Back button that navigates to `target` when clicked.
/// Screens can call this after drawing their central content to ensure consistent layout.
pub fn bottom_left_back(ctx: &egui::Context, app: &mut crate::CsApp, target: crate::Screen) {
	egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
		ui.horizontal(|ui| {
			if ui.button("⬅ Back").clicked() {
				app.screen = target;
			}
			// fill the rest of the bar so the button stays at the left
			ui.add_space(ui.available_size().x);
		});
	});
}
