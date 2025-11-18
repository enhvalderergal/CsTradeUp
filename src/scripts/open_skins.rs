use rand::prelude::*;

/// Open a case for `user_id`, charging `case_cost`. Selects a skin from the
/// catalog using rarity-weighted randomness, inserts an inventory row and
/// returns `(inventory_id, Skin)` on success.
pub fn open_case(db_path: &str, user_id: i64, case_cost: f64) -> Result<(i64, crate::models::Skin), String> {
    // Ensure user has funds
    let user = crate::db::get_user_by_id(db_path, user_id).map_err(|e| e)?;
    let user = user.ok_or_else(|| "User not found".to_string())?;
    if user.balance < case_cost {
        return Err("Not enough funds to open case".into());
    }

    // Deduct cost immediately
    crate::db::change_user_balance(db_path, user_id, -case_cost).map_err(|e| e)?;

    // Load catalog
    let skins = crate::db::list_skins(db_path).map_err(|e| e)?;
    if skins.is_empty() {
        return Err("No skins available in catalog".into());
    }

    // Map rarities to weights
    fn rarity_weight(r: &Option<String>) -> f64 {
        match r.as_ref().map(|s| s.to_lowercase().trim().to_string()).as_deref() {
            Some("common") => 70.0,
            Some("uncommon") => 20.0,
            Some("rare") => 8.0,
            Some("epic") | Some("mythic") | Some("legendary") => 2.0,
            _ => 10.0,
        }
    }

    let weights: Vec<f64> = skins.iter().map(|s| rarity_weight(&s.rarity)).collect();
    let total: f64 = weights.iter().sum();

    // Choose a random skin using cumulative weights
    let mut rng = thread_rng();
    let mut choice = rng.gen_range(0.0..total);
    let mut idx = 0usize;
    while idx < weights.len() {
        if choice < weights[idx] {
            break;
        }
        choice -= weights[idx];
        idx += 1;
    }
    if idx >= skins.len() { idx = skins.len() - 1; }
    let selected = skins[idx].clone();

    // Insert inventory row for the selected skin
    let inv = crate::db::add_inventory_item(db_path, user_id, selected.id).map_err(|e| e)?;
    Ok((inv.id, selected))
}
