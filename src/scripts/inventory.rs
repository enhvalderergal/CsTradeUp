use crate::models::OwnedSkin;
use crate::db;

/// Domain struct describing a skin to be added/managed in inventory.
#[derive(Debug, Clone)]
pub struct SkinInfo {
    pub name: String,
    pub rarity: Option<String>,
    pub price: f64,
    pub collection: Option<String>,
    pub weapon_type: Option<String>,
    pub image_base64: Option<String>,
}

/// Add a skin to a user's inventory by ensuring the skin exists in the catalog
/// and then inserting/updating an inventory row that references the catalog id.
pub fn add_skin(db_path: &str, user_id: i64, skin: SkinInfo) -> Result<OwnedSkin, String> {
    // Ensure the skin exists in the catalog
    let existing_skin = db::get_skin_by_name(db_path, &skin.name)?;
    let catalog_skin = if let Some(s) = existing_skin {
        s
    } else {
        db::add_skin(
            db_path,
            &skin.name,
            skin.rarity.as_deref(),
            skin.price,
            skin.collection.as_deref(),
            skin.weapon_type.as_deref(),
            skin.image_base64.as_deref(),
        )?
    };

    // Always insert a new inventory ownership row (duplicates allowed).
    let inv = db::add_inventory_item(db_path, user_id, catalog_skin.id)?;
    Ok(OwnedSkin { inventory: inv, skin: Some(catalog_skin) })
}

/// Remove `quantity` units of the named skin from the user's inventory. If quantity reaches 0, delete the row.
pub fn remove_skin_by_name(db_path: &str, user_id: i64, skin_name: &str) -> Result<(), String> {
    // Find catalog skin by name and remove a single ownership row for the user (if any)
    if let Some(skin) = db::get_skin_by_name(db_path, skin_name)? {
        let items = db::get_inventory_for_user(db_path, user_id)?;
        if let Some(existing) = items.into_iter().find(|it| it.inventory.skin_id == skin.id) {
            db::remove_inventory_item(db_path, existing.inventory.id)?;
            Ok(())
        } else {
            Err(format!("No skin named '{}' found for user {}", skin_name, user_id))
        }
    } else {
        Err(format!("No catalog skin named '{}'", skin_name))
    }
}

/// Get all inventory items for a user (thin wrapper around db layer).
pub fn list_inventory(db_path: &str, user_id: i64) -> Result<Vec<OwnedSkin>, String> {
    crate::db::get_inventory_for_user(db_path, user_id)
}

/// Set quantity of a specific inventory item by id.
// set_quantity removed; inventory no longer tracks a numeric quantity.

/// Remove an item by id.
pub fn remove_item(db_path: &str, item_id: i64) -> Result<(), String> {
    crate::db::remove_inventory_item(db_path, item_id)
}
