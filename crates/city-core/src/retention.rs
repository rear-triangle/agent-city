//! Bounded detail retention and explicit project lifecycle controls.
use crate::*;
use serde_json::{json, Value};
use std::collections::HashSet;

pub fn require_live_project(world: &World, id: &str) -> Result<()> {
    let project = world
        .projects
        .iter()
        .find(|p| p.id == id)
        .ok_or_else(|| CityError::new("not_found", "Project not found."))?;
    if project.archived {
        return Err(CityError::new(
            "project_archived",
            "Restore this archived project before starting work.",
        ));
    }
    Ok(())
}
pub fn archive_project(world: &mut World, id: &str, archived: bool) -> Result<()> {
    if world
        .tasks
        .iter()
        .any(|t| t.project_id == id && (t.state.executing() || t.state == TaskState::Waiting))
    {
        return Err(CityError::new(
            "work_unresolved",
            "Stop or reconcile this project's active work before changing its archive state.",
        ));
    }
    let project = world
        .projects
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or_else(|| CityError::new("not_found", "Project not found."))?;
    project.archived = archived;
    project.revision += 1;
    let cities: HashSet<String> = world
        .cities
        .iter()
        .filter(|c| c.project_id == id)
        .map(|c| c.id.clone())
        .collect();
    for city in world.cities.iter_mut().filter(|c| cities.contains(&c.id)) {
        city.held = true;
    }
    for resident in world
        .residents
        .iter_mut()
        .filter(|r| cities.contains(&r.city_id))
    {
        resident.held = true;
    }
    for task in world
        .tasks
        .iter_mut()
        .filter(|t| t.project_id == id && t.state == TaskState::Queued)
    {
        task.state = TaskState::Paused;
        task.revision += 1;
    }
    world.event(
        "project_archive",
        id,
        if archived {
            "Project archived with history retained and dispatch held."
        } else {
            "Project restored; dispatch remains held until explicitly continued."
        },
    );
    Ok(())
}
fn scope(world: &World, id: &str) -> Result<Value> {
    let project = world
        .projects
        .iter()
        .find(|p| p.id == id)
        .ok_or_else(|| CityError::new("not_found", "Project not found."))?;
    let cities: Vec<_> = world.cities.iter().filter(|c| c.project_id == id).collect();
    let residents: Vec<_> = world
        .residents
        .iter()
        .filter(|r| cities.iter().any(|c| c.id == r.city_id))
        .collect();
    let tasks: Vec<_> = world.tasks.iter().filter(|t| t.project_id == id).collect();
    Ok(
        json!({"project":project,"cities":cities,"residents":residents,"tasks":tasks,
        "runs":world.runs.iter().filter(|r|tasks.iter().any(|t|t.id==r.task_id)).collect::<Vec<_>>(),
        "roads":world.roads.iter().filter(|road|residents.iter().any(|r|r.id==road.from||r.id==road.to)).collect::<Vec<_>>(),
        "messages":world.messages.iter().filter(|m|tasks.iter().any(|t|t.id==m.task_id)).collect::<Vec<_>>(),
        "decisions":world.decisions.iter().filter(|d|tasks.iter().any(|t|t.id==d.task_id)).collect::<Vec<_>>(),
        "memories":world.memories.iter().filter(|m|residents.iter().any(|r|r.id==m.resident_id)).collect::<Vec<_>>(),
        "results":world.results.iter().filter(|r|r.project_id==id).collect::<Vec<_>>(),
        "acceptances":world.acceptance_ledger.iter().filter(|a|tasks.iter().any(|t|t.id==a.goal_id)).collect::<Vec<_>>() }),
    )
}
pub fn deletion_preview(world: &World, id: &str) -> Result<Value> {
    let scoped = scope(world, id)?;
    let mut counts = serde_json::Map::new();
    for key in [
        "cities",
        "residents",
        "tasks",
        "runs",
        "roads",
        "messages",
        "decisions",
        "memories",
        "results",
        "acceptances",
    ] {
        counts.insert(key.into(), json!(scoped[key].as_array().unwrap().len()));
    }
    Ok(
        json!({"projectId":id,"name":scoped["project"]["name"],"archived":scoped["project"]["archived"],"digest":files::digest(&serde_json::to_vec(&scoped)?),"counts":counts,"allowed":scoped["project"]["archived"]==true&&world.projects.len()>1&&!world.tasks.iter().any(|t|t.project_id==id&&(t.state.executing()||t.state==TaskState::Waiting)),"notice":"Deletes this project's application records and resident home projections. Project workspace files, independent provider history, backups, and previously exported archives remain. Command deduplication IDs remain. This is logical deletion, not a secure-erasure guarantee. Create another project before deleting the last project."}),
    )
}
pub fn delete_project(world: &mut World, id: &str, digest: &str) -> Result<()> {
    let preview = deletion_preview(world, id)?;
    if preview["digest"] != digest {
        return Err(CityError::new(
            "stale_deletion",
            "The project changed. Review a fresh deletion preview.",
        ));
    }
    if preview["allowed"] != true {
        return Err(CityError::new(
            "deletion_unavailable",
            "Archive and reconcile this project first. Keep at least one other project.",
        ));
    }
    let scoped = scope(world, id)?;
    let ids: HashSet<String> = scoped
        .as_object()
        .unwrap()
        .values()
        .filter_map(Value::as_array)
        .flatten()
        .filter_map(|item| item["id"].as_str().map(String::from))
        .chain(std::iter::once(id.to_string()))
        .collect();
    world.deleted_homes.extend(
        world
            .residents
            .iter()
            .filter(|r| ids.contains(&r.id))
            .map(|r| r.id.clone()),
    );
    world.projects.retain(|p| p.id != id);
    world.cities.retain(|c| !ids.contains(&c.id));
    world.residents.retain(|r| !ids.contains(&r.id));
    world.tasks.retain(|t| !ids.contains(&t.id));
    world.runs.retain(|r| !ids.contains(&r.id));
    world.roads.retain(|r| !ids.contains(&r.id));
    world.messages.retain(|m| !ids.contains(&m.id));
    world.decisions.retain(|d| !ids.contains(&d.id));
    world.memories.retain(|m| !ids.contains(&m.id));
    world.results.retain(|r| !ids.contains(&r.id));
    world.acceptance_ledger.retain(|a| !ids.contains(&a.id));
    world.events.retain(|e| !ids.contains(&e.subject));
    world.event("project_deleted",id,"Project application records deleted; external workspace, provider history, and exports unchanged.");
    Ok(())
}
impl Runtime {
    pub fn cleanup_status(&self, retry: bool) -> Result<Value> {
        let store = self.store.lock().unwrap();
        if retry {
            store.repair_homes()?;
        }
        let pending = store
            .world
            .deleted_homes
            .iter()
            .filter(|id| store.directory.join("homes").join(id).exists())
            .count();
        Ok(json!({"pendingHomes":pending}))
    }

