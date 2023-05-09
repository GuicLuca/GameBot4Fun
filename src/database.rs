use std::sync::{Arc};
use log::info;
use tokio::sync::Mutex;

use tokio_rusqlite::{Connection, Error};

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
        ")
    }).await?;

    info!("Database has been migrated successfully");
    Ok(())
}

pub type SharedConnection = Arc<Mutex<Connection>>;