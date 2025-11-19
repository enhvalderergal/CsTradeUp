#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod models;
mod ui;
mod scripts;

use eframe::egui;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::{env, path::PathBuf};

/// Small helper storing a texture handle and original image size
pub struct TextureInfo {
    pub handle: egui::TextureHandle,
    pub size: [usize; 2],
}

/// State used while animating an open-case roll.
pub struct OpenCaseState {
    pub options: Vec<crate::models::Skin>,
    pub selected: crate::models::Skin,
    pub selected_inv_id: i64,
    pub start: Instant,
    pub duration: Duration,
    pub finished: bool,
}

/// Represents the different screens in the application
#[derive(Clone)]
pub enum Screen {
    MainMenu,
    Auth(AuthMode),
    LoggedIn(String),
    Buy,
    Sell,
    Tradeup,
    OpenSkins,
    Inventory,
}

// Since we are useing the same structure for both login and registration, we define an enum to differentiate the modes
#[derive(Clone)]
pub enum AuthMode {
    Login,
    Register,
}

pub struct CsApp {
    screen: Screen,
    pub db_path: String,

    // current logged-in user id (set after successful auth)
    pub current_user_id: Option<i64>,

    // temp auth fields
    pub username: String,
    pub password: String,
    pub message: String,
    splash_deadline: Instant,
    // Texture cache for loaded skin images. Keyed by skin name (or id as string).
    pub textures: HashMap<String, TextureInfo>,
    // Optional open-case animation state
    pub open_case_state: Option<OpenCaseState>,
    // Inventory ids selected for a tradeup
    pub tradeup_selection: Vec<i64>,
    // Selected skin id in the Buy screen
    pub buy_selection: Option<i64>,
}

impl Default for CsApp {
    fn default() -> Self {
         // Initialize the database
    let db_path = get_default_db_path();
       
        // Try to initialize the database, capture any error message for debugging
        let mut message = String::new();
        if let Err(e) = db::init_db(&db_path) {
            message = format!("DB init error: {}", e);
        }

        Self {
            screen: Screen::MainMenu,
            db_path,
            current_user_id: None,
            username: String::new(),
            password: String::new(),
            message,
            textures: HashMap::new(),
            open_case_state: None,
            tradeup_selection: Vec::new(),
            buy_selection: None,
            // Show a little splash screen for 10 seconds (ui/splash.rs)
            splash_deadline: Instant::now() + Duration::from_secs(5),
        }
    }
}

/// Compute a sensible default DB file path for the current platform.
///
/// On Windows this prefers `%APPDATA%\CsTradeUp\cs_trade_up.db`.
/// On other platforms it falls back to `$HOME/.local/share/CsTradeUp/cs_trade_up.db`
/// or the current working directory if home/env vars are not available.
fn get_default_db_path() -> String {
    let file_name = "cs_trade_up.db";

    if cfg!(target_os = "windows") {
        if let Ok(appdata) = env::var("APPDATA") {
            let mut dir = PathBuf::from(appdata);
            dir.push("CsTradeUp");
            // Try to create the directory, ignore errors (fallback handled below)
            let _ = std::fs::create_dir_all(&dir);
            dir.push(file_name);
            return dir.to_string_lossy().into_owned();
        }
    } else {
        if let Ok(home) = env::var("HOME") {
            let mut dir = PathBuf::from(home);
            dir.push(".local");
            dir.push("share");
            dir.push("CsTradeUp");
            let _ = std::fs::create_dir_all(&dir);
            dir.push(file_name);
            return dir.to_string_lossy().into_owned();
        }
    }

    // Fallback: use a file in the current working directory
    file_name.to_string()
}

impl eframe::App for CsApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // If we're still in the splash period, show the splash screen
        if Instant::now() < self.splash_deadline {
            // Apply splash-specific window/frame settings so the splash looks like
            // a small undecorated centered dialog.
            // Note: these calls are idempotent and safe to call every frame.
            // Disable window decorations (no titlebar/toolbar)
            frame.set_decorations(false);
            // Keep the splash on top while visible
            frame.set_always_on_top(true);

            // Set a fixed splash window size roughly matching the splash card
            let splash_w = 460.0f32;
            let splash_h = 280.0f32;
            frame.set_window_size(egui::Vec2::new(splash_w, splash_h));

            // Try to center the window on the primary monitor. If the backend allows
            // setting a window position this will place it roughly centered. If not
            // supported the call is a no-op.
            if let Some(monitor_size) = frame.info().window_info.monitor_size {
                // monitor_size is a Vec2 in logical points
                let center_x = monitor_size.x * 0.5;
                let center_y = monitor_size.y * 0.5;
                let pos = egui::Pos2::new(center_x - splash_w * 0.5, center_y - splash_h * 0.5);
                frame.set_window_pos(pos);
            }

            // forward to the dedicated splash module
            ui::splash::show_splash(self, ctx);
            return;
        } else {
            // Restore normal window behaviour once the splash is gone.
            frame.set_decorations(true);
            frame.set_always_on_top(false);

            // Restore normal window size (match `initial_window_size` in `main`).
            let normal_w = 700.0_f32;
            let normal_h = 650.0_f32;
            frame.set_window_size(egui::Vec2::new(normal_w, normal_h));

            // Try to center the window on the primary monitor after resizing.
            if let Some(monitor_size) = frame.info().window_info.monitor_size {
                let center_x = monitor_size.x * 0.5;
                let center_y = monitor_size.y * 0.5;
                let pos = egui::Pos2::new(center_x - normal_w * 0.5, center_y - normal_h * 0.5);
                frame.set_window_pos(pos);
            }
        }

        match &self.screen {
            Screen::MainMenu => ui::main_menu::show_main_menu(self, ctx),
            Screen::Auth(mode) => ui::auth::show_auth(self, ctx, mode.clone()),
            Screen::LoggedIn(name) => ui::main_menu::show_logged_in(self, ctx, name.clone()),
            Screen::Buy => ui::screens::buy::show_buy(self, ctx),
            Screen::Sell => ui::screens::sell::show_sell(self, ctx),
            Screen::Tradeup => ui::screens::tradeup::show_tradeup(self, ctx),
            Screen::OpenSkins => ui::screens::open_skins::show_open_skins(self, ctx),
            Screen::Inventory => ui::screens::inventory::show_inventory(self, ctx),
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    // Set initial window size
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2::new(700.0, 700.0)),
        ..Default::default()
    };
    // Run the application
    eframe::run_native(
        "CsTradeUp", // Name of the application window
        options, // Window options (size etc.)
        Box::new(|_cc| Box::new(CsApp::default())), 
    )
}

