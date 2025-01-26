use std::path::Path;

use rusqlite::params;
use tokio_rusqlite::Connection;

use crate::status::Status;

#[derive(Clone, Debug)]
pub struct Database {
    conn: Connection,
}

impl Database {
    pub async fn new<P: AsRef<Path>>(database_path: P) -> Result<Self, tokio_rusqlite::Error> {
        let conn = Connection::open(database_path).await?;

        conn.call(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS statuses (
                    timestamp TEXT NOT NULL,
                    command   TEXT NOT NULL,
                    message   TEXT NOT NULL,
                    status    BOOLEAN NOT NULL
                )",
                [],
            )?;
            Ok(())
        })
        .await?;
        Ok(Self { conn })
    }

    pub async fn add_status(&self, status: Status) -> Result<(), tokio_rusqlite::Error> {
        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO statuses (timestamp, command, message, status)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![&status.timestamp, &status.cmd, &status.msg, &status.status],
                )?;
                Ok(())
            })
            .await
    }

    pub async fn get_statuses(&self) -> Result<Vec<Status>, tokio_rusqlite::Error> {
        self.conn
            .call(|conn| {
                let mut stmt =
                    conn.prepare("SELECT timestamp, command, message, status FROM statuses")?;
                let statuses = stmt
                    .query_map([], |row| {
                        Ok(Status {
                            timestamp: row.get(0)?,
                            cmd: row.get(1)?,
                            msg: row.get(2)?,
                            status: row.get(3)?,
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(statuses)
            })
            .await
    }
}
