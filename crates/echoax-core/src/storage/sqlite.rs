use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::{EchoAccessError, Result};

const CURRENT_SCHEMA_VERSION: i32 = 1;

/// Row from `sync_versions`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncVersionRecord {
    pub file_path: String,
    pub version: i64,
    pub hash: String,
    pub timestamp: String,
}

/// Row from `devices`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceRecord {
    pub device_id: String,
    pub hostname: String,
    pub last_seen: String,
}

/// Local SQLite store for sync metadata and device registry.
pub struct SqliteStore {
    conn: Mutex<Connection>,
}

impl SqliteStore {
    /// Opens an in-memory database (for tests and ephemeral use).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().map_err(sqlite_err)?;
        Self::apply_migrations(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Opens a database file at `path`, creating it if missing.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path).map_err(sqlite_err)?;
        Self::apply_migrations(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn apply_migrations(conn: &Connection) -> Result<()> {
        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row: &rusqlite::Row| row.get(0))
            .map_err(sqlite_err)?;

        if version < 1 {
            conn.execute_batch(
                r"
                CREATE TABLE IF NOT EXISTS sync_versions (
                    file_path TEXT PRIMARY KEY NOT NULL,
                    version INTEGER NOT NULL,
                    hash TEXT NOT NULL,
                    timestamp TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS devices (
                    device_id TEXT PRIMARY KEY NOT NULL,
                    hostname TEXT NOT NULL,
                    last_seen TEXT NOT NULL
                );
                ",
            )
            .map_err(sqlite_err)?;
            conn.pragma_update(None, "user_version", CURRENT_SCHEMA_VERSION)
                .map_err(sqlite_err)?;
        }

        Ok(())
    }

    fn lock_conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|e| EchoAccessError::Storage(format!("sqlite connection mutex poisoned: {e}")))
    }

    /// Insert or replace a sync version row.
    pub fn upsert_sync_version(
        &self,
        file_path: &str,
        version: i64,
        hash: &str,
        timestamp: &str,
    ) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            r#"
            INSERT INTO sync_versions (file_path, version, hash, timestamp)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(file_path) DO UPDATE SET
                version = excluded.version,
                hash = excluded.hash,
                timestamp = excluded.timestamp
            "#,
            params![file_path, version, hash, timestamp],
        )
        .map_err(sqlite_err)?;
        Ok(())
    }

    /// Returns the row for `file_path`, if any.
    pub fn get_sync_version(&self, file_path: &str) -> Result<Option<SyncVersionRecord>> {
        let conn = self.lock_conn()?;
        let row = conn
            .query_row(
                "SELECT file_path, version, hash, timestamp FROM sync_versions WHERE file_path = ?1",
                params![file_path],
                |r: &rusqlite::Row| {
                    Ok(SyncVersionRecord {
                        file_path: r.get(0)?,
                        version: r.get(1)?,
                        hash: r.get(2)?,
                        timestamp: r.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(sqlite_err)?;
        Ok(row)
    }

    /// Deletes the sync version row for `file_path`.
    pub fn delete_sync_version(&self, file_path: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "DELETE FROM sync_versions WHERE file_path = ?1",
            params![file_path],
        )
        .map_err(sqlite_err)?;
        Ok(())
    }

    /// Lists all sync version rows.
    pub fn list_sync_versions(&self) -> Result<Vec<SyncVersionRecord>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT file_path, version, hash, timestamp FROM sync_versions ORDER BY file_path",
            )
            .map_err(sqlite_err)?;
        let rows = stmt
            .query_map([], |r: &rusqlite::Row| {
                Ok(SyncVersionRecord {
                    file_path: r.get(0)?,
                    version: r.get(1)?,
                    hash: r.get(2)?,
                    timestamp: r.get(3)?,
                })
            })
            .map_err(sqlite_err)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(sqlite_err)?);
        }
        Ok(out)
    }

    /// Insert or replace a device row.
    pub fn upsert_device(&self, device_id: &str, hostname: &str, last_seen: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            r#"
            INSERT INTO devices (device_id, hostname, last_seen)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(device_id) DO UPDATE SET
                hostname = excluded.hostname,
                last_seen = excluded.last_seen
            "#,
            params![device_id, hostname, last_seen],
        )
        .map_err(sqlite_err)?;
        Ok(())
    }

    pub fn get_device(&self, device_id: &str) -> Result<Option<DeviceRecord>> {
        let conn = self.lock_conn()?;
        let row = conn
            .query_row(
                "SELECT device_id, hostname, last_seen FROM devices WHERE device_id = ?1",
                params![device_id],
                |r: &rusqlite::Row| {
                    Ok(DeviceRecord {
                        device_id: r.get(0)?,
                        hostname: r.get(1)?,
                        last_seen: r.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(sqlite_err)?;
        Ok(row)
    }

    pub fn delete_device(&self, device_id: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "DELETE FROM devices WHERE device_id = ?1",
            params![device_id],
        )
        .map_err(sqlite_err)?;
        Ok(())
    }

    pub fn list_devices(&self) -> Result<Vec<DeviceRecord>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn
            .prepare("SELECT device_id, hostname, last_seen FROM devices ORDER BY device_id")
            .map_err(sqlite_err)?;
        let rows = stmt
            .query_map([], |r: &rusqlite::Row| {
                Ok(DeviceRecord {
                    device_id: r.get(0)?,
                    hostname: r.get(1)?,
                    last_seen: r.get(2)?,
                })
            })
            .map_err(sqlite_err)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(sqlite_err)?);
        }
        Ok(out)
    }
}

