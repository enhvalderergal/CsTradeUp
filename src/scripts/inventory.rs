use crate::models::InventoryItem;

/// Domain struct describing a skin to be added/managed in inventory.
#[derive(Debug, Clone)]
pub struct SkinInfo {
    pub name: String,
    pub rarity: String,
    pub price: f64,
}

/// Add a skin to a user's inventory. If the skin already exists (by name) we increment quantity,
/// otherwise we insert a new row. Returns the resulting InventoryItem on success.
pub fn add_skin(
    db_path: &str,
    user_id: i64,
    skin: SkinInfo,
    quantity: i32,
) -> Result<InventoryItem, String> {
    // Fetch current inventory for the user and check if the skin already exists
    let items = crate::db::get_inventory_for_user(db_path, user_id)?;

    if let Some(mut existing) = items.into_iter().find(|it| it.skin_name == skin.name) {
        let new_q = existing.quantity.saturating_add(quantity);
        crate::db::update_inventory_quantity(db_path, existing.id, new_q)?;
        existing.quantity = new_q;
        // Optionally update rarity/price if changed -- for now we leave existing metadata
        Ok(existing)
    } else {
        crate::db::add_inventory_item(
            db_path,
            user_id,
            &skin.name,
            &skin.rarity,
            skin.price,
            quantity,
        )
    }
}

/// Remove `quantity` units of the named skin from the user's inventory. If quantity reaches 0, delete the row.
pub fn remove_skin_by_name(db_path: &str, user_id: i64, skin_name: &str, quantity: i32) -> Result<(), String> {
    let items = crate::db::get_inventory_for_user(db_path, user_id)?;
    if let Some(existing) = items.into_iter().find(|it| it.skin_name == skin_name) {
        if quantity >= existing.quantity {
            crate::db::remove_inventory_item(db_path, existing.id)?;
        } else {
            let new_q = existing.quantity - quantity;
            crate::db::update_inventory_quantity(db_path, existing.id, new_q)?;
        }
        Ok(())
    } else {
        Err(format!("No skin named '{}' found for user {}", skin_name, user_id))
    }
}

/// Get all inventory items for a user (thin wrapper around db layer).
pub fn list_inventory(db_path: &str, user_id: i64) -> Result<Vec<InventoryItem>, String> {
    crate::db::get_inventory_for_user(db_path, user_id)
}

/// Set quantity of a specific inventory item by id.
pub fn set_quantity(db_path: &str, item_id: i64, quantity: i32) -> Result<(), String> {
    crate::db::update_inventory_quantity(db_path, item_id, quantity)
}

/// Remove an item by id.
pub fn remove_item(db_path: &str, item_id: i64) -> Result<(), String> {
    crate::db::remove_inventory_item(db_path, item_id)
}
