mod db;
mod models;
mod ui;
mod scripts;

use eframe::egui;
use std::time::{Duration, Instant};

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

    // temp auth fields
    pub username: String,
    pub password: String,
    pub message: String,
    splash_deadline: Instant,
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
            username: String::new(),
            password: String::new(),
            message,
            // Show a little splash screen for 2 seconds (ui/main_menu.rs)
            splash_deadline: Instant::now() + Duration::from_secs(2),
        }
    }
}

impl eframe::App for CsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // If we're still in the splash period, show the splash screen
        if Instant::now() < self.splash_deadline {
            ui::main_menu::show_splash(self, ctx);
            return;
        }

        match &self.screen {
            Screen::MainMenu => ui::main_menu::show_main_menu(self, ctx),
            Screen::Auth(mode) => ui::auth::show_auth(self, ctx, mode.clone()),
            Screen::LoggedIn(name) => ui::main_menu::show_logged_in(self, ctx, name.clone()),
            Screen::Buy => scripts::buy::show_buy(self, ctx),
            Screen::Sell => scripts::sell::show_sell(self, ctx),
            Screen::Tradeup => scripts::tradeup::show_tradeup(self, ctx),
            Screen::OpenSkins => scripts::open_skins::show_open_skins(self, ctx),
            Screen::Inventory => scripts::inventory::show_inventory(self, ctx),
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

