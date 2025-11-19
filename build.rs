use std::path::Path;

fn main() {
    // Path to icon - change if you store your .ico elsewhere.
    let icon_path = "assets/icon.ico";

    if !Path::new(icon_path).exists() {
        // Emit a cargo warning but do not fail the build. This lets
        // developers build without an icon while reminding them where
        // to place the file to enable embedding.
        println!("cargo:warning=Icon not found at '{}'. Place your .ico there to embed it.", icon_path);
        return;
    }

    let mut res = winres::WindowsResource::new();
    res.set_icon(icon_path);
    res.compile().unwrap();
}
