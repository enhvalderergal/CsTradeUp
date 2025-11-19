fn main() {
    // Uses the `winres` crate to set the icon for the Windows executable.
    // Put your .ico file at `assets/icon.ico` (or change the path below).
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icon.ico");
    res.compile().unwrap();
}
