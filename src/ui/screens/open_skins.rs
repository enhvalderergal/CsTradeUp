use eframe::egui;
use std::time::Instant;
use base64::Engine as _;

const CASE_COST: f64 = 5.0;

pub fn show_open_skins(app: &mut crate::CsApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Open Case");
            ui.add_space(8.0);
            // Show current balance
            if let Some(user_id) = app.current_user_id {
                if let Ok(Some(user)) = crate::db::get_user_by_id(&app.db_path, user_id) {
                    ui.label(format!("Balance: ${:.2}", user.balance));
                    ui.add_space(6.0);
                }
            }
            if app.current_user_id.is_none() {
                ui.label("You must be logged in to open cases.");
                if ui.button("⬅ Back").clicked() {
                    app.screen = crate::Screen::MainMenu;
                }
                return;
            }

            let user_id = app.current_user_id.unwrap();

            // If an open-case animation is running, render it
            if let Some(state) = &mut app.open_case_state {
                let elapsed = Instant::now().duration_since(state.start).as_secs_f32();
                let total = state.duration.as_secs_f32();
                let t = (elapsed / total).clamp(0.0, 1.0);

                // easing (ease-out cubic)
                let ease = 1.0 - (1.0 - t).powf(3.0);
                // determine the spin offset into the options list
                let spins = 6.0; // number of full cycles
                let pos = (ease * spins * state.options.len() as f32) as usize;

                ui.label("Rolling...");
                ui.add_space(6.0);

                // Center the carousel horizontally by computing left padding.
                let window = 7usize;
                let max_dim = 64.0_f32;
                let carousel_width = window as f32 * max_dim + ((window - 1) as f32) * 6.0;
                let avail = ui.available_width();
                let left_pad = ((avail - carousel_width) / 2.0).max(0.0);

                ui.horizontal(|ui| {
                    if left_pad > 0.0 { ui.add_space(left_pad); }
                    let center = pos % state.options.len();
                    for i in 0..window {
                        let idx = (center + i + state.options.len() - window/2) % state.options.len();
                        let skin = &state.options[idx];
                        // draw thumbnail (best-effort)
                        let key = format!("skin-{}", skin.id);
                        if let Some(info) = app.textures.get(&key) {
                            let (w, h) = (info.size[0] as f32, info.size[1] as f32);
                            let (dw, dh) = if w > 0.0 && h > 0.0 {
                                let ratio = w / h;
                                if ratio >= 1.0 { (max_dim, max_dim / ratio) } else { (max_dim * ratio, max_dim) }
                            } else { (max_dim, max_dim) };
                            ui.add(egui::Image::new((info.handle.id(), egui::vec2(dw, dh))));
                        } else if let Some(b64) = &skin.image_base64 {
                            let data = if b64.starts_with("data:") {
                                match b64.find(',') { Some(idx) => &b64[idx + 1..], None => &b64 }
                            } else { &b64 };
                            if let Ok(raw) = base64::engine::general_purpose::STANDARD.decode(data) {
                                if let Ok(img) = image::load_from_memory(&raw) {
                                    let img = img.to_rgba8();
                                    let (w, h) = img.dimensions();
                                    let size = [w as usize, h as usize];
                                    let pixels = img.into_raw();
                                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                                    let handle = ctx.load_texture(key.clone(), color_image, egui::TextureOptions::default());
                                    app.textures.insert(key.clone(), crate::TextureInfo { handle: handle.clone(), size });
                                    let (dw, dh) = if w > 0 && h > 0 {
                                        let aw = w as f32; let ah = h as f32; let ratio = aw / ah;
                                        if ratio >= 1.0 { (max_dim, max_dim / ratio) } else { (max_dim * ratio, max_dim) }
                                    } else { (max_dim, max_dim) };
                                    ui.add(egui::Image::new((handle.id(), egui::vec2(dw, dh))));
                                } else {
                                    ui.label(&skin.name);
                                }
                            } else {
                                ui.label(&skin.name);
                            }
                        } else {
                            ui.label(&skin.name);
                        }
                        ui.add_space(6.0);
                    }
                });

                ui.add_space(8.0);

                if elapsed >= total && !state.finished {
                    state.finished = true;
                }

                if state.finished {
                    ui.label(egui::RichText::new(format!("You won: {}", state.selected.name)).strong());
                    ui.add_space(6.0);
                    // show colored rarity under the name
                    let r = state.selected.rarity.clone().unwrap_or_default();
                    ui.label(crate::ui::rarity::rarity_richtext(&r));
                    ui.add_space(6.0);
                    if ui.button("Collect").clicked() {
                        app.message = format!("Received {} (inv #{})", state.selected.name, state.selected_inv_id);
                        app.open_case_state = None;
                    }
                }

                return;
            }

            // No animation running — show open case button
            ui.label(format!("Open a case for ${:.2}", CASE_COST));
            ui.add_space(6.0);
            if ui.button("Open Case").clicked() {
                // perform the case opening logic (select skin and insert inventory)
                match crate::scripts::open_skins::open_case(&app.db_path, user_id, CASE_COST) {
                    Ok((inv_id, selected)) => {
                        // Build an options carousel for animation (sample and include selected)
                        let mut opts = crate::db::list_skins(&app.db_path).unwrap_or_default();
                        // ensure selected is included
                        if !opts.iter().any(|s| s.id == selected.id) {
                            opts.push(selected.clone());
                        }
                        // shuffle for visual variety
                        use rand::seq::SliceRandom;
                        let mut rng = rand::thread_rng();
                        opts.shuffle(&mut rng);

                        app.open_case_state = Some(crate::OpenCaseState {
                            options: opts,
                            selected: selected,
                            selected_inv_id: inv_id,
                            start: Instant::now(),
                            duration: std::time::Duration::from_secs_f32(2.6),
                            finished: false,
                        });
                    }
                    Err(e) => {
                        app.message = format!("Failed to open case: {}", e);
                    }
                }
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
