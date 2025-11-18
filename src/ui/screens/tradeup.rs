use eframe::egui;

pub fn show_tradeup(app: &mut crate::CsApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Trade Up");
            ui.add_space(8.0);

            if app.current_user_id.is_none() {
                ui.label("You must be logged in to perform tradeups.");
                if ui.button("⬅ Back").clicked() {
                    app.screen = crate::Screen::MainMenu;
                }
                return;
            }

            let user_id = app.current_user_id.unwrap();

            // Load inventory
            let items = match crate::db::get_inventory_for_user(&app.db_path, user_id) {
                Ok(it) => it,
                Err(e) => { ui.label(format!("Failed to load inventory: {}", e)); return; }
            };

            // Precompute selection validity so we can keep the Trade Up button visible
            let selection_count = app.tradeup_selection.len();
            // Determine if selection is exactly 10 and all same rarity
            let mut rar: Option<String> = None;
            let mut valid = true;
            if selection_count != 10 {
                valid = false;
            } else {
                for id in &app.tradeup_selection {
                    if let Some(entry) = items.iter().find(|e| e.inventory.id == *id) {
                        if let Some(s) = &entry.skin {
                            if let Some(r0) = &rar {
                                if r0.to_lowercase() != s.rarity.clone().unwrap_or_default().to_lowercase() {
                                    valid = false;
                                    break;
                                }
                            } else {
                                rar = Some(s.rarity.clone().unwrap_or_default());
                            }
                        } else { valid = false; break; }
                    } else { valid = false; break; }
                }
            }

            ui.horizontal(|ui| {
                if ui.button("Clear Selection").clicked() {
                    app.tradeup_selection.clear();
                }
                ui.add_space(8.0);
                ui.label(format!("Selected: {} (need 10)", selection_count));

                ui.add_space(12.0);
                if ui.add_enabled(valid, egui::Button::new("Trade Up")).clicked() {
                    let ids = app.tradeup_selection.clone();
                    match crate::scripts::tradeup::compose_tradeup(&app.db_path, user_id, ids) {
                        Ok(new_id) => {
                            app.message = format!("Tradeup succeeded: new inventory id {}", new_id);
                            app.tradeup_selection.clear();
                        }
                        Err(e) => app.message = format!("Tradeup failed: {}", e),
                    }
                }
            });

            ui.add_space(6.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                for it in items.iter() {
                    ui.horizontal(|ui| {
                        let id = it.inventory.id;
                        let mut selected = app.tradeup_selection.contains(&id);
                        if ui.checkbox(&mut selected, "").changed() {
                            if selected {
                                if !app.tradeup_selection.contains(&id) {
                                    app.tradeup_selection.push(id);
                                }
                            } else {
                                app.tradeup_selection.retain(|&x| x != id);
                            }
                        }

                        // thumbnail small
                            if let Some(skin) = &it.skin {
                                let key = format!("skin-{}", skin.id);
                                if let Some(info) = app.textures.get(&key) {
                                    let max_dim = 40.0_f32;
                                    let (w, h) = (info.size[0] as f32, info.size[1] as f32);
                                    let (dw, dh) = if w > 0.0 && h > 0.0 {
                                        let ratio = w / h;
                                        if ratio >= 1.0 { (max_dim, max_dim / ratio) } else { (max_dim * ratio, max_dim) }
                                    } else { (max_dim, max_dim) };
                                    ui.add(egui::Image::new((info.handle.id(), egui::vec2(dw, dh))));
                                } else {
                                    ui.label(&skin.name);
                                }
                                ui.vertical(|ui| {
                                    ui.label(&skin.name);
                                    let r = skin.rarity.clone().unwrap_or_default();
                                    ui.small(crate::ui::rarity::rarity_richtext(&r));
                                });
                            } else {
                                ui.label(format!("inv {} — Unknown", id));
                            }
                    });
                    ui.add_space(6.0);
                }
            });

            ui.add_space(8.0);

            // Show message about validity below the list
            if !valid {
                ui.colored_label(egui::Color32::YELLOW, "Selected items must be valid and of the same rarity to enable Trade Up.");
            } else if let Some(rstr) = &rar {
                ui.label(format!("Ready to trade up 10 items of rarity: {}", rstr));
            }

            ui.add_space(12.0);
            if ui.button("⬅ Back").clicked() {
                app.screen = crate::Screen::LoggedIn(app.username.clone());
            }
        });
    });

    // Standard bottom-left back button
    crate::ui::bottom_left_back(ctx, app, crate::Screen::LoggedIn(app.username.clone()));
}
