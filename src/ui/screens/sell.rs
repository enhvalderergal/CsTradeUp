use eframe::egui;
use base64::Engine as _;

pub fn show_sell(app: &mut crate::CsApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Sell Skins");
        ui.add_space(8.0);

        if app.current_user_id.is_none() {
            ui.label("You must be logged in to sell items.");
            if ui.button("⬅ Back").clicked() {
                app.screen = crate::Screen::MainMenu;
            }
            return;
        }

        let user_id = app.current_user_id.unwrap();

        // List the user's owned skins
        match crate::db::get_inventory_for_user(&app.db_path, user_id) {
            Ok(items) => {
                if items.is_empty() {
                    ui.label("You have no items to sell.");
                } else {
                    ui.label("Select an item to sell (click Sell to accept offer):");
                    ui.add_space(8.0);

                    // Keep the original list layout but add an internal scrollbar like the Buy menu.
                    egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                        for it in items {
                            ui.horizontal(|ui| {
                                // Image area (small thumbnail)
                                if let Some(skin) = &it.skin {
                                    if let Some(b64) = &skin.image_base64 {
                                        let key = format!("skin-{}", skin.id);
                                        if let Some(info) = app.textures.get(&key) {
                                            let max_dim = 48.0_f32;
                                            let (w, h) = (info.size[0] as f32, info.size[1] as f32);
                                            let (dw, dh) = if w > 0.0 && h > 0.0 {
                                                let ratio = w / h;
                                                if ratio >= 1.0 { (max_dim, max_dim / ratio) } else { (max_dim * ratio, max_dim) }
                                            } else { (max_dim, max_dim) };
                                            ui.add(egui::Image::new((info.handle.id(), egui::vec2(dw, dh))));
                                        } else {
                                            let data = if b64.starts_with("data:") { match b64.find(',') { Some(idx) => &b64[idx + 1..], None => &b64 } } else { &b64 };
                                            if let Ok(raw) = base64::engine::general_purpose::STANDARD.decode(data) {
                                                if let Ok(img) = image::load_from_memory(&raw) {
                                                    let img = img.to_rgba8();
                                                    let (w, h) = img.dimensions();
                                                    let size = [w as usize, h as usize];
                                                    let pixels = img.into_raw();
                                                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                                                    let handle = ctx.load_texture(key.clone(), color_image, egui::TextureOptions::default());
                                                    app.textures.insert(key.clone(), crate::TextureInfo { handle: handle.clone(), size });
                                                    let max_dim = 48.0_f32;
                                                    let (dw, dh) = if size[0] > 0 && size[1] > 0 {
                                                        let aw = size[0] as f32; let ah = size[1] as f32; let ratio = aw / ah;
                                                        if ratio >= 1.0 { (max_dim, max_dim / ratio) } else { (max_dim * ratio, max_dim) }
                                                    } else { (max_dim, max_dim) };
                                                    ui.add(egui::Image::new((handle.id(), egui::vec2(dw, dh))));
                                                } else {
                                                    ui.label(&skin.name);
                                                }
                                            } else {
                                                ui.label(&skin.name);
                                            }
                                        }
                                    } else {
                                        ui.label(&skin.name);
                                    }
                                } else {
                                    ui.label("Unknown");
                                }

                                let name = it.skin.as_ref().map(|s| s.name.clone()).unwrap_or_else(|| "Unknown".into());
                                let price = it.skin.as_ref().map(|s| s.price).unwrap_or(0.0);
                                ui.label(format!("{} — ${:.2}", name, price));

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add_enabled(true, egui::Button::new("Sell")).clicked() {
                                        match crate::scripts::sell::sell_item(&app.db_path, user_id, it.inventory.id, price) {
                                            Ok(new_bal) => {
                                                app.message = format!("Sold {} for ${:.2} — balance: ${:.2}", name, price, new_bal);
                                            }
                                            Err(e) => {
                                                app.message = format!("Failed to sell item: {}", e);
                                            }
                                        }
                                    }
                                });
                            });
                            ui.add_space(8.0);
                        }
                    });
                }
            }
            Err(e) => {
                app.message = format!("Failed to load inventory: {}", e);
            }
        }

        ui.add_space(12.0);
        if ui.button("⬅ Back").clicked() {
            app.screen = crate::Screen::LoggedIn(app.username.clone());
        }
    });

    // Standard bottom-left back button
    crate::ui::bottom_left_back(ctx, app, crate::Screen::LoggedIn(app.username.clone()));
}
