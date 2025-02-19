use poise::serenity_prelude::prelude::TypeMapKey;
use sqlx::SqliteConnection;

pub struct DbConnection;
impl TypeMapKey for DbConnection {
    type Value = SqliteConnection;
}
