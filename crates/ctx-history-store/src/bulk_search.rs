//! Crash-safe FTS5 merge suppression and bounded compaction for bulk imports.
//!
//! FTS5 may perform an automatic or crisis merge inside a single row insert,
//! producing a WAL far larger than the imported data. Bulk mode persists a
//! recovery marker before disabling those merges. Event rows and their search
//! projections still commit together; interrupted work remains searchable.
//! Finishing bulk mode restores normal merge settings without compacting the
//! accumulated segments. FTS5 segments are immediately searchable; expensive
//! compaction is reserved for an explicit optimize operation.

use ctx_history_core::utc_now;
use std::{
    ffi::OsString,
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use rusqlite::{params, Connection, ErrorCode, OptionalExtension};

use crate::object_store::restrict_private_file;
use crate::schema::ddl::table_exists;
use crate::{Result, Store, StoreError};

const EVENT_SEARCH_FTS_TABLES: [&str; 2] = ["event_search", "event_search_scriptgram"];
const ALL_FTS_TABLES: [&str; 5] = [
    "ctx_history_search",
    "event_search",
    "artifact_search",
    "ctx_history_search_scriptgram",
    "event_search_scriptgram",
];
const BULK_MODE_MARKER_KEY: &str = "event_search_bulk_mode_v1";
const BULK_MODE_AUTOMERGE_KEY_PREFIX: &str = "event_search_bulk_mode_v1:automerge:";
const BULK_MODE_CRISISMERGE_KEY_PREFIX: &str = "event_search_bulk_mode_v1:crisismerge:";
const FTS_AUTOMERGE_DEFAULT: i64 = 4;
const FTS_CRISISMERGE_DEFAULT: i64 = 16;
const FTS_BULK_CRISISMERGE: i64 = 1_000_000;
// FTS5's merge page budget is not a hard upper bound on WAL pages: merging a
// large segment can rewrite substantially more data inside one statement.
// Keep each step deliberately small so checkpoints remain safe on large real
// indexes, not only on compact synthetic fixtures.
const FTS_MERGE_PAGE_BUDGET: i64 = 16;
const BULK_LOCK_SUFFIX: &str = ".event-search-bulk.lock.sqlite";

/// Owns the cross-process lock for one event-search bulk operation.
///
/// SQLite releases the sidecar database's writer lock if the process exits,
/// including after an unclean exit. The guard intentionally cannot be cloned.
pub struct EventSearchBulkGuard {
    lock_conn: Option<Connection>,
    store_path: PathBuf,
    depth: Arc<AtomicUsize>,
    depth_counted: bool,
    wal_autocheckpoint: Option<i64>,
}

impl Drop for EventSearchBulkGuard {
    fn drop(&mut self) {
        if let Some(lock_conn) = &self.lock_conn {
            let _ = lock_conn.execute_batch("ROLLBACK");
        }
        if self.depth_counted {
            self.depth.fetch_sub(1, Ordering::SeqCst);
        }
    }
}

impl Store {
    /// Acquire the bulk-import lock and persist merge suppression.
    pub fn begin_event_search_bulk_mode(&self) -> Result<EventSearchBulkGuard> {
        if self.event_search_bulk_depth.fetch_add(1, Ordering::SeqCst) > 0 {
            return Ok(EventSearchBulkGuard {
                lock_conn: None,
                store_path: self.path.clone(),
                depth: Arc::clone(&self.event_search_bulk_depth),
                depth_counted: true,
                wal_autocheckpoint: None,
            });
        }
        let acquired = match self.acquire_event_search_bulk_lock(self.busy_timeout) {
            Ok(acquired) => acquired,
            Err(error) => {
                self.event_search_bulk_depth.fetch_sub(1, Ordering::SeqCst);
                return Err(error);
            }
        };
        let mut guard = match acquired {
            Some(guard) => guard,
            None => {
                self.event_search_bulk_depth.fetch_sub(1, Ordering::SeqCst);
                return Err(StoreError::BulkSearchImportBusy);
            }
        };
        guard.depth_counted = true;
        let wal_autocheckpoint = self.wal_autocheckpoint()?;
        self.set_wal_autocheckpoint(0)?;
        guard.wal_autocheckpoint = Some(wal_autocheckpoint);
        if let Err(error) = self.begin_immediate_batch() {
            let _ = self.restore_wal_autocheckpoint(&guard);
            return Err(error);
        }
        let result = (|| {
            ensure_search_projection_stats_table(self)?;
            if !bulk_mode_pending(self)? {
                for table in EVENT_SEARCH_FTS_TABLES {
                    if !table_exists(&self.conn, table)? {
                        continue;
                    }
                    save_bulk_mode_config(
                        self,
                        &format!("{BULK_MODE_AUTOMERGE_KEY_PREFIX}{table}"),
                        fts_config_value(self, table, "automerge", FTS_AUTOMERGE_DEFAULT)?,
                    )?;
                    save_bulk_mode_config(
                        self,
                        &format!("{BULK_MODE_CRISISMERGE_KEY_PREFIX}{table}"),
                        fts_config_value(self, table, "crisismerge", FTS_CRISISMERGE_DEFAULT)?,
                    )?;
                }
                save_bulk_mode_config(self, BULK_MODE_MARKER_KEY, 1)?;
            }
            suppress_event_search_merges(self)
        })();
        if let Err(err) = result {
            let _ = self.rollback_batch();
            let _ = self.restore_wal_autocheckpoint(&guard);
            return Err(err);
        }
        if let Err(err) = self.commit_batch() {
            let _ = self.rollback_batch();
            let _ = self.restore_wal_autocheckpoint(&guard);
            return Err(err);
        }
        Ok(guard)
    }

    /// Restore normal FTS merge settings after a bulk import.
    ///
    /// Pending segments remain searchable. Rewriting them here makes setup's
    /// latency and temporary disk usage proportional to the entire existing
    /// index, so compaction belongs to the explicit optimize path instead.
    pub fn finish_event_search_bulk_mode(&self, guard: &EventSearchBulkGuard) -> Result<()> {
        if guard.store_path != self.path {
            return Err(StoreError::InvalidBulkSearchGuard);
        }
        if guard.lock_conn.is_none() {
            return Ok(());
        }
        if guard.depth_counted && guard.depth.load(Ordering::SeqCst) != 1 {
            return Err(StoreError::InvalidBulkSearchGuard);
        }
        let finish_result = self.restore_event_search_bulk_mode();
        let restore_result = self.restore_wal_autocheckpoint(guard);
        match (finish_result, restore_result) {
            (Ok(()), Ok(())) => Ok(()),
            (_, Err(error)) => Err(error),
            (Err(error), Ok(())) => Err(error),
        }
    }

    pub(crate) fn recover_event_search_bulk_mode(&self) -> Result<()> {
        if !bulk_mode_pending(self)? {
            return Ok(());
        }
        // A live importer owns this lock. A stale marker has no owner, so the
        // next writable open adopts and completes its bounded recovery.
        if let Some(guard) = self.acquire_event_search_bulk_lock(Duration::ZERO)? {
            self.finish_event_search_bulk_mode(&guard)?;
        }
        Ok(())
    }

    pub(crate) fn merge_all_fts_tables_bounded(&self) -> Result<()> {
        // Serialize unconditionally. Reading the marker before acquiring the
        // lock would let a new bulk import start in the handoff window.
        let guard = self
            .acquire_event_search_bulk_lock(self.busy_timeout)?
            .ok_or(StoreError::BulkSearchImportBusy)?;
        if bulk_mode_pending(self)? {
            self.finish_event_search_bulk_mode(&guard)?;
        }
        for table in ALL_FTS_TABLES {
            self.merge_fts_table_bounded(table, true)?;
        }
        Ok(())
    }

    fn merge_fts_table_bounded(
        &self,
        table: &'static str,
        mut start_full_merge: bool,
    ) -> Result<()> {
        if !table_exists(&self.conn, table)? {
            return Ok(());
        }
        loop {
            let page_budget = if start_full_merge {
                -FTS_MERGE_PAGE_BUDGET
            } else {
                FTS_MERGE_PAGE_BUDGET
            };
            let changed = self.merge_fts_table_step(table, page_budget)?;
            start_full_merge = false;
            if !changed {
                return Ok(());
            }
        }
    }

    fn merge_fts_table_step(&self, table: &'static str, page_budget: i64) -> Result<bool> {
        self.begin_immediate_batch()?;
        let result = merge_fts_table_in_transaction(self, table, page_budget);
        let changed = match result {
            Ok(changed) => changed,
            Err(err) => {
                let _ = self.rollback_batch();
                return Err(err);
            }
        };
        if let Err(err) = self.commit_batch() {
            let _ = self.rollback_batch();
            return Err(err);
        }
        self.checkpoint_wal_truncate_required()?;
        Ok(changed)
    }

    fn restore_event_search_bulk_mode(&self) -> Result<()> {
        self.begin_immediate_batch()?;
        let result = (|| {
            if !bulk_mode_pending(self)? {
                return Ok(());
            }
            restore_event_search_merge_config(self)?;
            clear_bulk_mode_state(self)
        })();
        match result {
            Ok(()) => {}
            Err(err) => {
                let _ = self.rollback_batch();
                return Err(err);
            }
        };
        if let Err(err) = self.commit_batch() {
            let _ = self.rollback_batch();
            return Err(err);
        }
        Ok(())
    }

    fn acquire_event_search_bulk_lock(
        &self,
        busy_timeout: Duration,
    ) -> Result<Option<EventSearchBulkGuard>> {
        let lock_path = event_search_bulk_lock_path(&self.path);
        let lock_conn = Connection::open(&lock_path)?;
        restrict_private_file(&lock_path)?;
        lock_conn.busy_timeout(busy_timeout)?;
        let result = lock_conn.execute_batch(
            "PRAGMA journal_mode=DELETE;\
             CREATE TABLE IF NOT EXISTS bulk_search_lock (id INTEGER PRIMARY KEY);\
             BEGIN IMMEDIATE",
        );
        match result {
            Ok(()) => Ok(Some(EventSearchBulkGuard {
                lock_conn: Some(lock_conn),
                store_path: self.path.clone(),
                depth: Arc::clone(&self.event_search_bulk_depth),
                depth_counted: false,
                wal_autocheckpoint: None,
            })),
            Err(err) if sqlite_is_busy(&err) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    fn wal_autocheckpoint(&self) -> Result<i64> {
        Ok(self
            .conn
            .pragma_query_value(None, "wal_autocheckpoint", |row| row.get(0))?)
    }

    fn set_wal_autocheckpoint(&self, pages: i64) -> Result<()> {
        self.conn.pragma_update(None, "wal_autocheckpoint", pages)?;
        Ok(())
    }

    fn restore_wal_autocheckpoint(&self, guard: &EventSearchBulkGuard) -> Result<()> {
        if let Some(pages) = guard.wal_autocheckpoint {
            self.set_wal_autocheckpoint(pages)?;
        }
        Ok(())
    }
}

fn merge_fts_table_in_transaction(
    store: &Store,
    table: &'static str,
    page_budget: i64,
) -> Result<bool> {
    let before = store.conn.total_changes();
    let sql = format!("INSERT INTO {table}({table}, rank) VALUES ('merge', ?1)");
    store.conn.execute(&sql, params![page_budget])?;
    Ok(store.conn.total_changes().saturating_sub(before) >= 2)
}

fn event_search_bulk_lock_path(store_path: &std::path::Path) -> PathBuf {
    let mut value = OsString::from(store_path.as_os_str());
    value.push(BULK_LOCK_SUFFIX);
    PathBuf::from(value)
}

fn sqlite_is_busy(err: &rusqlite::Error) -> bool {
    matches!(
        err,
        rusqlite::Error::SqliteFailure(failure, _)
            if matches!(failure.code, ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked)
    )
}

fn suppress_event_search_merges(store: &Store) -> Result<()> {
    for table in EVENT_SEARCH_FTS_TABLES {
        if !table_exists(&store.conn, table)? {
            continue;
        }
        set_fts_config(store, table, "automerge", 0)?;
        set_fts_config(store, table, "crisismerge", FTS_BULK_CRISISMERGE)?;
    }
    Ok(())
}

fn restore_event_search_merge_config(store: &Store) -> Result<()> {
    for table in EVENT_SEARCH_FTS_TABLES {
        if !table_exists(&store.conn, table)? {
            continue;
        }
        let automerge =
            bulk_mode_config(store, &format!("{BULK_MODE_AUTOMERGE_KEY_PREFIX}{table}"))?
                .unwrap_or(FTS_AUTOMERGE_DEFAULT);
        let crisismerge =
            bulk_mode_config(store, &format!("{BULK_MODE_CRISISMERGE_KEY_PREFIX}{table}"))?
                .unwrap_or(FTS_CRISISMERGE_DEFAULT);
        set_fts_config(store, table, "automerge", automerge)?;
        set_fts_config(store, table, "crisismerge", crisismerge)?;
    }
    Ok(())
}

fn set_fts_config(store: &Store, table: &'static str, key: &str, value: i64) -> Result<()> {
    debug_assert!(ALL_FTS_TABLES.contains(&table));
    let sql = format!("INSERT INTO {table}({table}, rank) VALUES (?1, ?2)");
    store.conn.execute(&sql, params![key, value])?;
    Ok(())
}

fn fts_config_value(store: &Store, table: &'static str, key: &str, default: i64) -> Result<i64> {
    debug_assert!(ALL_FTS_TABLES.contains(&table));
    let sql = format!("SELECT v FROM {table}_config WHERE k = ?1");
    Ok(store
        .conn
        .query_row(&sql, params![key], |row| row.get(0))
        .optional()?
        .unwrap_or(default))
}

fn ensure_search_projection_stats_table(store: &Store) -> Result<()> {
    store.conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS search_projection_stats (
            key TEXT PRIMARY KEY NOT NULL,
            value INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL
        )
        "#,
        [],
    )?;
    Ok(())
}

fn bulk_mode_pending(store: &Store) -> Result<bool> {
    if !table_exists(&store.conn, "search_projection_stats")? {
        return Ok(false);
    }
    Ok(bulk_mode_config(store, BULK_MODE_MARKER_KEY)?.is_some())
}

fn bulk_mode_config(store: &Store, key: &str) -> Result<Option<i64>> {
    Ok(store
        .conn
        .query_row(
            "SELECT value FROM search_projection_stats WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()?)
}

fn save_bulk_mode_config(store: &Store, key: &str, value: i64) -> Result<()> {
    store.conn.execute(
        r#"
        INSERT INTO search_projection_stats (key, value, updated_at_ms)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(key) DO UPDATE SET
            value = excluded.value,
            updated_at_ms = excluded.updated_at_ms
        "#,
        params![key, value, utc_now().timestamp_millis()],
    )?;
    Ok(())
}

fn clear_bulk_mode_state(store: &Store) -> Result<()> {
    store.conn.execute(
        "DELETE FROM search_projection_stats WHERE key = ?1 OR key LIKE ?2",
        params![BULK_MODE_MARKER_KEY, "event_search_bulk_mode_v1:%"],
    )?;
    Ok(())
}
