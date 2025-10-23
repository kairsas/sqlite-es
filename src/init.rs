use sqlx::{Pool, Sqlite};

/// A convenience method for initializing database tables for Sqlite DB.
#[allow(unused)] // it's used in tests and possibly by this api clients
pub async fn init_tables(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let init_sql = include_str!("../db/init.sql");
    sqlx::query(init_sql).execute(pool).await?;

    Ok(())
}
