#[derive(Debug, Clone)] /// Represents the data structure of a user in the system
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    /// User account balance (stored as REAL in SQLite)
    pub balance: f64,
}

#[derive(Debug, Clone)] // Represents a single inventory item owned by a user.
pub struct InventoryItem {
    pub id: i64,
    pub user_id: i64,
    pub skin_id: i64,
}


#[derive(Debug, Clone)]
/// Represents a canonical skin in the catalog.
pub struct Skin {
    pub id: i64,
    pub name: String,
    pub rarity: Option<String>,
    pub price: f64,
    pub collection: Option<String>,
    pub weapon_type: Option<String>,
    /// Image stored as base64 data URI or raw base64 string
    pub image_base64: Option<String>,
}

#[derive(Debug, Clone)]
/// Lightweight association used in-memory when exposing a user's owned skin
/// together with catalog metadata. Not directly mapped to a single DB table.
pub struct OwnedSkin {
    pub inventory: InventoryItem,
    pub skin: Option<Skin>,
}
