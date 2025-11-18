use eframe::egui;
use base64::Engine as _;

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

                        // Proper wrapping grid!
                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    if items.is_empty() {
                                        render_placeholder_grid(ui, 4, 3);
                                        return;
                                    }

                                    let cols = 4;
                                    let spacing = 8.0;

                                    for chunk in items.chunks(cols) {
                                        // A full row
                                        ui.horizontal(|ui| {
                                            ui.with_layout(
                                                egui::Layout::left_to_right(egui::Align::Center),
                                                |ui| {
                                                    let avail = ui.available_size();
                                                    let total_spacing = spacing * (chunk.len() as f32 - 1.0);
                                                    let slot_size = ((avail.x - total_spacing)
                                                        / chunk.len() as f32)
                                                        .clamp(48.0, 160.0);

                                                    for it in chunk {
                                                        let (rect, response) = ui
                                                            .allocate_exact_size(
                                                                egui::vec2(slot_size, slot_size),
                                                                egui::Sense::click(),
                                                            );
                                                        let painter = ui.painter_at(rect);

                                                        // Background tile
                                                        let rounding = egui::Rounding::same(8.0);
                                                        let bg = ui.visuals().widgets.inactive.bg_fill;
                                                        painter.rect_filled(rect, rounding, bg);

                                                        // Tile contents
                                                        ui.allocate_ui_at_rect(rect, |ui| {
                                                            ui.vertical_centered(|ui| {
                                                                let skin_name = it.skin.as_ref()
                                                                    .map(|s| s.name.as_str())
                                                                    .unwrap_or("Unknown");

                                                                let price = it.skin.as_ref()
                                                                    .map(|s| s.price)
                                                                    .unwrap_or(0.0);

                                                                ui.add_space(slot_size * 0.06);

                                                                // Image (decoded + cached)
                                                                if let Some(skin) = &it.skin {
                                                                    if let Some(b64) = &skin.image_base64 {
                                                                        let key = format!("skin-{}", skin.id);

                                                                        if let Some(info) =
                                                                            app.textures.get(&key)
                                                                        {
                                                                            let img_dim =
                                                                                slot_size * 0.6;
                                                                            let (w, h) = (
                                                                                info.size[0] as f32,
                                                                                info.size[1] as f32,
                                                                            );
                                                                            let (dw, dh) =
                                                                                if w > 0.0 && h > 0.0 {
                                                                                    let r = w / h;
                                                                                    if r >= 1.0 {
                                                                                        (img_dim, img_dim / r)
                                                                                    } else {
                                                                                        (img_dim * r, img_dim)
                                                                                    }
                                                                                } else {
                                                                                    (img_dim, img_dim)
                                                                                };
                                                                            ui.add(
                                                                                egui::Image::new((
                                                                                    info.handle.id(),
                                                                                    egui::vec2(dw, dh),
                                                                                )),
                                                                            );
                                                                        } else {
                                                                            // Decode on-demand
                                                                            let data = if b64.starts_with("data:") {
                                                                                match b64.find(',') {
                                                                                    Some(idx) => &b64[idx + 1..],
                                                                                    None => &b64,
                                                                                }
                                                                            } else {
                                                                                &b64
                                                                            };

                                                                            if let Ok(raw) = base64::engine::general_purpose::STANDARD.decode(data) {
                                                                                if let Ok(img) = image::load_from_memory(&raw) {
                                                                                    let img = img.to_rgba8();
                                                                                    let (w, h) = img.dimensions();
                                                                                    let size = [w as usize, h as usize];
                                                                                    let pixels = img.into_raw();
                                                                                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                                                                                    let handle = ctx.load_texture(
                                                                                        key.clone(),
                                                                                        color_image,
                                                                                        egui::TextureOptions::default(),
                                                                                    );
                                                                                    app.textures.insert(
                                                                                        key.clone(),
                                                                                        crate::TextureInfo {
                                                                                            handle: handle.clone(),
                                                                                            size,
                                                                                        },
                                                                                    );

                                                                                    let img_dim = slot_size * 0.6;
                                                                                    let (dw, dh) = {
                                                                                        let wf = size[0] as f32;
                                                                                        let hf = size[1] as f32;
                                                                                        let r = wf / hf;
                                                                                        if r >= 1.0 {
                                                                                            (img_dim, img_dim / r)
                                                                                        } else {
                                                                                            (img_dim * r, img_dim)
                                                                                        }
                                                                                    };
                                                                                    ui.add(
                                                                                        egui::Image::new((handle.id(), egui::vec2(dw, dh))),
                                                                                    );
                                                                                }
                                                                            }
                                                                        }
                                                                    } else {
                                                                        ui.label(egui::RichText::new(skin_name).strong());
                                                                    }
                                                                } else {
                                                                    ui.label(egui::RichText::new(skin_name).strong());
                                                                }

                                                                ui.add_space(slot_size * 0.04);

                                                                // Price + Sell button bottom bar
                                                                ui.horizontal(|ui| {
                                                                    ui.label(format!("${:.2}", price));
                                                                    ui.with_layout(
                                                                        egui::Layout::right_to_left(egui::Align::Center),
                                                                        |ui| {
                                                                            if ui.add_sized([64.0, 24.0], egui::Button::new("Sell")).clicked() {
                                                                                if let Some(uid) = app.current_user_id {
                                                                                    match crate::scripts::sell::sell_item(&app.db_path, uid, it.inventory.id, price) {
                                                                                        Ok(new_bal) => {
                                                                                            app.message = format!("Sold {} for ${:.2} — balance: ${:.2}",
                                                                                                it.skin.as_ref().map(|s| s.name.clone()).unwrap_or("Unknown".into()),
                                                                                                price, new_bal);
                                                                                        }
                                                                                        Err(e) => app.message = format!("Failed to sell item: {}", e),
                                                                                    }
                                                                                }
                                                                            }
                                                                        },
                                                                    );
                                                                });
                                                            });
                                                        });

                                                        ui.add_space(spacing);
                                                    }
                                                },
                                            );
                                        });
                                        ui.add_space(spacing);
                                    }
                                });
                            });
                    }

                    Err(e) => {
                        app.message = format!("Inventory load error: {}", e);
                        render_placeholder_grid(ui, 4, 3);
                    }
                }
            });
        });

    crate::ui::bottom_left_back(ctx, app, crate::Screen::LoggedIn(app.username.clone()));
}
