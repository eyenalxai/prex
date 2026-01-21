use anyhow::{Context, Result, anyhow};
use rusqlite::{Connection, Row, params};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ModManagerEntry {
    pub appid: String,
    pub exe_path: PathBuf,
    pub is_active: bool,
}

fn row_to_entry(row: &Row) -> rusqlite::Result<ModManagerEntry> {
    Ok(ModManagerEntry {
        appid: row.get(0)?,
        exe_path: PathBuf::from(row.get::<_, String>(1)?),
        is_active: row.get::<_, i64>(2)? != 0,
    })
}

fn db_path() -> Result<PathBuf> {
    Ok(crate::paths::data_dir()?.join("prex.db"))
}

fn open_db() -> Result<Connection> {
    let path = db_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Creating data directory at {}", parent.display()))?;
    }
    let conn = Connection::open(&path)
        .with_context(|| format!("Opening database at {}", path.display()))?;
    init_schema(&conn)?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS mod_managers (
            appid TEXT PRIMARY KEY,
            exe_path TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 0
        );",
    )
    .context("Creating mod_managers table")?;
    Ok(())
}

pub fn add_mod_manager(appid: &str, exe_path: &Path) -> Result<()> {
    let conn = open_db()?;
    conn.execute(
        "INSERT INTO mod_managers (appid, exe_path, is_active)
         VALUES (?1, ?2, COALESCE((SELECT is_active FROM mod_managers WHERE appid = ?1), 0))
         ON CONFLICT(appid) DO UPDATE SET exe_path = excluded.exe_path;",
        params![appid, exe_path.to_string_lossy()],
    )
    .context("Saving mod manager entry")?;
    Ok(())
}

pub fn remove_mod_manager(appid: &str) -> Result<()> {
    let conn = open_db()?;
    let affected = conn
        .execute("DELETE FROM mod_managers WHERE appid = ?1", params![appid])
        .context("Removing mod manager entry")?;
    if affected == 0 {
        return Err(anyhow!("No mod manager registered for appid {appid}"));
    }
    Ok(())
}

pub fn list_mod_managers() -> Result<Vec<ModManagerEntry>> {
    let conn = open_db()?;
    let mut stmt = conn
        .prepare("SELECT appid, exe_path, is_active FROM mod_managers ORDER BY appid")
        .context("Reading mod manager entries")?;
    let rows = stmt
        .query_map([], row_to_entry)
        .context("Mapping mod manager entries")?;
    let mut entries = Vec::new();
    for row in rows {
        entries.push(row.context("Reading mod manager entry row")?);
    }
    Ok(entries)
}

pub fn set_active(appid: &str) -> Result<()> {
    let mut conn = open_db()?;
    let tx = conn.transaction().context("Starting mod manager update")?;
    tx.execute("UPDATE mod_managers SET is_active = 0", [])
        .context("Clearing active mod manager")?;
    let affected = tx
        .execute(
            "UPDATE mod_managers SET is_active = 1 WHERE appid = ?1",
            params![appid],
        )
        .context("Setting active mod manager")?;
    if affected == 0 {
        return Err(anyhow!("No mod manager registered for appid {appid}"));
    }
    tx.commit().context("Committing mod manager update")?;
    Ok(())
}

pub fn get_active() -> Result<Option<ModManagerEntry>> {
    let conn = open_db()?;
    let mut stmt = conn
        .prepare("SELECT appid, exe_path, is_active FROM mod_managers WHERE is_active = 1")
        .context("Reading active mod manager")?;
    let rows = stmt
        .query_map([], row_to_entry)
        .context("Querying active mod manager")?;
    let mut entries = Vec::new();
    for row in rows {
        entries.push(row.context("Reading active mod manager row")?);
    }
    match entries.len() {
        0 => Ok(None),
        1 => Ok(entries.into_iter().next()),
        _ => Err(anyhow!("Multiple active mod managers found")),
    }
}
