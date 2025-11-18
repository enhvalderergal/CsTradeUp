use crate::db;

/// Sell an owned inventory item. Credits the user's balance by `price`
/// and removes the inventory row. Returns the new balance on success.
pub fn sell_item(db_path: &str, user_id: i64, inventory_id: i64, price: f64) -> Result<f64, String> {
    // Credit the user's balance first, then remove the inventory item.
    let new_bal = db::change_user_balance(db_path, user_id, price)?;
    db::remove_inventory_item(db_path, inventory_id)?;
    Ok(new_bal)
}
