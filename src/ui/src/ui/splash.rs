use eframe::egui;
use std::time::Instant;

// Animated splash screen. Uses the existing `CsApp::splash_deadline` to compute
// progress over a short duration and draws a scaled, fading title and a
// progress bar. Keeping animation details here isolates the visual code.
pub fn show_splash(app: &mut crate::CsApp, ctx: &egui::Context) {
    // Total duration (must match how `CsApp` set `splash_deadline`)
    let total_secs: f32 = 2.0;

    // If for any reason the deadline is already past, clamp to 1.0
    let now = Instant::now();
    let remaining = if app.splash_deadline > now {
        app.splash_deadline.duration_since(now).as_secs_f32()
    } else {
        0.0
    };

    let elapsed = (total_secs - remaining).max(0.0);
    let mut p = (elapsed / total_secs).clamp(0.0, 1.0);

    // Simple easing for a nicer motion
    let ease = 1.0 - (1.0 - p).powf(3.0);

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
                // Animated main title: grows and fades in
                let base_size = 36.0;
                let max_extra = 120.0;
                let text_size = base_size + ease * max_extra;

                let alpha = (ease * 255.0).clamp(0.0, 255.0) as u8;
                let color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha);

                ui.label(
                    egui::RichText::new("CsTradeUp")
                        .size(text_size)
                        .color(color)
                        .strong(),
                );

                ui.add_space(8.0);

                // Subtle animated dots that move based on progress
                let dots = match (p * 4.0) as usize % 4 {
                    0 => "",
                    1 => ".",
                    2 => "..",
                    _ => "...",
                };

                ui.label(egui::RichText::new(format!("Loading{}", dots)).color(color));

                ui.add_space(12.0);

                // Progress bar with the eased progress
                ui.add_sized(
                    [300.0, 12.0],
                    egui::ProgressBar::new(ease)
                        .show_percentage()
                        .animate(true),
                );

                // Small hint text that fades out as the splash finishes
                if p < 0.9 {
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("Preparing interface...")
                            .small()
                            .color(color),
                    );
                }

                // Request repaint while animating so frames keep coming
                if p < 1.0 {
                    ctx.request_repaint();
                }
            },
        );
    });
}