fn sqlite_err(e: rusqlite::Error) -> EchoAccessError {
    EchoAccessError::Storage(e.to_string())
}

#[cfg(test)]
impl SqliteStore {
    fn user_version(&self) -> Result<i32> {
        let conn = self.lock_conn()?;
        conn.pragma_query_value(None, "user_version", |row: &rusqlite::Row| row.get(0))
            .map_err(sqlite_err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_sets_user_version() {
        let store = SqliteStore::open_in_memory().unwrap();
        assert_eq!(store.user_version().unwrap(), CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn sync_version_crud() {
        let store = SqliteStore::open_in_memory().unwrap();
        store.upsert_sync_version("a/b.txt", 1, "h1", "t1").unwrap();
        let row = store.get_sync_version("a/b.txt").unwrap().unwrap();
        assert_eq!(row.file_path, "a/b.txt");
        assert_eq!(row.version, 1);
        assert_eq!(row.hash, "h1");
        assert_eq!(row.timestamp, "t1");

        store.upsert_sync_version("a/b.txt", 2, "h2", "t2").unwrap();
        let row2 = store.get_sync_version("a/b.txt").unwrap().unwrap();
        assert_eq!(row2.version, 2);

        let list = store.list_sync_versions().unwrap();
        assert_eq!(list.len(), 1);

        store.delete_sync_version("a/b.txt").unwrap();
        assert!(store.get_sync_version("a/b.txt").unwrap().is_none());
    }

    #[test]
    fn device_crud() {
        let store = SqliteStore::open_in_memory().unwrap();
        store
            .upsert_device("dev-1", "host-a", "2026-01-01")
            .unwrap();
        let d = store.get_device("dev-1").unwrap().unwrap();
        assert_eq!(d.hostname, "host-a");

        store
            .upsert_device("dev-1", "host-b", "2026-01-02")
            .unwrap();
        let d2 = store.get_device("dev-1").unwrap().unwrap();
        assert_eq!(d2.hostname, "host-b");

        assert_eq!(store.list_devices().unwrap().len(), 1);
        store.delete_device("dev-1").unwrap();
        assert!(store.get_device("dev-1").unwrap().is_none());
    }
}
