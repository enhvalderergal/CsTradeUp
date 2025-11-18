use rusqlite::{params, Connection, Result};
use crate::models::User;

type DbResult<T> = std::result::Result<T, String>;
pub fn init_db(path: &str) -> Result<()> {
    let conn = Connection::open(path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL,
            balance REAL DEFAULT 100.0
        )",
        [],
    )?;

    // Migration: ensure older DBs get the `balance` column so SELECTs that include it don't fail.
    // If the column is missing, add it and initialize existing rows to the default starting balance.
    {
        let mut has_balance = false;
        let mut info_stmt = conn.prepare("PRAGMA table_info(users)")?;
        let mut rows = info_stmt.query([])?;
        while let Some(row) = rows.next()? {
            let name: String = row.get(1)?;
            if name == "balance" {
                has_balance = true;
                break;
            }
        }

        if !has_balance {
            conn.execute("ALTER TABLE users ADD COLUMN balance REAL DEFAULT 100.0", [])?;
            conn.execute("UPDATE users SET balance = 100.0 WHERE balance IS NULL", [])?;
        }
    }

    // Inventory table: keeps track of skins owned by users. Each row represents
    // a single copy owned by a user. Duplicates are allowed by inserting
    // multiple rows for the same skin_id/user_id.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS inventory (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            skin_id INTEGER NOT NULL,
            FOREIGN KEY(user_id) REFERENCES users(id),
            FOREIGN KEY(skin_id) REFERENCES skins(id)
        )",
        [],
    )?;

    // Catalog of available skins
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
    )?;

    // If a developer-provided data/skins.json exists, seed those skins into the catalog.
    let seed_path = std::path::Path::new("data/skins.json");
    if seed_path.exists() {
        if let Ok(json) = std::fs::read_to_string(seed_path) {
            if let Ok(list) = serde_json::from_str::<Vec<SeedSkin>>(&json) {
                for s in list {
                    let _ = add_skin(
                        path,
                        &s.name,
                        s.rarity.as_deref(),
                        s.price.unwrap_or(0.0),
                        s.collection.as_deref(),
                        s.weapon_type.as_deref(),
                        s.image_base64.as_deref(),
                    );
                }
            }
        }
    }

    Ok(())
}

// Helper struct for seeding from JSON (local to db.rs)
#[derive(serde::Deserialize)]
struct SeedSkin {
    name: String,
    rarity: Option<String>,
    price: Option<f64>,
    collection: Option<String>,
    weapon_type: Option<String>,
    image_base64: Option<String>,
}

