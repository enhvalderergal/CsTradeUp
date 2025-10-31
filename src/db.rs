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
