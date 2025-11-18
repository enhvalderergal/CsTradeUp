use eframe::egui;
use eframe::egui::{Color32, Pos2, Rect, Rounding, Shape, Stroke};
use std::f32::consts::TAU;
use std::time::Instant;

/// Splash screen with steam-like bokeh background and spinner.
pub fn show_splash(app: &mut crate::CsApp, ctx: &egui::Context) {
    // Should roughly match how CsApp sets splash_deadline
    let total_secs: f32 = 10.0;

    // Progress 0..1
    let now = Instant::now();
    let remaining = if app.splash_deadline > now {
        app.splash_deadline.duration_since(now).as_secs_f32()
    } else {
        0.0
    };
    let elapsed = (total_secs - remaining).max(0.0);
    let p = (elapsed / total_secs).clamp(0.0, 1.0);
    let t = elapsed;

    egui::CentralPanel::default()
        .frame(
            egui::Frame::none()
                // We paint the background ourselves
                .fill(Color32::from_rgba_premultiplied(0, 0, 0, 0))
                .inner_margin(egui::Margin::same(0.0)),
        )
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter();

            // Single rounded background (visually the whole window)
            let outer_rect = rect.shrink(4.0);
            let round = Rounding::same(24.0);

            painter.rect(
                outer_rect,
                round,
                Color32::from_rgba_premultiplied(16, 16, 24, 245),
                Stroke::new(1.0, Color32::from_white_alpha(24)),
            );

            // ----- Animated “bokeh” background (soft moving circles) -----
            // A few layers with different speeds/sizes for a parallax feel.
            // Use the elapsed splash time so the animation progresses as the
            // splash advances.

            let layers = &[
                (12, 120.0, 0.07, 16, 28),  // (count, radius, speed, min alpha, max alpha)
                (18, 80.0, 0.11, 18, 34),
                (28, 50.0, 0.16, 16, 26),
            ];

            for (layer_idx, (count, r, speed, amin, amax)) in layers.iter().enumerate() {
                for i in 0..*count {
                    let f = i as f32 / *count as f32;
                    // pseudo-random but deterministic per circle/layer
                    let phase = (f * TAU * 3.0) + (layer_idx as f32 * 1.73);
                    let ox = (phase.cos() * 0.5 + 0.5) * outer_rect.width();
                    let oy = (phase.sin() * 0.5 + 0.5) * outer_rect.height();

                    // animate slow drift
                    let wiggle = (t * *speed + phase).sin_cos();
                    let x = outer_rect.left() + ox + wiggle.0 * (outer_rect.width() * 0.08);
                    let y = outer_rect.top()  + oy + wiggle.1 * (outer_rect.height() * 0.06);

                    // soft color & alpha
                    let hue = (f * 360.0 + t * 6.0).rem_euclid(360.0);
                    let (r8, g8, b8) = hsv_u8(hue, 0.16, 0.22);
                    let a_f = egui::remap_clamp(wiggle.0, -1.0f32..=1.0f32, *amin as f32..=*amax as f32);
                    let a = a_f as u8;

                    painter.add(Shape::circle_filled(
                        Pos2::new(x, y),
                        *r,
                        Color32::from_rgba_premultiplied(r8, g8, b8, a),
                    ));
                }
            }

            // Slight dark vignette and center focus to make text/spinner readable
            let base = outer_rect.width().min(outer_rect.height());
            // Shaded rectangular area covering the spinner and the loading text
            let spinner_radius = 32.0_f32;
            let pad = 8.0_f32;
            // Text offsets used elsewhere in this file
            let text_x_offset = -110.0_f32;
            let title_y_offset = 44.0_f32;
            let status_vgap = 26.0_f32; // distance from title to status
            let status_font_h = 14.0_f32;

            // Center the shaded box on the splash center and give it rounded corners.
            let c = outer_rect.center();
            // Compute a comfortable size for the shaded box so it covers the
            // spinner and the two lines of text.
            let shade_w = 300.0_f32; // width of the shaded box
            let shade_h = spinner_radius * 2.0 + title_y_offset + status_vgap + status_font_h + pad * 2.0;
            let shade_center = c; // keep horizontally centered with spinner
            let shade_rect = Rect::from_center_size(shade_center, egui::vec2(shade_w, shade_h));
            painter.rect_filled(shade_rect, Rounding::same(12.0), Color32::from_black_alpha(110));

            painter.rect_filled(
                outer_rect,
                round,
                Color32::from_black_alpha(90),
            );

            // ----- Spinner -----
            let center = outer_rect.center();
            let radius = 32.0;
            let thickness = 5.0;

            // base ring
            painter.circle_stroke(
                center,
                radius,
                Stroke::new(thickness, Color32::from_white_alpha(70)),
            );

            // slow arc
            let spin = t * 0.9;
            let arc_span = TAU * 0.32;
            let start = spin % TAU;
            let end = start + arc_span;

            // Draw arc by sampling points along the circle and stroking a polyline
            {
                let segments = 32u32;
                let mut points = Vec::with_capacity((segments + 1) as usize);
                let span = end - start;
                for i in 0..=segments {
                    let a = start + (i as f32 / segments as f32) * span;
                    let px = center.x + radius * a.cos();
                    let py = center.y + radius * a.sin();
                    points.push(Pos2::new(px, py));
                }
                painter.add(Shape::line(points, Stroke::new(thickness, Color32::from_white_alpha(255))));
            }

            // ----- Title + status text -----
            let fade = if p < 0.15 {
                (p / 0.15).clamp(0.0, 1.0)
            } else if p > 0.85 {
                ((1.0 - p) / 0.15).clamp(0.0, 1.0)
            } else {
                1.0
            };

            let text_col = |base: u8| {
                let min_factor = 0.6;
                let f = min_factor + (1.0 - min_factor) * fade;
                Color32::from_white_alpha((base as f32 * f) as u8)
            };

            let title = egui::RichText::new("Loading user data…")
                .size(19.0)
                .strong()
                .color(Color32::from_white_alpha(230));

            let dots = ((t * 1.2) as i32 % 3) + 1;
            let status_string = if p < 0.98 {
                format!("Please wait{}", ".".repeat(dots as usize))
            } else {
                "Launching…".to_string()
            };

            // Center the title and status inside the shaded rect
            let shade_center = shade_rect.center();
            let title_pos = Pos2::new(shade_center.x, shade_center.y + title_y_offset - 10.0);
            painter.text(
                title_pos,
                egui::Align2::CENTER_TOP,
                title.text(),
                egui::FontId::proportional(19.0),
                text_col(255),
            );

            let status_pos = Pos2::new(shade_center.x, title_pos.y + status_vgap);
            painter.text(
                status_pos,
                egui::Align2::CENTER_TOP,
                status_string,
                egui::FontId::proportional(14.0),
                text_col(220),
            );

            if p < 1.0 {
                ctx.request_repaint();
            }
        });
}

/// Tiny HSV→RGB helper (0..360, 0..1, 0..1) returning u8.
fn hsv_u8(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match (h / 60.0) as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}
