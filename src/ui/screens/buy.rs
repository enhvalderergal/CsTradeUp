use eframe::egui;
use base64::Engine as _;

pub fn show_buy(app: &mut crate::CsApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Buy Skins");
        ui.add_space(6.0);

        // Balance
        if let Some(user_id) = app.current_user_id {
            if let Ok(Some(user)) = crate::db::get_user_by_id(&app.db_path, user_id) {
                ui.label(format!("Balance: ${:.2}", user.balance));
            }
        } else {
            ui.label("Not logged in.");
        }

        ui.separator();
        ui.add_space(6.0);

        // Load skins
        let skins = match crate::db::list_skins(&app.db_path) {
            Ok(s) => s,
            Err(e) => {
                ui.label(format!("Failed to load skins: {}", e));
                return;
            }
        };

        // FULL WIDTH SCROLL AREA
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                let available = ui.available_width();
                let tile_width = 150.0;
                let tile_height = 170.0;
                // reduce horizontal gap a bit to fit more cleanly
                let spacing = 6.0;

                // How many columns fit?
                // conservative column count so we don't overflow the right edge
                let cols = ((available + spacing) / (tile_width + spacing)).floor().max(1.0) as usize;

                // compute total grid width and left padding to center the grid and avoid touching the right edge
                let total_width = cols as f32 * tile_width + (cols.saturating_sub(1) as f32) * spacing;
                let left_pad = ((available - total_width) / 2.0).max(0.0);

                // Render as rows with fixed tile sizes so vertical spacing is consistent.
                let mut index = 0usize;
                while index < skins.len() {
                    ui.horizontal(|ui| {
                        // keep the grid centered and away from the right edge
                        if left_pad > 0.0 {
                            ui.add_space(left_pad);
                        }
                        for col in 0..cols {
                            if index >= skins.len() {
                                // placeholder to keep grid alignment and consistent row height
                                ui.allocate_exact_size(egui::vec2(tile_width, tile_height), egui::Sense::hover());
                            } else {
                                let skin = &skins[index];

                                // TILE BACKGROUND
                                let (rect, _resp) = ui.allocate_exact_size(egui::vec2(tile_width, tile_height), egui::Sense::click());
                                let painter = ui.painter();
                                painter.rect_filled(rect, egui::Rounding::same(8.0), ui.visuals().widgets.inactive.bg_fill);

                                ui.allocate_ui_at_rect(rect, |ui| {
                                    let mut img_h = 0.0_f32;
                                    ui.vertical_centered(|ui| {
                                        if let Some(b64) = &skin.image_base64 {
                                            let key = format!("skin-{}", skin.id);
                                            if let Some(info) = app.textures.get(&key) {
                                                let (w, h) = (info.size[0] as f32, info.size[1] as f32);
                                                let max_dim = 90.0_f32;
                                                let (dw, dh) = if w > 0.0 && h > 0.0 {
                                                    let ratio = w / h;
                                                    if ratio >= 1.0 { (max_dim, max_dim / ratio) } else { (max_dim * ratio, max_dim) }
                                                } else { (max_dim, max_dim) };
                                                img_h = dh;
                                                ui.add_space(6.0);
                                                ui.add(egui::Image::new((info.handle.id(), egui::vec2(dw, dh))));
                                            } else {
                                                let data = if b64.starts_with("data:") { b64.split(',').nth(1).unwrap_or("") } else { b64 };
                                                if let Ok(raw) = base64::engine::general_purpose::STANDARD.decode(data) {
                                                    if let Ok(img) = image::load_from_memory(&raw) {
                                                        let img = img.to_rgba8();
                                                        let (w, h) = img.dimensions();
                                                        let pixels = img.into_raw();
                                                        let color_image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &pixels);
                                                        let tex = ctx.load_texture(key.clone(), color_image, egui::TextureOptions::default());
                                                        app.textures.insert(key.clone(), crate::TextureInfo { handle: tex.clone(), size: [w as usize, h as usize] });
                                                        let max_dim = 90.0_f32;
                                                        let aw = w as f32; let ah = h as f32; let ratio = if ah>0.0 { aw/ah } else {1.0};
                                                        let (dw, dh) = if ratio >= 1.0 { (max_dim, max_dim / ratio) } else { (max_dim * ratio, max_dim) };
                                                        img_h = dh;
                                                        ui.add_space(6.0);
                                                        ui.add(egui::Image::new((tex.id(), egui::vec2(dw, dh))));
                                                    }
                                                }
                                            }
                                        }

                                        ui.add_space(4.0);
                                        ui.label(&skin.name);
                                        let rarity = skin.rarity.clone().unwrap_or_default();
                                        ui.small(crate::ui::rarity::rarity_richtext(&rarity));
                                    });

                                    // bottom bar: price left, Buy button right (smaller)
                                    let bottom_h = 28.0_f32;
                                    let spacer = (rect.height() - img_h - 48.0_f32).max(0.0);
                                    ui.add_space(spacer);
                                    ui.allocate_ui_at_rect(egui::Rect::from_min_max(egui::pos2(rect.left(), rect.bottom() - bottom_h), egui::pos2(rect.right(), rect.bottom())), |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("${:.2}", skin.price));
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.add_sized([64.0, 24.0], egui::Button::new("Buy")).clicked() {
                                                    if let Some(uid) = app.current_user_id {
                                                        match crate::scripts::buy::attempt_buy(&app.db_path, uid, skin.id, skin.price) {
                                                            Ok(_) => app.message = format!("Purchased {} for ${:.2}", skin.name, skin.price),
                                                            Err(e) => app.message = e,
                                                        }
                                                    } else {
                                                        app.message = "You must be logged in to buy skins".to_string();
                                                    }
                                                }
                                            });
                                        });
                                    });
                                });
                            }

                            // spacing between columns
                            if col != cols - 1 {
                                ui.add_space(spacing);
                            }

                            index += 1;
                        }
                    });

                    // spacing between rows
                    ui.add_space(spacing);
                }
            });

        ui.add_space(8.0);
    });

    // Standard bottom-left back button (consistent with other screens)
    crate::ui::bottom_left_back(ctx, app, crate::Screen::LoggedIn(app.username.clone()));
}
