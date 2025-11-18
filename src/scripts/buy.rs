use crate::db;

/// Attempt to purchase a skin for a user.
/// Performs balance checks, updates the user's balance and inserts an inventory row.
/// Returns Ok(()) on success or Err(String) with a user-friendly error message.
pub fn attempt_buy(db_path: &str, user_id: i64, skin_id: i64, price: f64) -> Result<(), String> {
    match db::get_user_by_id(db_path, user_id) {
        Ok(Some(user)) => {
            if user.balance < price {
                return Err("Not enough funds to buy this skin".into());
            }

            // Deduct balance
            db::change_user_balance(db_path, user_id, -price)?;

            // Add inventory row for the purchased skin
            db::add_inventory_item(db_path, user_id, skin_id)?;

            Ok(())
        }
        Ok(None) => Err("User not found".into()),
        Err(e) => Err(e),
    }
}
