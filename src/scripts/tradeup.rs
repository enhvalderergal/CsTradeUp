/// Tradeup-related functionality lives in this module. UI for composing a tradeup
/// should be implemented under `src/ui/screens/tradeup.rs`.
///
/// The helpers here perform the core logic (validation, DB changes) and
/// return results suitable for the UI to present.
use rand::prelude::*;
use rusqlite::{params, Connection};

fn canonical_rarity(s: &str) -> String {
    let s = s.to_lowercase();
    if s.contains("consumer") || s.contains("common") { "consumer".into() }
    else if s.contains("industrial") || s.contains("industrial grade") { "industrial".into() }
    else if s.contains("mil-spec") || s.contains("milspec") || s.contains("mil") { "mil-spec".into() }
    else if s.contains("restricted") { "restricted".into() }
    else if s.contains("classified") { "classified".into() }
    else if s.contains("covert") { "covert".into() }
    else if s.contains("rare special") || s.contains("rare") { "rare special".into() }
    else { s }
}

/// Compose a tradeup: consume exactly 10 inventory items owned by `user_id` (their inventory IDs),
/// all must have the same canonical rarity. Produces one random skin of the next-higher
/// canonical rarity. Knife/rare-special items are excluded from being produced.
pub fn compose_tradeup(db_path: &str, user_id: i64, input_item_ids: Vec<i64>) -> Result<i64, String> {
    if input_item_ids.len() != 10 {
        return Err("Tradeup requires exactly 10 items".into());
    }

    // Load the user's inventory to validate ownership and gather rarities
    let owned = crate::db::get_inventory_for_user(db_path, user_id)?;
    let mut by_id = std::collections::HashMap::new();
    for it in owned {
        by_id.insert(it.inventory.id, it);
    }

    let mut rarities = Vec::new();
    let mut skins_in = Vec::new();
    for id in &input_item_ids {
        let entry = by_id.get(id).ok_or_else(|| format!("You do not own inventory item {}", id))?;
        let skin = entry.skin.as_ref().ok_or_else(|| format!("Inventory item {} has no skin metadata", id))?;
        rarities.push(canonical_rarity(&skin.rarity.clone().unwrap_or_default()));
        skins_in.push((entry.inventory.id, skin.clone()));
    }

    // Ensure all rarities are identical
    if rarities.iter().any(|r| r != &rarities[0]) {
        return Err("All items must be the same rarity to trade up".into());
    }

    let current_rarity = rarities[0].clone();

    // Define rarity progression (low -> high)
    let rarity_order = vec![
        "consumer",
        "industrial",
        "mil-spec",
        "restricted",
        "classified",
        "covert",
        "rare special",
    ];

    let pos = rarity_order.iter().position(|&r| r == current_rarity.as_str());
    let idx = match pos {
        Some(i) => i,
        None => return Err(format!("Rarity '{}' cannot be traded up", current_rarity)),
    };

    if idx + 1 >= rarity_order.len() {
        return Err("No higher rarity available to trade up to".into());
    }

    let target_rarity = rarity_order[idx + 1];

    // Load candidate skins from catalog with target rarity, excluding knives / special rare knives
    let candidates_all = crate::db::list_skins(db_path)?;
    let candidates: Vec<crate::models::Skin> = candidates_all
        .into_iter()
        .filter(|s| {
            let r = s.rarity.clone().unwrap_or_default();
            let rcanon = canonical_rarity(&r);
            if rcanon != target_rarity { return false; }
            // Exclude knives: weapon_type contains "knife" or name contains special star
            let name_lower = s.name.to_lowercase();
            let weapon_lower = s.weapon_type.clone().unwrap_or_default().to_lowercase();
            if weapon_lower.contains("knife") || name_lower.contains('★') || name_lower.contains("knife") {
                return false;
            }
            true
        })
        .collect();

    if candidates.is_empty() {
        return Err(format!("No candidate skins found for target rarity '{}'", target_rarity));
    }

    // Pick a random candidate uniformly
    let mut rng = thread_rng();
    let selected = candidates.choose(&mut rng).unwrap().clone();

    // Perform DB transaction: insert resulting inventory row and delete consumed items
    let mut conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO inventory (user_id, skin_id) VALUES (?1, ?2)",
        params![user_id, selected.id],
    )
    .map_err(|e| e.to_string())?;
    let new_id = tx.last_insert_rowid();

    for (inv_id, _skin) in skins_in.iter() {
        let changes = tx
            .execute("DELETE FROM inventory WHERE id = ?1 AND user_id = ?2", params![inv_id, user_id])
            .map_err(|e| e.to_string())?;
        if changes == 0 {
            // Rollback and fail
            return Err(format!("Failed to consume inventory item {}", inv_id));
        }
    }

    tx.commit().map_err(|e| e.to_string())?;

    Ok(new_id)
}