    pub fn deletion_preview(&self, project_id: &str) -> Result<Value> {
        deletion_preview(&self.snapshot(), project_id)
    }
}

pub fn prune(world: &mut World) {
    for message in &mut world.messages {
        if provider::message_expired(message)
            && ["delivered", "blocked", "queued"].contains(&message.state.as_str())
        {
            message.state = "expired".into();
        }
    }
    let cutoff = chrono::Utc::now() - chrono::Duration::days(world.settings.history_days as i64);
    let closed: HashSet<_> = world
        .tasks
        .iter()
        .filter(|t| {
            t.id == t.goal_id
                && t.state.finished()
                && world
                    .tasks
                    .iter()
                    .filter(|member| member.goal_id == t.id)
                    .all(|member| member.state.finished())
        })
        .map(|t| t.id.clone())
        .collect();
    let eligible: HashSet<_> = world
        .tasks
        .iter()
        .filter(|t| {
            closed.contains(&t.goal_id) && !t.state.executing() && t.state != TaskState::Waiting
        })
        .map(|t| t.id.clone())
        .collect();
    let mut total: usize = world
        .runs
        .iter()
        .filter(|r| r.payloads_retained)
        .map(payload_bytes)
        .sum();
    let limit = world.settings.history_mib as usize * 1024 * 1024;
    for run in &mut world.runs {
        if !run.payloads_retained || run.ended_at.is_none() || !eligible.contains(&run.task_id) {
            continue;
        }
        let old = run
            .ended_at
            .as_ref()
            .and_then(|at| chrono::DateTime::parse_from_rfc3339(at).ok())
            .is_some_and(|at| at < cutoff);
        if old || total > limit {
            total = total.saturating_sub(payload_bytes(run));
            run.context.clear();
            run.messages.clear();
            for receipt in &mut run.tool_receipts {
                receipt.arguments = Value::Null;
                receipt.result = Value::Null;
            }
            run.payloads_retained = false;
        }
    }
    let expired: HashSet<_> = world
        .runs
        .iter()
        .filter(|r| !r.payloads_retained)
        .map(|r| r.task_id.clone())
        .collect();
    for task in world
        .tasks
        .iter_mut()
        .filter(|t| eligible.contains(&t.id) && expired.contains(&t.id))
    {
        task.output = task.output.chars().take(4000).collect();
    }
    for message in world
        .messages
        .iter_mut()
        .filter(|m| eligible.contains(&m.task_id) && expired.contains(&m.task_id))
    {
        message.body = message.body.chars().take(512).collect();
    }
    for decision in world
        .decisions
        .iter_mut()
        .filter(|d| eligible.contains(&d.task_id) && d.state != "pending")
    {
        if world
            .runs
            .iter()
            .any(|r| r.id == decision.run_id && !r.payloads_retained)
        {
            decision.arguments = Value::Null;
            decision.reason="Operation detail expired under history retention. Decision identity, scope, digest, and outcome remain.".into();
        }
    }
    let mut artifacts: usize = world.results.iter().map(|r| r.content.len()).sum();
    for result in world
        .results
        .iter_mut()
        .filter(|r| eligible.contains(&r.task_id))
    {
        if artifacts <= world.settings.artifact_mib as usize * 1024 * 1024 {
            break;
        }
        let replacement="Retained artifact copy expired under the configured size limit. Original files, if present, remain in the project workspace.";
        if result.content.len() > replacement.len() {
            artifacts -= result.content.len() - replacement.len();
            result.content = replacement.into();
        }
    }
    world.events.retain(|event| {
        chrono::DateTime::parse_from_rfc3339(&event.at).is_ok_and(|at| at >= cutoff)
    });
}
fn payload_bytes(run: &Run) -> usize {
    run.context.len()
        + serde_json::to_vec(&run.messages).map_or(0, |v| v.len())
        + run
            .tool_receipts
            .iter()
            .map(|r| serde_json::to_vec(&(&r.arguments, &r.result)).map_or(0, |v| v.len()))
            .sum::<usize>()
}

