use crate::*;
use fs2::FileExt;
use rusqlite::{params, Connection, OptionalExtension};
use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
};

pub struct Store {
    pub(crate) db: Connection,
    pub world: World,
    pub directory: PathBuf,
    _lock: File,
}
impl Drop for Store {
    fn drop(&mut self) {
        // Release explicitly even while a concurrently spawned child is between fork and exec.
        let _ = FileExt::unlock(&self._lock);
    }
}
impl Store {
    pub fn open(directory: &Path, demo: bool) -> Result<Self> {
        fs::create_dir_all(directory)?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(directory.join("instance.lock"))?;
        lock.try_lock_exclusive().map_err(|_| {
            CityError::new(
                "instance_running",
                "Agent City is already using this data directory.",
            )
        })?;
        let db = Connection::open(directory.join("city.sqlite3"))?;
        db.pragma_update(None, "foreign_keys", "ON")?;
        db.pragma_update(None, "journal_mode", "WAL")?;
        db.pragma_update(None, "synchronous", "FULL")?;
        db.pragma_update(None, "secure_delete", "ON")?;
        let version: u32 = db.pragma_query_value(None, "user_version", |r| r.get(0))?;
        if version > 1 {
            return Err(CityError::new(
                "newer_schema",
                "This data directory requires a newer Agent City version.",
            ));
        }
        db.execute_batch("CREATE TABLE IF NOT EXISTS world (id INTEGER PRIMARY KEY CHECK(id=1), revision INTEGER NOT NULL, data TEXT NOT NULL);
          CREATE TABLE IF NOT EXISTS commands (id TEXT PRIMARY KEY, digest TEXT NOT NULL, receipt TEXT NOT NULL);
          CREATE TABLE IF NOT EXISTS outbox (run_id TEXT PRIMARY KEY, task_id TEXT NOT NULL, state TEXT NOT NULL);
          CREATE UNIQUE INDEX IF NOT EXISTS one_active_task ON outbox(task_id) WHERE state IN ('pending','dispatched','unknown');
          CREATE TABLE IF NOT EXISTS events (revision INTEGER PRIMARY KEY, data TEXT NOT NULL);
          PRAGMA user_version=1;")?;
        let data: Option<String> = db
            .query_row("SELECT data FROM world WHERE id=1", [], |r| r.get(0))
            .optional()?;
        let world = match data {
            Some(s) => serde_json::from_str::<World>(&s)?,
            None => World::empty(demo),
        };
        if world.demo != demo {
            return Err(CityError::new(
                "namespace_mismatch",
                "Demo and live data must use separate directories.",
            ));
        }
        let mut store = Self {
            db,
            world,
            directory: directory.to_owned(),
            _lock: lock,
        };
        let mut recovered = store.world.clone();
        for skill in &mut recovered.skills {
            if let Some(reason) = skills::capability_limitation(&skill.content, &skill.files) {
                skill.available = false;
                skill.limitation = Some(reason);
            }
        }
        for task in &mut recovered.tasks {
            if task.state.executing() || task.state == TaskState::Waiting {
                task.state = TaskState::Unknown;
                task.blocker = Some("The app stopped before execution was reconciled. Inspect the last checkpoint and confirm the prior work stopped before continuing.".into());
            }
        }
        for decision in &mut recovered.decisions {
            if decision.state == "pending" {
                decision.state = "invalidated".into();
            }
        }
        for run in &mut recovered.runs {
            if run.ended_at.is_none() {
                run.state = "unknown".into();
            }
        }
        store.persist(recovered, None)?;
        store.repair_homes()?;
        Ok(store)
    }
    pub fn receipt(&self, id: &str, digest: &str) -> Result<Option<Receipt>> {
        let row: Option<(String, String)> = self
            .db
            .query_row(
                "SELECT digest,receipt FROM commands WHERE id=?",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        match row {
            Some((d, r)) if d == digest => Ok(Some(serde_json::from_str(&r)?)),
            Some(_) => Err(CityError::new(
                "duplicate_id",
                "This command ID was already used for a different action.",
            )),
            None => Ok(None),
        }
    }
    pub fn persist(&mut self, next: World, command: Option<(&str, &str, &Receipt)>) -> Result<()> {
        self.persist_with_records(next, command, &[])
    }
    pub fn command_records(&self) -> Result<Vec<CommandRecord>> {
        let mut statement = self
            .db
            .prepare("SELECT id,digest,receipt FROM commands ORDER BY id")?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        rows.map(|row| {
            let (id, digest, receipt) = row?;
            Ok(CommandRecord {
                id,
                digest,
                receipt: serde_json::from_str(&receipt)?,
            })
        })
        .collect()
    }
    pub fn persist_with_records(
        &mut self,
        mut next: World,
        command: Option<(&str, &str, &Receipt)>,
        records: &[CommandRecord],
    ) -> Result<()> {
        crate::retention::prune(&mut next);
        next.revision = self.world.revision + 1;
        let mut changed_homes = std::collections::HashSet::new();
        for r in &next.residents {
            if !self.world.residents.iter().any(|old| {
                old.id == r.id
                    && old.name == r.name
                    && old.role == r.role
                    && old.skills == r.skills
                    && old.revision == r.revision
            }) {
                changed_homes.insert(r.id.clone());
            }
        }
        for m in &next.memories {
            if !self.world.memories.iter().any(|old| {
                old.id == m.id
                    && old.revision == m.revision
                    && old.accepted == m.accepted
                    && old.content == m.content
                    && old.kind == m.kind
            }) {
                changed_homes.insert(m.resident_id.clone());
            }
        }
        for r in &next.runs {
            if r.ended_at.is_some()
                && !self.world.runs.iter().any(|old| {
                    old.id == r.id
                        && old.ended_at == r.ended_at
                        && old.payloads_retained == r.payloads_retained
                })
            {
                changed_homes.insert(r.resident_id.clone());
            }
        }
        let data = serde_json::to_string(&next)?;
        let tx = self.db.transaction()?;
        tx.execute("INSERT INTO world VALUES (1,?,?) ON CONFLICT(id) DO UPDATE SET revision=excluded.revision,data=excluded.data", params![next.revision as i64, data])?;
        if let Some((id, digest, receipt)) = command {
            tx.execute(
                "INSERT INTO commands VALUES (?,?,?)",
                params![id, digest, serde_json::to_string(receipt)?],
            )?;
        }
        for record in records {
            let prior: Option<String> = tx
                .query_row(
                    "SELECT digest FROM commands WHERE id=?",
                    [&record.id],
                    |row| row.get(0),
                )
                .optional()?;
            if prior
                .as_ref()
                .is_some_and(|digest| digest != &record.digest)
            {
                return Err(CityError::new("command_conflict","An imported command ID conflicts with this installation. Restore into a new data directory."));
            }
            tx.execute(
                "INSERT OR IGNORE INTO commands VALUES (?,?,?)",
                params![
                    record.id,
                    record.digest,
                    serde_json::to_string(&record.receipt)?
                ],
            )?;
        }
        for run in &next.runs {
            if self.world.runs.iter().any(|old| {
                old.id == run.id && old.state == run.state && old.ended_at == run.ended_at
            }) {
                continue;
            }
            let state = if run.state == "starting" {
                "pending"
            } else if run.state == "unknown" {
                "unknown"
            } else if run.ended_at.is_none() {
                "dispatched"
            } else {
                "done"
            };
            tx.execute("INSERT INTO outbox VALUES (?,?,?) ON CONFLICT(run_id) DO UPDATE SET state=excluded.state", params![run.id,run.task_id,state])?;
        }
        tx.execute(
            "INSERT INTO events VALUES (?,?)",
            params![
                next.revision as i64,
                serde_json::to_string(&next.events.last())?
            ],
        )?;
        tx.execute(
            "DELETE FROM events WHERE revision < ?",
            [next.revision.saturating_sub(10000) as i64],
        )?;
        tx.execute(
            "DELETE FROM events WHERE julianday(json_extract(data, '$.at')) < julianday('now', ?)",
            [format!("-{} days", next.settings.history_days)],
        )?;
        tx.execute("DELETE FROM events WHERE revision IN (SELECT revision FROM (SELECT revision, SUM(length(CAST(data AS BLOB))) OVER (ORDER BY revision DESC) AS bytes FROM events) WHERE bytes > ?)", [(next.settings.history_mib as i64)*1024*1024])?;
        if next.deleted_homes.len() > self.world.deleted_homes.len() {
            let retained: std::collections::HashSet<&str> = next
                .tasks
                .iter()
                .map(|t| t.id.as_str())
                .chain(next.residents.iter().map(|r| r.id.as_str()))
                .chain(next.projects.iter().map(|p| p.id.as_str()))
                .collect();
            for removed in self
                .world
                .tasks
                .iter()
                .map(|t| t.id.as_str())
                .chain(self.world.residents.iter().map(|r| r.id.as_str()))
                .chain(self.world.projects.iter().map(|p| p.id.as_str()))
                .filter(|id| !retained.contains(id))
            {
                tx.execute(
                    "DELETE FROM events WHERE json_extract(data, '$.subject') = ?",
                    [removed],
                )?;
            }
        }
        for task in next.tasks.iter().filter(|t| {
            t.redacted
                && !self
                    .world
                    .tasks
                    .iter()
                    .any(|old| old.id == t.id && old.redacted)
        }) {
            tx.execute(
                "DELETE FROM events WHERE json_extract(data, '$.subject') = ?",
                [&task.id],
            )?;
            for run in next.runs.iter().filter(|r| r.task_id == task.id) {
                tx.execute(
                    "DELETE FROM events WHERE json_extract(data, '$.subject') = ?",
                    [&run.id],
                )?;
            }
        }
        tx.commit()?;
        self.world = next;
        // Projections are recoverable; SQLite remains authoritative if disk writes fail.
        if let Err(e) = self.repair_homes_for(Some(&changed_homes)) {
            eprintln!("Home projection pending: {}", e.code);
        }
        Ok(())
    }
    pub fn repair_homes(&self) -> Result<()> {
        self.repair_homes_for(None)
    }
    fn repair_homes_for(&self, ids: Option<&std::collections::HashSet<String>>) -> Result<()> {
        if fs::symlink_metadata(self.directory.join("homes"))
            .is_ok_and(|m| m.file_type().is_symlink())
        {
            return Err(CityError::new(
                "invalid_home_root",
                "The resident home directory cannot be a symbolic link.",
            ));
        }
        for id in &self.world.deleted_homes {
            if uuid::Uuid::parse_str(id).is_err()
                || self.world.residents.iter().any(|r| r.id == *id)
            {
                return Err(CityError::new(
                    "invalid_cleanup",
                    "Invalid deleted-home record.",
                ));
            }
            let path = self.directory.join("homes").join(id);
            if let Ok(metadata) = fs::symlink_metadata(&path) {
                if metadata.file_type().is_symlink() {
                    fs::remove_file(&path)?;
                } else {
                    fs::remove_dir_all(&path)?;
                }
            }
        }
        for resident in self
            .world
            .residents
            .iter()
            .filter(|r| ids.is_none_or(|ids| ids.contains(&r.id)))
        {
            let dir = self.directory.join("homes").join(&resident.id);
            fs::create_dir_all(dir.join("skills"))?;
            fs::create_dir_all(dir.join("sessions"))?;
            for skill in self
                .world
                .skills
                .iter()
                .filter(|s| resident.skills.contains(&s.id))
            {
                let version = dir.join("skills").join(&skill.id);
                if !version.join("SKILL.md").exists() {
                    std::fs::create_dir_all(&version)?;
                    files::atomic_write(&version.join("SKILL.md"), skill.content.as_bytes())?;
                    for (path, content) in &skill.files {
                        let relative = Path::new(path);
                        if relative.is_absolute()
                            || relative
                                .components()
                                .any(|c| matches!(c, std::path::Component::ParentDir))
                        {
                            continue;
                        }
                        let target = version.join(relative);
                        if let Some(parent) = target.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        files::atomic_write(&target, content.as_bytes())?;
                    }
                }
            }
            for run in self
                .world
                .runs
                .iter()
                .filter(|run| run.resident_id == resident.id && run.ended_at.is_some())
            {
                let path = dir.join("sessions").join(format!("{}.json", run.id));
                if !path.exists() || !run.payloads_retained {
                    files::atomic_write(&path, &serde_json::to_vec_pretty(run)?)?;
                }
            }
            let notes = self
                .world
                .memories
                .iter()
                .filter(|m| m.resident_id == resident.id && m.accepted)
                .map(|m| {
                    format!(
                        "{}\nSource: {}\nRevision: {}\n",
                        m.content, m.source, m.revision
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            let content = format!(
                "# {}\n\nRole: {}\nIdentity: {}\n\n{}",
                resident.name, resident.role, resident.id, notes
            );
            let path = dir.join("NOTES.md");
            let revision_path = dir.join("projection.sha256");
            let previous = fs::read_to_string(&revision_path).unwrap_or_default();
            if let Ok(existing) = fs::read_to_string(&path) {
                if existing == content {
                    continue;
                }
                if !previous.is_empty() && files::digest(existing.as_bytes()) != previous {
                    // Preserve external edits for explicit import; never overwrite a conflict.
                    files::atomic_write(&dir.join("NOTES.current.md"), content.as_bytes())?;
                    continue;
                }
            }
            files::atomic_write(&path, content.as_bytes())?;
            files::atomic_write(&revision_path, files::digest(content.as_bytes()).as_bytes())?;
        }
        Ok(())
    }
    pub fn backup(&self, destination: &Path) -> Result<()> {
        if destination.exists() {
            return Err(CityError::new(
                "already_exists",
                "Choose a new backup filename.",
            ));
        }
        self.db.backup("main", destination, None)?;
        Ok(())
    }
}
