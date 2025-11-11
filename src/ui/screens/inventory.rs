use eframe::egui;

// Render fixed-size placeholder grid (scrollable container will host this)
fn render_placeholder_grid(ui: &mut egui::Ui, cols: usize, rows: usize) {
    let spacing = 8.0_f32;

    // Compute slot size so the grid fills available width
    let avail = ui.available_size();
    let total_h_spacing = spacing * (cols as f32 - 1.0);
    let slot = ((avail.x - total_h_spacing) / cols as f32).clamp(48.0, 160.0);

    // Center vertically if there's extra space
    let total_height = rows as f32 * slot + (rows as f32 - 1.0) * spacing;
    if avail.y > total_height {
        let top_pad = (avail.y - total_height) / 2.0;
        ui.add_space(top_pad);
    }

    for _r in 0..rows {
        ui.horizontal(|ui| {
            for c in 0..cols {
                ui.add_sized([slot, slot], egui::Button::new(" "));
                if c < cols - 1 {
                    ui.add_space(spacing);
                }
            }
        });
        ui.add_space(spacing);
    }
}

pub fn show_inventory(app: &mut crate::CsApp, ctx: &egui::Context) {
    egui::CentralPanel::default()
        .frame(egui::Frame {
            fill: ctx.style().visuals.panel_fill,
            inner_margin: egui::Margin::symmetric(16.0, 12.0),
            ..Default::default()
        })
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(egui::RichText::new("🎒 Inventory").size(26.0).strong());
                ui.add_space(12.0);

                if app.current_user_id.is_none() {
                    ui.label(
                        egui::RichText::new("You are not logged in.")
                            .italics()
                            .color(ui.visuals().weak_text_color()),
                    );
                    ui.add_space(8.0);
                    if ui.button("⬅ Back").clicked() {
                        app.screen = crate::Screen::MainMenu;
                    }
                    return;
                }

                let user_id = app.current_user_id.unwrap();

                match crate::scripts::inventory::list_inventory(&app.db_path, user_id) {
                    Ok(items) => {
                        ui.horizontal(|ui| {
                            if ui.button("🔄 Refresh").clicked() {
                                app.message = String::from("Refreshed inventory");
                            }
                        });

                        ui.add_space(8.0);

                        // Scrollable inventory
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                        if items.is_empty() {
                            render_placeholder_grid(ui, 4, 3);
                } else {
                    let cols = 4;
                    let spacing = 8.0;
                            for chunk in items.chunks(cols) {
                        ui.horizontal_centered(|ui| {
                            for it in chunk {
                                ui.group(|ui| {
                                            // Make item tiles use the same responsive slot sizing as placeholders
                                            let avail = ui.available_size();
                                            let total_h_spacing = spacing * (cols as f32 - 1.0);
                                            let slot_size = ((avail.x - total_h_spacing) / cols as f32).clamp(48.0, 160.0);
                                            ui.set_min_size(egui::vec2(slot_size, slot_size));
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new(&it.skin_name).strong());
                                        ui.label(format!("{} • ${:.2}", it.rarity, it.price));
                                        ui.horizontal(|ui| {
                                            if ui.button("−").clicked() {
                                                let new_q = it.quantity.saturating_sub(1);
                                                if new_q == 0 {
                                                    let _ = crate::scripts::inventory::remove_item(&app.db_path, it.id);
                                                    app.message = format!("Removed {}", it.skin_name);
                                                } else if let Err(e) = crate::scripts::inventory::set_quantity(&app.db_path, it.id, new_q) {
                                                    app.message = format!("Failed: {}", e);
                                                } else {
                                                    app.message = format!("Updated {} → {}", it.skin_name, new_q);
                                                }
                                            }
                                            ui.label(format!("{}", it.quantity));
                                            if ui.button("+").clicked() {
                                                if let Err(e) = crate::scripts::inventory::set_quantity(&app.db_path, it.id, it.quantity + 1) {
                                                    app.message = format!("Failed: {}", e);
                                                } else {
                                                    app.message = format!("Updated {} → {}", it.skin_name, it.quantity + 1);
                                                }
                                            }
                                        });
                                        if ui.button("❌ Remove").clicked() {
                                            if let Err(e) = crate::scripts::inventory::remove_item(&app.db_path, it.id) {
                                                app.message = format!("Failed to remove item: {}", e);
                                            } else {
                                                app.message = format!("Removed {}", it.skin_name);
                                            }
                                        }
                                    });
                                });
                                ui.add_space(spacing);
                            }
                        });
                        ui.add_space(spacing);
                    }
                }
            });
        });
});

                        // end Ok(items)
                    }
                    Err(e) => {
                        app.message = format!("Inventory load error: {}", e);
                        render_placeholder_grid(ui, 4, 3);
                    }
                }
            });
        });

    // Standard bottom-left back button for inventory
    crate::ui::bottom_left_back(ctx, app, crate::Screen::LoggedIn(app.username.clone()));
}
