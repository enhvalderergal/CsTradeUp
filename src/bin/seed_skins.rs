use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
struct SeedSkin {
    name: String,
    rarity: Option<String>,
    price: Option<f64>,
    collection: Option<String>,
    weapon_type: Option<String>,
    image_base64: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = "data/db.sqlite"; // change if your DB path differs
    let json_path = "data/skins.json";

    let content = fs::read_to_string(json_path)?;
    let list: Vec<SeedSkin> = serde_json::from_str(&content)?;

    for s in list {
        // call into crate::db by constructing a small one-off connection using rusqlite
        let _ = CsTradeUp_seed::seed_one(db_path, &s);
    }

    println!("Seeding finished (best-effort).");
    Ok(())
}

// Local small helper module to avoid importing the whole crate as a binary
mod CsTradeUp_seed {
    use rusqlite::params;
    use super::SeedSkin;

    pub fn seed_one(db_path: &str, s: &SeedSkin) -> Result<(), String> {
        let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS skins (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                rarity TEXT,
                price REAL DEFAULT 0.0,
                collection TEXT,
                weapon_type TEXT,
                image_base64 TEXT
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT OR IGNORE INTO skins (name, rarity, price, collection, weapon_type, image_base64) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![s.name, s.rarity, s.price, s.collection, s.weapon_type, s.image_base64],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }
}
