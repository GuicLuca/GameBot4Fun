use std::sync::{Arc};
use log::info;
use tokio::sync::Mutex;

use tokio_rusqlite::{Connection, Error};

/**
 * This method execute migration on the database
 * to ensure every table are created before the bot
 * use it.
 *
 * @param conn: SharedConnection, the database access
 *
 * @return Result<(), Error>
 */
pub async fn run_migrations(conn: SharedConnection) -> Result<(), Error> {
    conn.lock().await.call(|conn|{
        conn.execute_batch(
        "
            CREATE TABLE IF NOT EXISTS tips (
                  id INTEGER PRIMARY KEY AUTOINCREMENT,
                  title TEXT NOT NULL UNIQUE,
                  content TEXT NOT NULL,
                  tags TEXT
            );

            CREATE TABLE IF NOT EXISTS scheduler_config (
                  id INTEGER PRIMARY KEY AUTOINCREMENT,
                  channel INTEGER NOT NULL,
                  hour INTEGER NOT NULL,
                  minute INTEGER NOT NULL
            );
            "
        )
    }).await?;

    info!("Database has been migrated successfully");
    Ok(())
}

// Used to share the database connection through each tokio task and functions.
pub type SharedConnection = Arc<Mutex<Connection>>;