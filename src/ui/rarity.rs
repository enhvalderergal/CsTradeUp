use eframe::egui;

fn canonical_rarity(s: &str) -> String {
    let s = s.to_lowercase();
    if s.contains("consumer") || s.contains("common") { "consumer".into() }
    else if s.contains("industrial") || s.contains("industrial grade") { "industrial".into() }
    else if s.contains("mil-spec") || s.contains("milspec") || s.contains("mil") { "mil-spec".into() }
    else if s.contains("restricted") { "restricted".into() }
    else if s.contains("classified") { "classified".into() }
    else if s.contains("covert") { "covert".into() }
    else if s.contains("rare special") || s.contains("rare") { "rare special".into() }
    else { s }
}

pub fn rarity_color(r: &str) -> egui::Color32 {
    match canonical_rarity(r).as_str() {
        "consumer" => egui::Color32::from_rgb(200, 200, 200),
        "industrial" => egui::Color32::from_rgb(102, 178, 255),
        "mil-spec" => egui::Color32::from_rgb(40, 120, 255),
        "restricted" => egui::Color32::from_rgb(178, 102, 255),
        "classified" => egui::Color32::from_rgb(255, 102, 178),
        "covert" => egui::Color32::from_rgb(255, 60, 60),
        "rare special" => egui::Color32::from_rgb(255, 200, 0),
        _ => egui::Color32::from_gray(180),
    }
}

/// Return a `RichText` with the appropriate color for the given rarity string.
pub fn rarity_richtext(r: &str) -> egui::RichText {
    let color = rarity_color(r);
    egui::RichText::new(r).color(color).strong()
}
