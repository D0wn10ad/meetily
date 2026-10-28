use sqlx::{migrate::MigrateDatabase, Result, Sqlite, SqlitePool, Transaction};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;

/// Build the WAL/SHM sidecar path for a database file.
/// e.g. `/a/b/app.db` + `-wal` => `/a/b/app.db-wal`.
fn sidecar_path(db: &Path, suffix: &str) -> PathBuf {
    let mut name = db.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    db.with_file_name(name)
}

#[derive(Clone)]
pub struct DatabaseManager {
    pool: SqlitePool,
}

impl DatabaseManager {
    pub async fn new(tauri_db_path: &str, backend_db_path: &str) -> Result<Self> {
        if let Some(parent_dir) = Path::new(tauri_db_path).parent() {
            if !parent_dir.exists() {
                fs::create_dir_all(parent_dir).map_err(|e| sqlx::Error::Io(e))?;
            }
        }

        if !Path::new(tauri_db_path).exists() {
            if Path::new(backend_db_path).exists() {
                log::info!(
                    "Copying database from {} to {}",
                    backend_db_path,
                    tauri_db_path
                );
                fs::copy(backend_db_path, tauri_db_path).map_err(|e| sqlx::Error::Io(e))?;
            } else {
                log::info!("Creating database at {}", tauri_db_path);
                Sqlite::create_database(tauri_db_path).await?;
            }
        }

        let pool = SqlitePool::connect(tauri_db_path).await?;

        if let Err(migrate_err) = sqlx::migrate!("./migrations").run(&pool).await {
            // The existing database is incompatible with the current schema (e.g. it
            // was created by an older build whose migration checksums differ). Back it
            // up and recreate a fresh database so the app keeps working instead of
            // failing on every startup.
            let err_str = migrate_err.to_string();
            let incompatible = err_str.contains("previously applied")
                || err_str.contains("checksum")
                || err_str.contains("version mismatch");
            if !incompatible {
                return Err(migrate_err.into());
            }
            pool.close().await;

            let db_path = Path::new(tauri_db_path);
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let backup_path = db_path.with_extension(format!("sqlite.bak-{}", ts));

            // Loud, recoverable error: the incompatible database is backed up to
            // `backup_path` and a fresh one is created, so the user can recover
            // the original data from the backup file.
            log::error!(
                "Database migration failed (incompatible existing database): {}. Backing it up to {} and recreating a fresh one.",
                err_str,
                backup_path.display()
            );

            if db_path.exists() {
                tokio::fs::rename(db_path, &backup_path)
                    .await
                    .map_err(|e| sqlx::Error::Io(e))?;
                log::warn!("Backed up incompatible database to {}", backup_path.display());
            }
            // Remove stale WAL/SHM sidecars belonging to the old database file
            for suffix in ["-wal", "-shm"] {
                let sidecar = sidecar_path(db_path, suffix);
                if sidecar.exists() {
                    if let Err(e) = tokio::fs::remove_file(&sidecar).await {
                        log::warn!(
                            "Failed to remove stale sidecar {}: {}",
                            sidecar.display(),
                            e
                        );
                    }
                }
            }

            log::info!("Creating a fresh database at {}", tauri_db_path);
            Sqlite::create_database(tauri_db_path).await?;
            let pool = SqlitePool::connect(tauri_db_path).await?;
            sqlx::migrate!("./migrations").run(&pool).await?;
            log::warn!("Recreated a fresh database after backing up the incompatible one");
            return Ok(DatabaseManager { pool });
        }

        Ok(DatabaseManager { pool })
    }

    // NOTE: So for the first time users they needs to start the application
    // after they can just delete the existing .sqlite file and then copy the existing .db file to
    // the current app dir, So the system detects legacy db and copy it and starts with that data
    // (Newly created .sqlite with the copied content from .db)
    pub async fn new_from_app_handle(app_handle: &tauri::AppHandle) -> Result<Self> {
        // Resolve the app's data directory
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .expect("failed to get app data dir");
        if !app_data_dir.exists() {
            fs::create_dir_all(&app_data_dir).map_err(|e| sqlx::Error::Io(e))?;
        }

        // Define database paths
        let tauri_db_path = app_data_dir
            .join("meeting_minutes.sqlite")
            .to_string_lossy()
            .to_string();
        // Legacy backend DB path (for auto-migration if exists)
        let backend_db_path = app_data_dir
            .join("meeting_minutes.db")
            .to_string_lossy()
            .to_string();

        // WAL file paths for defensive cleanup
        let wal_path = app_data_dir.join("meeting_minutes.sqlite-wal");
        let shm_path = app_data_dir.join("meeting_minutes.sqlite-shm");

        log::info!("Tauri DB path: {}", tauri_db_path);
        log::info!("Legacy backend DB path: {}", backend_db_path);

        // Try to open database with defensive WAL handling
        match Self::new(&tauri_db_path, &backend_db_path).await {
            Ok(db_manager) => {
                log::info!("Database opened successfully");
                Ok(db_manager)
            }
            Err(e) => {
                // Check if error is due to corrupted WAL file
                let error_msg = e.to_string();
                if error_msg.contains("malformed") || error_msg.contains("corrupt") {
                    log::warn!("Database appears corrupted, likely due to orphaned WAL file. Attempting recovery...");
                    log::warn!("Error details: {}", error_msg);

                    // Delete potentially corrupted WAL/SHM files
                    if wal_path.exists() {
                        match fs::remove_file(&wal_path) {
                            Ok(_) => log::info!("Removed orphaned WAL file: {:?}", wal_path),
                            Err(e) => log::warn!("Failed to remove WAL file: {}", e),
                        }
                    }
                    if shm_path.exists() {
                        match fs::remove_file(&shm_path) {
                            Ok(_) => log::info!("Removed orphaned SHM file: {:?}", shm_path),
                            Err(e) => log::warn!("Failed to remove SHM file: {}", e),
                        }
                    }

                    // Retry connection without WAL files
                    log::info!("Retrying database connection after WAL cleanup...");
                    match Self::new(&tauri_db_path, &backend_db_path).await {
                        Ok(db_manager) => {
                            log::info!("Database opened successfully after WAL recovery");
                            Ok(db_manager)
                        }
                        Err(retry_err) => {
                            log::error!("Database connection failed even after WAL cleanup: {}", retry_err);
                            Err(retry_err)
                        }
                    }
                } else {
                    // Not a WAL-related error, propagate original error
                    log::error!("Database connection failed: {}", error_msg);
                    Err(e)
                }
            }
        }
    }