/// Create a user. Returns Ok(()) on success, or Err(String) with a friendly message on failure. Fx if username is taken then we return a friendly error.
pub fn create_user(path: &str, username: &str, password: &str) -> DbResult<User> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;

    // Check if username already exists
    let existing: i64 = conn
        .query_row(
            "SELECT COUNT(1) FROM users WHERE username = ?1",
            params![username],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if existing > 0 {
        return Err("username taken".into());
    }

    conn.execute(
        "INSERT INTO users (username, password, balance) VALUES (?1, ?2, 100.0)",
        params![username, password],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    Ok(User {
        id,
        username: username.to_string(),
        password: password.to_string(),
        balance: 100.0,
    })
}

pub fn authenticate(path: &str, username: &str, password: &str) -> DbResult<Option<User>> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, username, password, balance FROM users WHERE username = ?1")
        .map_err(|e| e.to_string())?;

    let mut rows = stmt
        .query_map(params![username], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                password: row.get(2)?,
                balance: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;

    if let Some(res) = rows.next() {
        let user = res.map_err(|e| e.to_string())?;
        if user.password == password {
            Ok(Some(user))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

/// Add an inventory item for a user. Returns the inserted InventoryItem on success.
/// Insert an inventory item referring to an existing catalog skin id.
pub fn add_inventory_item(
    path: &str,
    user_id: i64,
    skin_id: i64,
) -> DbResult<crate::models::InventoryItem> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO inventory (user_id, skin_id) VALUES (?1, ?2)",
        params![user_id, skin_id],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    Ok(crate::models::InventoryItem { id, user_id, skin_id })
}

/// Get a user by id
pub fn get_user_by_id(path: &str, user_id: i64) -> DbResult<Option<crate::models::User>> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, username, password, balance FROM users WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    match stmt.query_row(params![user_id], |row| {
        Ok(crate::models::User {
            id: row.get(0)?,
            username: row.get(1)?,
            password: row.get(2)?,
            balance: row.get(3)?,
        })
    }) {
        Ok(u) => Ok(Some(u)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Change (add or subtract) a user's balance by `delta` and return the new balance.
pub fn change_user_balance(path: &str, user_id: i64, delta: f64) -> DbResult<f64> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE users SET balance = balance + ?1 WHERE id = ?2",
        params![delta, user_id],
    )
    .map_err(|e| e.to_string())?;

    // Return updated balance
    let new_balance: f64 = conn
        .query_row("SELECT balance FROM users WHERE id = ?1", params![user_id], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    Ok(new_balance)
}

/// Insert an inventory item referring to an existing catalog skin id.
// (removed add_inventory_item_with_skin_id - inventory now references skins by id only)

/// Add a skin to the canonical catalog. Returns the inserted Skin.
pub fn add_skin(
    path: &str,
    name: &str,
    rarity: Option<&str>,
    price: f64,
    collection: Option<&str>,
    weapon_type: Option<&str>,
    image_base64: Option<&str>,
) -> DbResult<crate::models::Skin> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR IGNORE INTO skins (name, rarity, price, collection, weapon_type, image_base64) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![name, rarity, price, collection, weapon_type, image_base64],
    )
    .map_err(|e| e.to_string())?;

    // Return the skin (fetch id)
    let mut stmt = conn
    .prepare("SELECT id, name, rarity, price, collection, weapon_type, image_base64 FROM skins WHERE name = ?1")
        .map_err(|e| e.to_string())?;

    let skin = stmt
        .query_row(params![name], |row| {
            Ok(crate::models::Skin {
                id: row.get(0)?,
                name: row.get(1)?,
                rarity: row.get(2)?,
                price: row.get(3)?,
                collection: row.get(4)?,
                weapon_type: row.get(5)?,
        image_base64: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(skin)
}

pub fn get_skin_by_name(path: &str, name: &str) -> DbResult<Option<crate::models::Skin>> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, rarity, price, collection, weapon_type, image_base64 FROM skins WHERE name = ?1")
        .map_err(|e| e.to_string())?;

    match stmt.query_row(params![name], |row| {
        Ok(crate::models::Skin {
            id: row.get(0)?,
            name: row.get(1)?,
            rarity: row.get(2)?,
            price: row.get(3)?,
            collection: row.get(4)?,
            weapon_type: row.get(5)?,
            image_base64: row.get(6)?,
        })
    }) {
        Ok(s) => Ok(Some(s)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn list_skins(path: &str) -> DbResult<Vec<crate::models::Skin>> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, rarity, price, collection, weapon_type, image_base64 FROM skins ORDER BY name")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(crate::models::Skin {
                id: row.get(0)?,
                name: row.get(1)?,
                rarity: row.get(2)?,
                price: row.get(3)?,
                collection: row.get(4)?,
                weapon_type: row.get(5)?,
                image_base64: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut skins = Vec::new();
    for r in rows {
        skins.push(r.map_err(|e| e.to_string())?);
    }

    Ok(skins)
}

/// Get all inventory items for a given user.
pub fn get_inventory_for_user(path: &str, user_id: i64) -> DbResult<Vec<crate::models::OwnedSkin>> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;

    // Join inventory -> skins so callers get inventory + catalog metadata in one call
    let mut stmt = conn
        .prepare(
            "SELECT i.id, i.user_id, i.skin_id,
                    s.id, s.name, s.rarity, s.price, s.collection, s.weapon_type, s.image_base64
             FROM inventory i
             LEFT JOIN skins s ON i.skin_id = s.id
             WHERE i.user_id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![user_id], |row| {
            let inv = crate::models::InventoryItem {
                id: row.get(0)?,
                user_id: row.get(1)?,
                skin_id: row.get(2)?,
            };

            // skin fields may be NULL (if skin_id is null or missing)
            let skin_id: Option<i64> = row.get(3)?;
            let skin = if let Some(_) = skin_id {
                Some(crate::models::Skin {
                    id: row.get(3)?,
                    name: row.get(4)?,
                    rarity: row.get(5)?,
                    price: row.get(6)?,
                    collection: row.get(7)?,
                    weapon_type: row.get(8)?,
                    image_base64: row.get(9)?,
                })
            } else {
                None
            };

            Ok(crate::models::OwnedSkin { inventory: inv, skin })
        })
        .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    for r in rows {
        items.push(r.map_err(|e| e.to_string())?);
    }

    Ok(items)
}

/// Update the quantity of an inventory item. Returns Ok(()) on success.
// Removed update_inventory_quantity - inventory no longer tracks a quantity field.

/// Remove an inventory item by id.
pub fn remove_inventory_item(path: &str, item_id: i64) -> DbResult<()> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM inventory WHERE id = ?1", params![item_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
