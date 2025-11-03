#[derive(Debug, Clone)] /// Represents the data structure of a user in the system
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
/// Represents a single inventory item owned by a user.
pub struct InventoryItem {
    pub id: i64,
    pub user_id: i64,
    pub skin_name: String,
    pub rarity: String,
    pub price: f64,
    pub quantity: i32,
}
