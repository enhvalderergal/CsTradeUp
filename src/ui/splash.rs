use eframe::egui;
use eframe::egui::{Color32, Pos2, Rect, Rounding, Shape, Stroke};
use std::f32::consts::TAU;
use std::time::Instant;

/// Steam-like animated splash:
/// - soft, moving “bokeh” background
/// - centered glassy card
/// - circular spinner & status text
/// Uses `app.splash_deadline` to know when to hide.
pub fn show_splash(app: &mut crate::CsApp, ctx: &egui::Context) {
    // How long the splash lasts (must match how CsApp sets splash_deadline)
    let total_secs: f32 = 2.0;

    // Compute progress 0..1
    let now = Instant::now();
    let remaining = if app.splash_deadline > now {
        app.splash_deadline.duration_since(now).as_secs_f32()
    } else {
        0.0
    };
    let elapsed = (total_secs - remaining).max(0.0);
    let p = (elapsed / total_secs).clamp(0.0, 1.0);

    // Drive animation by elapsed splash time (use the computed elapsed value so the
    // spinner rotates as the splash progresses). Using `now.elapsed()` here was
    // returning ~0 because `now` was just created above.
    let t = elapsed;

    egui::CentralPanel::default().frame(
        egui::Frame::none()
            .fill(Color32::from_black_alpha(240)) // dark base
            .inner_margin(egui::Margin::symmetric(0.0, 0.0)),
    )
    .show(ctx, |ui| {
        let rect = ui.max_rect();
        let painter = ui.painter();

        // ----- Animated “bokeh” background (soft moving circles) -----
        // A few layers with different speeds/sizes for a parallax feel.
        let layers = &[
            (12, 240.0, 0.07, 16, 28),  // (count, radius, speed, min alpha, max alpha)
            (18, 160.0, 0.11, 18, 34),
            (28, 100.0, 0.16, 16, 26),
        ];

        for (layer_idx, (count, r, speed, amin, amax)) in layers.iter().enumerate() {
            for i in 0..*count {
                let f = i as f32 / *count as f32;
                // pseudo-random but deterministic per circle/layer
                let phase = (f * TAU * 3.0) + (layer_idx as f32 * 1.73);
                let ox = (phase.cos() * 0.5 + 0.5) * rect.width();
                let oy = (phase.sin() * 0.5 + 0.5) * rect.height();

                // animate slow drift
                let wiggle = (t * *speed + phase).sin_cos();
                let x = rect.left() + ox + wiggle.0 * (rect.width() * 0.08);
                let y = rect.top()  + oy + wiggle.1 * (rect.height() * 0.06);

                // soft color & alpha
                let hue = (f * 360.0 + t * 6.0).rem_euclid(360.0);
                let (r8, g8, b8) = hsv_u8(hue, 0.16, 0.22);
                // remap_clamp expects a RangeInclusive for the output range; build that
                let a_f = egui::remap_clamp(wiggle.0, -1.0f32..=1.0f32, *amin as f32..=*amax as f32);
                let a = a_f as u8;

                painter.add(Shape::circle_filled(
                    Pos2::new(x, y),
                    *r,
                    Color32::from_rgba_premultiplied(r8, g8, b8, a),
                ));
            }
        }

        // subtle vignette to focus center
        painter.rect(
            rect,
                Rounding::ZERO,
            Color32::from_black_alpha(80),
            Stroke::NONE,
        );

        // ----- Center “card” (glassy) -----
        let card_w = 420.0f32;
        let card_h = 220.0f32;
        let card_rect = Rect::from_center_size(rect.center(), egui::vec2(card_w, card_h));
        let card_round = Rounding::same(18.0);

        // glass fill + border
        painter.rect(
            card_rect,
            card_round,
            Color32::from_rgba_premultiplied(24, 24, 28, 230),
            Stroke::new(1.0, Color32::from_white_alpha(35)),
        );

        // inner top highlight
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(card_rect.left(), card_rect.top()),
                Pos2::new(card_rect.right(), card_rect.top() + 56.0),
            ),
            card_round,
            Color32::from_white_alpha(12),
        );

        // ----- Spinner (Steam-ish ring) -----
        let center = card_rect.center() + egui::vec2(0.0, -10.0);
        let radius = 28.0;
        let thickness = 5.0;

        // draw base ring
        painter.circle_stroke(center, radius, Stroke::new(thickness, Color32::from_white_alpha(36)));

        // rotating arc
        let spin = t * 2.1; // rad / s
        let arc_span = TAU * 0.30;
        let start = spin % TAU;
        let end = start + arc_span;

        // draw arc by sampling points along the circle and stroking a polyline
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
            painter.add(Shape::line(points, Stroke::new(thickness, Color32::from_white_alpha(200))));
        }

        // ----- Title + status -----
        let title = egui::RichText::new("Loading user data…")
            .size(18.0)
            .strong()
            .color(Color32::from_white_alpha(220));

        let status = if p < 0.98 {
            // subtle pulsing dots
            let dots = ((t * 1.5) as i32 % 3) + 1;
            format!("Please wait{}", ".".repeat(dots as usize))
        } else {
            "Launching…".to_string()
        };

        // Fade in/out the whole card a touch
        let fade = if p < 0.15 {
            (p / 0.15).clamp(0.0, 1.0)
        } else if p > 0.85 {
            ((1.0 - p) / 0.15).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let text_col = |a: u8| Color32::from_white_alpha(((a as f32) * fade) as u8);

        // layout text
        let title_pos = center + egui::vec2(-110.0, 44.0);
        painter.text(title_pos, egui::Align2::LEFT_TOP, title.text(), egui::FontId::proportional(18.0), text_col(230));

        let status_pos = title_pos + egui::vec2(0.0, 26.0);
        painter.text(
            status_pos,
            egui::Align2::LEFT_TOP,
            status,
            egui::FontId::proportional(14.0),
            text_col(200),
        );

        // Keep animating while splash is visible
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
