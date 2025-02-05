use poise::serenity_prelude::prelude::TypeMapKey;
use sqlx::SqliteConnection;

type Error = crate::Error;
type Context<'a> = crate::Context<'a>;

pub struct DbConnection;
impl TypeMapKey for DbConnection {
    type Value = SqliteConnection;
}
