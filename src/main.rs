mod db;
mod models;
mod ui;
mod scripts;

use eframe::egui;
use std::time::{Duration, Instant};
use std::collections::HashMap;

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
        let db_path = String::from("cs_trade_up.db");
       
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
            splash_deadline: Instant::now() + Duration::from_secs(2),
        }
    }
}

impl eframe::App for CsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // If we're still in the splash period, show the splash screen
        if Instant::now() < self.splash_deadline {
            // forward to the dedicated splash module
            ui::splash::show_splash(self, ctx);
            return;
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
        initial_window_size: Some(egui::Vec2::new(700.0, 480.0)),
        ..Default::default()
    };
    // Run the application
    eframe::run_native(
        "CsTradeUp", // Name of the application window
        options, // Window options (size etc.)
        Box::new(|_cc| Box::new(CsApp::default())), 
    )
}

