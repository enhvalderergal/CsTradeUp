use rusqlite::{params, Connection, Result};
use crate::models::User;

type DbResult<T> = std::result::Result<T, String>;

pub fn init_db(path: &str) -> Result<()> {
    let conn = Connection::open(path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL
        )",
        [],
    )?;
    // Inventory table: keeps track of skins owned by users. Kept minimal for now.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS inventory (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            skin_name TEXT NOT NULL,
            rarity TEXT,
            price REAL DEFAULT 0.0,
            quantity INTEGER NOT NULL DEFAULT 1,
            FOREIGN KEY(user_id) REFERENCES users(id)
        )",
        [],
    )?;
    Ok(())
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
        "INSERT INTO users (username, password) VALUES (?1, ?2)",
        params![username, password],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    Ok(User {
        id,
        username: username.to_string(),
        password: password.to_string(),
    })
}

pub fn authenticate(path: &str, username: &str, password: &str) -> DbResult<Option<User>> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, username, password FROM users WHERE username = ?1")
        .map_err(|e| e.to_string())?;

    let mut rows = stmt
        .query_map(params![username], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                password: row.get(2)?,
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
pub fn add_inventory_item(
    path: &str,
    user_id: i64,
    skin_name: &str,
    rarity: &str,
    price: f64,
    quantity: i32,
) -> DbResult<crate::models::InventoryItem> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO inventory (user_id, skin_name, rarity, price, quantity) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![user_id, skin_name, rarity, price, quantity],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    Ok(crate::models::InventoryItem {
        id,
        user_id,
        skin_name: skin_name.to_string(),
        rarity: rarity.to_string(),
        price,
        quantity,
    })
}

/// Get all inventory items for a given user.
pub fn get_inventory_for_user(path: &str, user_id: i64) -> DbResult<Vec<crate::models::InventoryItem>> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, user_id, skin_name, rarity, price, quantity FROM inventory WHERE user_id = ?1")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![user_id], |row| {
            Ok(crate::models::InventoryItem {
                id: row.get(0)?,
                user_id: row.get(1)?,
                skin_name: row.get(2)?,
                rarity: row.get(3)?,
                price: row.get(4)?,
                quantity: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    for r in rows {
        items.push(r.map_err(|e| e.to_string())?);
    }

    Ok(items)
}

/// Update the quantity of an inventory item. Returns Ok(()) on success.
pub fn update_inventory_quantity(path: &str, item_id: i64, quantity: i32) -> DbResult<()> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE inventory SET quantity = ?1 WHERE id = ?2",
        params![quantity, item_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Remove an inventory item by id.
pub fn remove_inventory_item(path: &str, item_id: i64) -> DbResult<()> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM inventory WHERE id = ?1", params![item_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
