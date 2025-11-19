use eframe::egui;
use base64::Engine as _;

pub fn show_buy(app: &mut crate::CsApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Buy Skins");
        ui.add_space(6.0);

        if let Some(user_id) = app.current_user_id {
            if let Ok(Some(user)) = crate::db::get_user_by_id(&app.db_path, user_id) {
                ui.label(format!("Balance: ${:.2}", user.balance));
            }
        }

        ui.separator();
        ui.add_space(6.0);

        let skins = match crate::db::list_skins(&app.db_path) {
            Ok(s) => s,
            Err(e) => {
                ui.label(format!("Failed to load skins: {}", e));
                return;
            }
        };

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                let viewport_width = ui.max_rect().width(); // ← this is the FIX
                let tile_w = 155.0;
                let tile_h = 200.0;
                let spacing = 10.0;

                let cols = ((viewport_width + spacing) / (tile_w + spacing))
                    .floor()
                    .max(1.0) as usize;

                ui.style_mut().spacing.item_spacing = egui::vec2(spacing, spacing);

                let mut row = 0usize;
                while row * cols < skins.len() {
                    ui.horizontal(|ui| {
                        for col in 0..cols {
                            let index = row * cols + col;
                            if index >= skins.len() {
                                break; // ← no broken placeholders
                            }

                            let skin = &skins[index];

                            egui::Frame::group(ui.style())
                                .rounding(8.0)
                                .show(ui, |ui| {
                                    ui.set_width(tile_w);
                                    ui.set_height(tile_h);

                                    ui.vertical_centered(|ui| {
                                        ui.add_space(4.0);

                                        // IMAGE
                                        if let Some(b64) = &skin.image_base64 {
                                            let key = format!("skin-{}", skin.id);

                                            if let Some(info) = app.textures.get(&key) {
                                                let (w, h) = (info.size[0] as f32, info.size[1] as f32);
                                                let max = 95.0;
                                                let ratio = w / h;
                                                let (dw, dh) = if ratio >= 1.0 {
                                                    (max, max / ratio)
                                                } else {
                                                    (max * ratio, max)
                                                };

                                                ui.add(egui::Image::new(
                                                    (info.handle.id(), egui::vec2(dw, dh))
                                                ));
                                            } else {
                                                let pure = if b64.starts_with("data:") {
                                                    b64.split(',').nth(1).unwrap_or("")
                                                } else {
                                                    b64
                                                };

                                                if let Ok(raw) =
                                                    base64::engine::general_purpose::STANDARD
                                                        .decode(pure)
                                                {
                                                    if let Ok(img) = image::load_from_memory(&raw) {
                                                        let img = img.to_rgba8();
                                                        let (w, h) = img.dimensions();
                                                        let pixels = img.into_raw();

                                                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                                            [w as usize, h as usize],
                                                            &pixels,
                                                        );

                                                        let tex = ctx.load_texture(
                                                            key.clone(),
                                                            color_image,
                                                            egui::TextureOptions::default(),
                                                        );

                                                        app.textures.insert(
                                                            key.clone(),
                                                            crate::TextureInfo {
                                                                handle: tex.clone(),
                                                                size: [w as usize, h as usize],
                                                            },
                                                        );

                                                        let max = 95.0;
                                                        let ratio = w as f32 / h as f32;
                                                        let (dw, dh) = if ratio >= 1.0 {
                                                            (max, max / ratio)
                                                        } else {
                                                            (max * ratio, max)
                                                        };

                                                        ui.add(egui::Image::new(
                                                            (tex.id(), egui::vec2(dw, dh)),
                                                        ));
                                                    }
                                                }
                                            }
                                        }

                                        ui.add_space(4.0);
                                        ui.label(&skin.name);

                                        if let Some(rarity) = skin.rarity.clone() {
                                            ui.small(crate::ui::rarity::rarity_richtext(&rarity));
                                        }

                                        ui.with_layout(
                                            egui::Layout::bottom_up(egui::Align::Center),
                                            |ui| {
                                                if ui.button(format!("Buy  ${:.2}", skin.price)).clicked() {
                                                    if let Some(uid) = app.current_user_id {
                                                        match crate::scripts::buy::attempt_buy(
                                                            &app.db_path,
                                                            uid,
                                                            skin.id,
                                                            skin.price,
                                                        ) {
                                                            Ok(_) => {
                                                                app.message = format!(
                                                                    "Purchased {} for ${:.2}",
                                                                    skin.name, skin.price
                                                                )
                                                            }
                                                            Err(e) => app.message = e,
                                                        }
                                                    }
                                                }
                                            },
                                        );
                                    });
                                });
                        }
                    });
                    row += 1;
                }
            });

        ui.add_space(8.0);
    });

    crate::ui::bottom_left_back(ctx, app, crate::Screen::LoggedIn(app.username.clone()));
}