fn goal_scope(world: &World, id: &str) -> Result<Value> {
    if !world.tasks.iter().any(|t| t.id == id && t.goal_id == id) {
        return Err(CityError::new("not_found", "Goal not found."));
    }
    let tasks: Vec<_> = world.tasks.iter().filter(|t| t.goal_id == id).collect();
    let runs: Vec<_> = world
        .runs
        .iter()
        .filter(|r| tasks.iter().any(|t| t.id == r.task_id))
        .collect();
    Ok(json!({"tasks":tasks,"runs":runs,
        "messages":world.messages.iter().filter(|m|tasks.iter().any(|t|t.id==m.task_id)).collect::<Vec<_>>(),
        "decisions":world.decisions.iter().filter(|d|tasks.iter().any(|t|t.id==d.task_id)).collect::<Vec<_>>(),
        "memories":world.memories.iter().filter(|m|runs.iter().any(|r|m.source.starts_with(&format!("Run {}:",r.id)))).collect::<Vec<_>>(),
        "results":world.results.iter().filter(|r|tasks.iter().any(|t|t.id==r.task_id)).collect::<Vec<_>>() }))
}
pub fn redaction_preview(world: &World, id: &str) -> Result<Value> {
    let scope = goal_scope(world, id)?;
    let counts: serde_json::Map<String, Value> = scope
        .as_object()
        .unwrap()
        .iter()
        .map(|(key, value)| (key.clone(), json!(value.as_array().unwrap().len())))
        .collect();
    Ok(
        json!({"goalId":id,"digest":files::digest(&serde_json::to_vec(&scope)?),"counts":counts,"allowed":world.tasks.iter().filter(|t|t.goal_id==id).all(|t|t.state.finished()),"notice":"Removes this finished goal's text, raw run payloads, messages, task notes, approval details, and retained result content. Identity, tool names, outcomes, digests, and acceptance history remain. This goal cannot resume afterward. Project files, externally edited home notes, independent provider history, backups, and prior exports are unchanged. This is logical redaction, not secure erasure."}),
    )
}
pub fn redact_goal(world: &mut World, id: &str, digest: &str) -> Result<()> {
    let preview = redaction_preview(world, id)?;
    if preview["digest"] != digest {
        return Err(CityError::new(
            "stale_redaction",
            "Goal details changed. Review a new preview.",
        ));
    }
    if preview["allowed"] != true {
        return Err(CityError::new(
            "work_unresolved",
            "Finish or cancel every assignment in this goal before redaction.",
        ));
    }
    let scoped = goal_scope(world, id)?;
    let ids: HashSet<String> = scoped
        .as_object()
        .unwrap()
        .values()
        .filter_map(Value::as_array)
        .flatten()
        .filter_map(|item| item["id"].as_str().map(String::from))
        .collect();
    for task in world.tasks.iter_mut().filter(|t| ids.contains(&t.id)) {
        task.title = "Redacted goal history".into();
        task.instructions.clear();
        task.criteria.clear();
        task.output.clear();
        task.checkpoint = "Content redacted by the Chief; identity and outcome retained.".into();
        task.blocker = None;
        task.pending_direction = None;
        task.redacted = true;
        task.revision += 1;
    }
    for run in world.runs.iter_mut().filter(|r| ids.contains(&r.id)) {
        run.context.clear();
        run.messages.clear();
        run.provider_thread = None;
        run.provider_turn = None;
        run.payloads_retained = false;
        for receipt in &mut run.tool_receipts {
            receipt.arguments = Value::Null;
            receipt.result = Value::Null;
        }
    }
    for message in world.messages.iter_mut().filter(|m| ids.contains(&m.id)) {
        message.body = "Content redacted by the Chief.".into();
        if message.purpose != "result ready" {
            message.purpose = "redacted".into();
        }
    }
    for decision in world.decisions.iter_mut().filter(|d| ids.contains(&d.id)) {
        decision.arguments = Value::Null;
        decision.reason = "Content redacted by the Chief.".into();
        decision.scope.clear();
    }
    for memory in world.memories.iter_mut().filter(|m| ids.contains(&m.id)) {
        memory.content.clear();
        memory.source = "Redacted task note".into();
        memory.accepted = false;
        memory.kind = "redacted".into();
        memory.revision += 1;
    }
    for result in world.results.iter_mut().filter(|r| ids.contains(&r.id)) {
        result.name = "Redacted result".into();
        result.content = "Content redacted by the Chief.".into();
        result.path = None;
    }
    world.events.retain(|e| !ids.contains(&e.subject));
    world.event(
        "goal_redacted",
        id,
        "Goal content redacted by the Chief; original outcomes and acceptance history retained.",
    );
    Ok(())
}
impl Runtime {
    pub fn redaction_preview(&self, goal_id: &str) -> Result<Value> {
        redaction_preview(&self.snapshot(), goal_id)
    }
}
