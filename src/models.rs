#[derive(Debug, Clone)] /// Represents the data structure of a user in the system
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
}
