//! Portable, data-only city archives. Workspaces are remapped explicitly; no files execute.
use crate::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};
const MAX_BYTES: u64 = 64 * 1024 * 1024;
#[derive(Serialize, Deserialize)]
struct Archive {
    format: u32,
    digest: String,
    world: serde_json::Value,
    #[serde(default)]
    commands: serde_json::Value,
}
pub fn export(world: &World, path: &Path) -> Result<()> {
    export_records(world, path, &[])
}
fn export_records(world: &World, path: &Path, records: &[CommandRecord]) -> Result<()> {
    if path.exists() {
        return Err(CityError::new(
            "already_exists",
            "Choose a new archive filename.",
        ));
    }
    let mut portable = world.clone();
    portable.deleted_homes.clear();
    // Default portable export excludes raw provider prompts, tool payloads, and approval requests.
    // The SQLite backup is the full-fidelity private recovery format.
    for run in &mut portable.runs {
        run.payloads_retained = false;
        run.context.clear();
        run.messages.clear();
        for tool in &mut run.tool_receipts {
            tool.arguments = serde_json::Value::Null;
            tool.result = serde_json::Value::Null;
        }
    }
    for decision in &mut portable.decisions {
        decision.arguments = serde_json::Value::Null;
    }
    for task in &mut portable.tasks {
        if task.run_id.is_some() {
            task.checkpoint="Portable history retains tool names and outcomes. Inspect your original database backup for exact provider payloads before resuming uncertain effects.".into();
        }
    }
    let world = serde_json::to_value(portable)?;
    let commands = serde_json::to_value(records)?;
    let bytes = serde_json::to_vec(&(&world, &commands))?;
    if bytes.len() as u64 > MAX_BYTES / 2 {
        return Err(CityError::new(
            "archive_limit",
            "This city exceeds the portable archive size limit. Save a database backup instead.",
        ));
    }
    let archive = Archive {
        format: 2,
        digest: files::digest(&bytes),
        world,
        commands,
    };
    files::atomic_write(path, &serde_json::to_vec(&archive)?)
}
pub fn read(path: &Path) -> Result<World> {
    read_records(path).map(|(world, _)| world)
}
fn read_records(path: &Path) -> Result<(World, Vec<CommandRecord>)> {
    if std::fs::metadata(path)?.len() > MAX_BYTES {
        return Err(CityError::new("archive_limit", "Archive exceeds 64 MiB."));
    }
    let archive: Archive = serde_json::from_slice(&std::fs::read(path)?)?;
    let bytes = match archive.format {
        1 if archive.commands.is_null() => serde_json::to_vec(&archive.world)?,
        2 => serde_json::to_vec(&(&archive.world, &archive.commands))?,
        _ => {
            return Err(CityError::new(
                "invalid_archive",
                "Unsupported archive version.",
            ))
        }
    };
    if archive.world["schemaVersion"] != 1 || files::digest(&bytes) != archive.digest {
        return Err(CityError::new(
            "invalid_archive",
            "Unsupported schema or archive digest mismatch.",
        ));
    }
    let records: Vec<CommandRecord> = if archive.format == 2 {
        serde_json::from_value(archive.commands)?
    } else {
        vec![]
    };
    let mut ids = HashSet::new();
    for record in &records {
        if record.id.is_empty()
            || record.id.len() > 200
            || !ids.insert(record.id.clone())
            || record.digest.len() != 64
            || !record.digest.bytes().all(|b| b.is_ascii_hexdigit())
            || record.receipt.command_id != record.id
            || record.receipt.status != "confirmed"
        {
            return Err(CityError::new(
                "invalid_archive",
                "Invalid or duplicate command retry record.",
            ));
        }
    }
    let world: World = serde_json::from_value(archive.world)?;
    validate(&world)?;
    Ok((world, records))
}
fn validate(w: &World) -> Result<()> {
    let bad = || {
        CityError::new(
            "invalid_archive",
            "Archive identities, relationships, or workspace paths are invalid.",
        )
    };
    if !(1..=4).contains(&w.settings.concurrency)
        || !(1..=100).contains(&w.settings.max_turns)
        || !(1..=32).contains(&w.settings.max_tool_steps)
        || !(1..=365).contains(&w.settings.history_days)
        || !(1..=500).contains(&w.settings.history_mib)
        || !(1..=5000).contains(&w.settings.artifact_mib)
    {
        return Err(bad());
    }
    let mut ids = HashSet::new();
    for id in w
        .projects
        .iter()
        .map(|x| &x.id)
        .chain(w.cities.iter().map(|x| &x.id))
        .chain(w.residents.iter().map(|x| &x.id))
        .chain(w.tasks.iter().map(|x| &x.id))
        .chain(w.runs.iter().map(|x| &x.id))
        .chain(w.roads.iter().map(|x| &x.id))
        .chain(w.messages.iter().map(|x| &x.id))
        .chain(w.memories.iter().map(|x| &x.id))
        .chain(w.decisions.iter().map(|x| &x.id))
        .chain(w.results.iter().map(|x| &x.id))
        .chain(w.acceptance_ledger.iter().map(|x| &x.id))
    {
        if uuid::Uuid::parse_str(id).is_err() || !ids.insert(id) {
            return Err(bad());
        }
    }
    for skill in &w.skills {
        if skill.id != skill.digest
            || skill.digest
                != files::digest(&serde_json::to_vec(&(skill.content.clone(), &skill.files))?)
        {
            return Err(bad());
        }
        if skill.files.iter().any(|(path, _)| {
            let p = Path::new(path);
            p.is_absolute()
                || p.components()
                    .any(|c| matches!(c, std::path::Component::ParentDir))
        }) {
            return Err(bad());
        }
    }
    for p in &w.projects {
        if !Path::new(&p.workspace).is_absolute()
            || Path::new(&p.workspace)
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(bad());
        }
        if !w
            .cities
            .iter()
            .any(|c| c.project_id == p.id && c.mayor_id == p.lead_mayor)
        {
            return Err(bad());
        }
    }
    for c in &w.cities {
        if !w.projects.iter().any(|p| p.id == c.project_id)
            || !w
                .residents
                .iter()
                .any(|r| r.id == c.mayor_id && r.city_id == c.id && r.lifecycle == "active")
        {
            return Err(bad());
        }
    }
    for r in &w.residents {
        if !w.cities.iter().any(|c| c.id == r.city_id) {
            return Err(bad());
        }
        for related in r.ancestors.iter().chain(r.successor.iter()) {
            if related == &r.id
                || !w
                    .residents
                    .iter()
                    .any(|a| a.id == *related && a.city_id == r.city_id)
            {
                return Err(bad());
            }
        }
    }
    for t in &w.tasks {
        if t.assigned_skills.as_ref().is_some_and(|ids| {
            ids.iter()
                .any(|id| !w.skills.iter().any(|skill| skill.id == *id))
        }) {
            return Err(bad());
        }
        if !w
            .cities
            .iter()
            .any(|c| c.id == t.city_id && c.project_id == t.project_id)
            || !w
                .residents
                .iter()
                .any(|r| r.id == t.resident_id && r.city_id == t.city_id)
            || !w
                .tasks
                .iter()
                .any(|g| g.id == t.goal_id && g.project_id == t.project_id)
        {
            return Err(bad());
        }
        for related in t.parent_id.iter().chain(t.reviewed_task.iter()) {
            if related == &t.id
                || !w.tasks.iter().any(|other| {
                    other.id == *related
                        && other.goal_id == t.goal_id
                        && other.project_id == t.project_id
                })
            {
                return Err(bad());
            }
        }
        if t.run_id.as_ref().is_some_and(|id| {
            !w.runs
                .iter()
                .any(|run| run.id == *id && run.task_id == t.id)
        }) {
            return Err(bad());
        }
        let mut pending = t.dependencies.clone();
        let mut seen = HashSet::new();
        while let Some(id) = pending.pop() {
            if id == t.id {
                return Err(bad());
            }
            if !seen.insert(id.clone()) {
                continue;
            }
            let dep = w
                .tasks
                .iter()
                .find(|d| d.id == id && d.goal_id == t.goal_id)
                .ok_or_else(bad)?;
            pending.extend(dep.dependencies.clone());
        }
    }
    for r in &w.runs {
        let task = w.tasks.iter().find(|t| t.id == r.task_id).ok_or_else(bad)?;
        // A mayor handover can change the task owner while preserving prior runs.
        if !w
            .residents
            .iter()
            .any(|a| a.id == r.resident_id && a.city_id == task.city_id)
            || r.input_revisions.keys().any(|id| {
                !w.tasks
                    .iter()
                    .any(|t| t.id == *id && t.goal_id == task.goal_id)
            })
        {
            return Err(bad());
        }
    }
    for road in &w.roads {
        if road.rule_version != 1
            || !["auto", "road", "transit"].contains(&road.presentation.as_str())
        {
            return Err(CityError::new(
                "invalid_archive",
                "Unsupported road presentation rule.",
            ));
        }
        if !w.residents.iter().any(|r| r.id == road.from)
            || !w.residents.iter().any(|r| r.id == road.to)
        {
            return Err(bad());
        }
    }
    for memory in &w.memories {
        if !w.residents.iter().any(|r| r.id == memory.resident_id) {
            return Err(bad());
        }
    }
    for decision in &w.decisions {
        if !w.runs.iter().any(|r| {
            r.id == decision.run_id
                && r.task_id == decision.task_id
                && r.resident_id == decision.resident_id
        }) {
            return Err(bad());
        }
    }
    for message in &w.messages {
        let task = w
            .tasks
            .iter()
            .find(|t| t.id == message.task_id)
            .ok_or_else(bad)?;
        for resident in [&message.from, &message.to] {
            let resident = w
                .residents
                .iter()
                .find(|r| r.id == *resident)
                .ok_or_else(bad)?;
            if !w
                .cities
                .iter()
                .any(|c| c.id == resident.city_id && c.project_id == task.project_id)
            {
                return Err(bad());
            }
        }
        if (!message.goal_id.is_empty() && message.goal_id != task.goal_id)
            || (!message.city_id.is_empty()
                && !w
                    .residents
                    .iter()
                    .any(|r| r.id == message.to && r.city_id == message.city_id))
            || message
                .route
                .iter()
                .any(|id| !w.roads.iter().any(|r| r.id == *id))
            || message.causation_id.as_ref().is_some_and(|id| {
                id == &message.id
                    || !w.messages.iter().any(|m| {
                        m.id == *id
                            && w.tasks
                                .iter()
                                .any(|t| t.id == m.task_id && t.goal_id == task.goal_id)
                    })
            })
        {
            return Err(bad());
        }
    }
    let mut accepted = HashSet::new();
    for entry in &w.acceptance_ledger {
        let goal = w
            .tasks
            .iter()
            .find(|t| t.id == entry.goal_id && t.id == t.goal_id)
            .ok_or_else(bad)?;
        if entry.reversed_at.is_none()
            && (goal.state != TaskState::Accepted || !accepted.insert(&entry.goal_id))
        {
            return Err(bad());
        }
    }
    for result in &w.results {
        let p = w
            .projects
            .iter()
            .find(|p| p.id == result.project_id)
            .ok_or_else(bad)?;
        if !w
            .tasks
            .iter()
            .any(|t| t.id == result.task_id && t.project_id == p.id)
        {
            return Err(bad());
        }
        if let Some(path) = &result.path {
            let path = Path::new(path);
            if !path.starts_with(&p.workspace)
                || path
                    .components()
                    .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                return Err(bad());
            }
        }
    }
    Ok(())
}
impl Runtime {
    pub fn export_archive(&self, path: &Path) -> Result<()> {
        let store = self.store.lock().unwrap();
        export_records(&store.world, path, &store.command_records()?)
    }
    pub fn inspect_archive(&self, path: &Path) -> Result<serde_json::Value> {
        let w = read(path)?;
        Ok(
            serde_json::json!({"projects":w.projects,"residents":w.residents.len(),"tasks":w.tasks.len(),"demo":w.demo}),
        )
    }
    pub fn import_archive(&self, path: &Path, workspaces: &HashMap<String, String>) -> Result<()> {
        let (mut incoming, records) = read_records(path)?;
        // Validate every new root before committing any imported record.
        for project in &mut incoming.projects {
            let root = workspaces.get(&project.id).ok_or_else(|| {
                CityError::new(
                    "workspace_required",
                    "Select a workspace for every imported project.",
                )
            })?;
            let root = std::fs::canonicalize(root)?;
            if !root.is_dir() {
                return Err(CityError::new(
                    "workspace_required",
                    "A project workspace must be a directory.",
                ));
            }
            project.workspace = root.display().to_string();
        }
        for city in &mut incoming.cities {
            city.held = true;
        }
        for r in &mut incoming.residents {
            r.held = true;
            r.model = None;
        }
        for t in &mut incoming.tasks {
            if t.state.executing() || t.state == TaskState::Waiting {
                t.state = TaskState::Unknown;
                t.blocker=Some("Restored execution has an uncertain outcome. Inspect the prior workspace and provider before continuing.".into());
            } else if t.state == TaskState::Queued {
                t.state = TaskState::Paused;
            }
        }
        for d in &mut incoming.decisions {
            if d.state == "pending" || d.state == "allowed" {
                d.state = "invalidated".into();
            }
        }
        for r in &mut incoming.runs {
            r.provider_thread = None;
            r.provider_turn = None;
            if r.ended_at.is_none() {
                r.state = "unknown".into();
            }
        }
        // Deliverable content remains available. Old absolute paths never open on a new machine.
        for r in &mut incoming.results {
            r.path = None;
        }
        for skill in &mut incoming.skills {
            if let Some(reason) = skills::capability_limitation(&skill.content, &skill.files) {
                skill.available = false;
                skill.limitation = Some(reason);
            }
        }
        incoming.deleted_homes.clear();
        incoming.settings.skill_roots.clear();
        incoming.settings.policy = "local_only".into();
        incoming.settings.preferred_model = None;
        let mut store = self.store.lock().unwrap();
        let w = &store.world;
        if w.demo != incoming.demo {
            return Err(CityError::new(
                "namespace_mismatch",
                "Demo and live archives cannot be mixed.",
            ));
        }
        if !w.tasks.is_empty()
            || !w.memories.is_empty()
            || w.residents.len() > 1
            || w.projects.len() > 1
        {
            return Err(CityError::new("restore_requires_empty_city","Restore into a fresh data directory to preserve your existing city. The archive has not changed your data."));
        }
        incoming.event("archive_restored","city","Portable archive and command retry records restored with all cities held and workspaces remapped.");
        store.persist_with_records(incoming, None, &records)?;
        let _ = self.changed.send(store.world.revision);
        Ok(())
    }
}