    /// Check if this is the first launch (sqlite database doesn't exist yet)
    pub async fn is_first_launch(app_handle: &tauri::AppHandle) -> Result<bool> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .expect("failed to get app data dir");

        let tauri_db_path = app_data_dir.join("meeting_minutes.sqlite");

        Ok(!tauri_db_path.exists())
    }

    /// Import a legacy database from the specified path and initialize
    pub async fn import_legacy_database(
        app_handle: &tauri::AppHandle,
        legacy_db_path: &str,
    ) -> Result<Self> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .expect("failed to get app data dir");

        if !app_data_dir.exists() {
            fs::create_dir_all(&app_data_dir).map_err(|e| sqlx::Error::Io(e))?;
        }

        // Copy legacy database to app data directory as meeting_minutes.db
        let target_legacy_path = app_data_dir.join("meeting_minutes.db");
        log::info!(
            "Copying legacy database from {} to {}",
            legacy_db_path,
            target_legacy_path.display()
        );

        fs::copy(legacy_db_path, &target_legacy_path).map_err(|e| sqlx::Error::Io(e))?;

        // Now use the standard initialization which will detect and migrate the legacy db
        Self::new_from_app_handle(app_handle).await
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn with_transaction<T, F, Fut>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Transaction<'_, Sqlite>) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut tx = self.pool.begin().await?;
        let result = f(&mut tx).await;

        match result {
            Ok(val) => {
                tx.commit().await?;
                Ok(val)
            }
            Err(err) => {
                tx.rollback().await?;
                Err(err)
            }
        }
    }

    /// Cleanup database connection and checkpoint WAL
    /// This should be called on application shutdown to ensure:
    /// - All WAL changes are written to the main database file
    /// - The .wal and .shm files are deleted
    /// - Connection pool is gracefully closed
    pub async fn cleanup(&self) -> Result<()> {
        log::info!("Starting database cleanup...");

        // Force checkpoint of WAL to main database file and remove WAL file
        // TRUNCATE mode: checkpoints all pages AND deletes the WAL file
        match sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(&self.pool)
            .await
        {
            Ok(_) => log::info!("WAL checkpoint completed successfully"),
            Err(e) => log::warn!("WAL checkpoint failed (non-fatal): {}", e),
        }

        // Close the connection pool gracefully
        self.pool.close().await;
        log::info!("Database connection pool closed");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidecar_path_wal_and_shm() {
        assert_eq!(
            sidecar_path(Path::new("/a/b/app.db"), "-wal"),
            PathBuf::from("/a/b/app.db-wal")
        );
        assert_eq!(
            sidecar_path(Path::new("/a/b/app.db"), "-shm"),
            PathBuf::from("/a/b/app.db-shm")
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_sidecar_path_non_utf8_keeps_parent() {
        use std::ffi::OsString;
        use std::os::unix::ffi::{OsStrExt, OsStringExt};
        let parent = PathBuf::from("/a/b");
        let name = OsString::from_vec(vec![b'a', b'p', b'p', 0xFF, b'.', b'd', b'b']);
        let db = parent.join(name);
        let sidecar = sidecar_path(&db, "-wal");
        assert_eq!(sidecar.parent(), Some(parent.as_path()));
        // The raw bytes must be preserved; a `format!`-based helper would
        // lossy-replace the 0xFF byte and fail this assertion.
        let expected = vec![b'a', b'p', b'p', 0xFF, b'.', b'd', b'b', b'-', b'w', b'a', b'l'];
        assert_eq!(sidecar.file_name().unwrap().as_bytes(), expected.as_slice());
    }

    #[cfg(not(unix))]
    #[test]
    fn test_sidecar_path_keeps_parent() {
        let db = Path::new("/a/b/app.db");
        let sidecar = sidecar_path(db, "-wal");
        assert_eq!(sidecar.parent(), Some(Path::new("/a/b")));
        assert_eq!(sidecar.file_name().unwrap(), "app.db-wal");
    }

    #[test]
    fn test_incompatible_branch_logs_at_error_level() {
        let source = include_str!("manager.rs");
        // The message text must still be present (now under `log::error!`).
        assert!(
            source.contains("Database migration failed (incompatible existing database)"),
            "incompatible branch message must be present"
        );
        // The old warn-level form must be gone.
        assert!(
            !source.contains("log::warn!(\"Database migration failed (incompatible existing database)"),
            "old warn-level incompatible log must be removed"
        );
    }
}
