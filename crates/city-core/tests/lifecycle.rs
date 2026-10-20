use city_core::*;
use tempfile::TempDir;
fn command(action: Action) -> Command {
    Command {
        schema_version: 1,
        id: id(),
        expected_revision: None,
        action,
    }
}
fn assign(rt: &Runtime) -> Command {
    let w = rt.snapshot();
    command(Action::Assign {
        project_id: w.projects[0].id.clone(),
        city_id: w.cities[0].id.clone(),
        resident_id: None,
        title: "Compare two local approaches".into(),
        criteria: "Give a recommendation and a limitation".into(),
    })
}
#[test]
fn mayor_only_startup_is_persistent_and_single_instance() {
    let dir = TempDir::new().unwrap();
    let rt = Runtime::open(dir.path(), false).unwrap();
    let w = rt.snapshot();
    assert_eq!(w.residents.len(), 1);
    assert_eq!(w.cities[0].mayor_id, w.residents[0].id);
    assert!(w.runs.is_empty());
    assert!(Runtime::open(dir.path(), false).is_err());
    drop(rt);
    let restored = Runtime::open(dir.path(), false).unwrap().snapshot();
    assert_eq!(restored.residents[0].id, w.residents[0].id);
}
#[test]
fn command_retry_cannot_duplicate_or_change_assignment() {
    let dir = TempDir::new().unwrap();
    let rt = Runtime::open(dir.path(), false).unwrap();
    let c = assign(&rt);
    let first = rt.command(c.clone()).unwrap();
    let second = rt.command(c.clone()).unwrap();
    assert_eq!(first.target_id, second.target_id);
    assert_eq!(rt.snapshot().tasks.len(), 1);
    let mut reused = c;
    reused.action = Action::Configure {
        settings: Settings::default(),
    };
    assert_eq!(rt.command(reused).unwrap_err().code, "duplicate_id");
}
#[test]
fn wire_command_round_trip_matches_ui_envelope() {
    let dir = TempDir::new().unwrap();
    let rt = Runtime::open(dir.path(), false).unwrap();
    let c = assign(&rt);
    let json = serde_json::to_value(&c).unwrap();
    let roundtrip: Command = serde_json::from_value(json).unwrap();
    assert!(rt.command(roundtrip).is_ok());
}
#[test]
fn stale_memory_cannot_overwrite_owner_instruction() {
    let dir = TempDir::new().unwrap();
    let rt = Runtime::open(dir.path(), false).unwrap();
    let resident = rt.snapshot().residents[0].id.clone();
    rt.command(command(Action::SaveMemory {
        resident_id: resident.clone(),
        content: "Keep source references".into(),
        base_revision: 0,
    }))
    .unwrap();
    let error = rt
        .command(command(Action::SaveMemory {
            resident_id: resident.clone(),
            content: "Drop the sources".into(),
            base_revision: 0,
        }))
        .unwrap_err();
    assert_eq!(error.code, "stale_revision");
    assert_eq!(rt.snapshot().memories.len(), 1);
    assert!(
        std::fs::read_to_string(dir.path().join("homes").join(resident).join("NOTES.md"))
            .unwrap()
            .contains("Keep source references")
    );
}
#[test]
fn external_home_edit_survives_projection_repair() {
    let dir = TempDir::new().unwrap();
    let rt = Runtime::open(dir.path(), false).unwrap();
    let r = rt.snapshot().residents[0].id.clone();
    let path = dir.path().join("homes").join(&r).join("NOTES.md");
    std::fs::write(&path, "External note").unwrap();
    rt.command(command(Action::SaveMemory {
        resident_id: r,
        content: "Chief instruction".into(),
        base_revision: 0,
    }))
    .unwrap();
    assert_eq!(std::fs::read_to_string(path).unwrap(), "External note");
}
#[test]
fn project_files_reject_traversal_and_symlink_escape() {
    let root = TempDir::new().unwrap();
    let external = TempDir::new().unwrap();
    std::fs::write(external.path().join("private.txt"), "private").unwrap();
    std::os::unix::fs::symlink(external.path(), root.path().join("link")).unwrap();
    let base = root.path().to_str().unwrap();
    assert!(files::read(base, "../private.txt").is_err());
    assert!(files::read(base, "link/private.txt").is_err());
    assert!(files::workspace_path(base, "link/output.txt", false).is_err());
}
#[test]
fn backup_restores_identity_without_copying_wal() {
    let dir = TempDir::new().unwrap();
    let target = TempDir::new().unwrap();
    let rt = Runtime::open(dir.path(), false).unwrap();
    let task = rt.command(assign(&rt)).unwrap().target_id;
    rt.backup(&target.path().join("city.sqlite3")).unwrap();
    let restored = Runtime::open(target.path(), false).unwrap();
    assert_eq!(restored.snapshot().tasks[0].id, task.unwrap());
}
#[test]
fn demo_and_live_stores_cannot_be_mixed() {
    let dir = TempDir::new().unwrap();
    drop(Runtime::open(dir.path(), true).unwrap());
    assert!(Runtime::open(dir.path(), false).is_err());
}
#[tokio::test]
async fn demo_delegates_reviews_then_waits_for_owner_acceptance() {
    let dir = TempDir::new().unwrap();
    let rt = Runtime::open(dir.path(), true).unwrap();
    let goal = rt.command(assign(&rt)).unwrap().target_id.unwrap();
    rt.start();
    tokio::time::timeout(std::time::Duration::from_secs(20), async {
        loop {
            let w = rt.snapshot();
            if w.tasks
                .iter()
                .any(|t| t.id == goal && t.state == TaskState::Review)
            {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    })
    .await
    .unwrap();
    let w = rt.snapshot();
    assert_eq!(w.residents.len(), 2);
    assert_eq!(
        w.tasks
            .iter()
            .filter(|t| t.kind == "work" && t.state == TaskState::Accepted)
            .count(),
        1
    );
    assert!(!w.messages.is_empty());
    assert!(w
        .tasks
        .iter()
        .find(|t| t.id == goal)
        .unwrap()
        .accepted_at
        .is_none());
    let c = command(Action::Accept {
        task_id: goal.clone(),
    });
    rt.command(c.clone()).unwrap();
    rt.command(c).unwrap();
    assert_eq!(
        rt.snapshot()
            .tasks
            .iter()
            .filter(|t| t.id == goal && t.accepted_at.is_some())
            .count(),
        1
    );
    let archive_path = dir.path().join("completed.agentcity");
    rt.export_archive(&archive_path).unwrap();
    let restored = archive::read(&archive_path).unwrap();
    assert_eq!(restored.acceptance_ledger.len(), 1);
    assert_eq!(restored.runs.len(), rt.snapshot().runs.len());
    // A valid checksum does not make broken or cross-owned relationships valid.
    let original: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&archive_path).unwrap()).unwrap();
    for pointer in [
        "/world/projects/0/leadMayor",
        "/world/tasks/0/parentId",
        "/world/tasks/0/runId",
        "/world/runs/0/residentId",
        "/world/messages/0/to",
        "/world/messages/0/goalId",
        "/world/results/0/taskId",
        "/world/acceptanceLedger/0/goalId",
    ] {
        let mut damaged = original.clone();
        *damaged.pointer_mut(pointer).expect(pointer) = id().into();
        damaged["digest"] =
            files::digest(&serde_json::to_vec(&(&damaged["world"], &damaged["commands"])).unwrap())
                .into();
        std::fs::write(&archive_path, serde_json::to_vec(&damaged).unwrap()).unwrap();
        assert_eq!(
            archive::read(&archive_path).unwrap_err().code,
            "invalid_archive",
            "{pointer}"
        );
    }
}

#[test]
fn portable_archive_remaps_workspaces_preserves_identity_and_holds_dispatch() {
    let source = TempDir::new().unwrap();
    let destination = TempDir::new().unwrap();
    let archive_dir = TempDir::new().unwrap();
    let rt = Runtime::open(source.path(), false).unwrap();
    rt.command(assign(&rt)).unwrap();
    let before = rt.snapshot();
    let path = archive_dir.path().join("city.agentcity");
    rt.export_archive(&path).unwrap();
    let restored = Runtime::open(destination.path(), false).unwrap();
    let roots = std::collections::HashMap::from([(
        before.projects[0].id.clone(),
        destination.path().display().to_string(),
    )]);
    restored.import_archive(&path, &roots).unwrap();
    let after = restored.snapshot();
    assert_eq!(after.residents[0].id, before.residents[0].id);
    assert_eq!(after.tasks[0].id, before.tasks[0].id);
    assert!(after.cities[0].held);
    assert_eq!(after.tasks[0].state, TaskState::Paused);
    assert_eq!(
        after.projects[0].workspace,
        std::fs::canonicalize(destination.path())
            .unwrap()
            .display()
            .to_string()
    );
    let mut damaged: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    damaged["world"]["projects"][0]["workspace"] = "/tmp/../outside".into();
    std::fs::write(&path, serde_json::to_vec(&damaged).unwrap()).unwrap();
    assert!(archive::read(&path).is_err());
    damaged["digest"] =
        files::digest(&serde_json::to_vec(&(&damaged["world"], &damaged["commands"])).unwrap())
            .into();
    std::fs::write(&path, serde_json::to_vec(&damaged).unwrap()).unwrap();
    assert_eq!(archive::read(&path).unwrap_err().code, "invalid_archive");
    assert_eq!(restored.snapshot().revision, after.revision);
}

#[test]
fn project_deletion_requires_current_preview_and_preserves_external_files_and_command_tombstones() {
    let data = TempDir::new().unwrap();
    let workspace = TempDir::new().unwrap();
    std::fs::write(workspace.path().join("keep.txt"), "External project file").unwrap();
    let rt = Runtime::open(data.path(), false).unwrap();
    let creation = command(Action::CreateProject {
        name: "Disposable".into(),
        purpose: "Deletion fixture".into(),
        workspace: workspace.path().display().to_string(),
    });
    let created = rt.command(creation.clone()).unwrap();
    let project = created.target_id.clone().unwrap();
    let world = rt.snapshot();
    let mayor = world
        .projects
        .iter()
        .find(|p| p.id == project)
        .unwrap()
        .lead_mayor
        .clone();
    rt.command(command(Action::SaveMemory {
        resident_id: mayor.clone(),
        content: "Private fixture note".into(),
        base_revision: 0,
    }))
    .unwrap();
    assert_eq!(rt.deletion_preview(&project).unwrap()["allowed"], false);
    rt.command(command(Action::ArchiveProject {
        project_id: project.clone(),
        archived: true,
    }))
    .unwrap();
    let preview = rt.deletion_preview(&project).unwrap();
    assert_eq!(preview["counts"]["residents"], 1);
    assert_eq!(preview["counts"]["memories"], 1);
    assert!(rt
        .command(command(Action::DeleteProject {
            project_id: project.clone(),
            digest: "altered".into()
        }))
        .is_err());
    rt.command(command(Action::DeleteProject {
        project_id: project.clone(),
        digest: preview["digest"].as_str().unwrap().into(),
    }))
    .unwrap();
    assert!(!rt.snapshot().projects.iter().any(|p| p.id == project));
    assert!(!data.path().join("homes").join(mayor).exists());
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("keep.txt")).unwrap(),
        "External project file"
    );
    assert_eq!(rt.command(creation).unwrap().target_id, created.target_id);
    assert_eq!(rt.snapshot().projects.len(), 1);
    drop(rt);
    let reopened = Runtime::open(data.path(), false).unwrap();
    assert_eq!(reopened.snapshot().projects.len(), 1);
    assert!(!serde_json::to_string(&reopened.snapshot())
        .unwrap()
        .contains("Private fixture note"));
}

#[test]
fn skill_requirements_reject_missing_tools_and_explicit_selection_uses_latest_version() {
    let dir = TempDir::new().unwrap();
    let package = dir.path().join("market");
    std::fs::create_dir(&package).unwrap();
    std::fs::write(package.join("SKILL.md"),"---\nname: market-analysis\ndescription: Market analysis for launch planning\nrequires_tools: [read_file, write_file]\n---\nCompare observed evidence.").unwrap();
    let mut catalog = skills::discover(&[dir.path().display().to_string()]).unwrap();
    let original = catalog[0].id.clone();
    assert!(catalog[0].available);
    assert!(skills::assignment(&catalog, "Mayor", "Market launch", None)
        .unwrap()
        .contains(&original));
    assert_eq!(
        skills::assignment(
            &catalog,
            "Analyst",
            "",
            Some(&serde_json::json!([original]))
        )
        .unwrap()[0],
        original
    );
    std::fs::write(package.join("SKILL.md"),"---\nname: market-analysis\ndescription: Market analysis\nrequires_tools: [web_search]\n---\nRequires current sources.").unwrap();
    catalog.extend(skills::discover(&[dir.path().display().to_string()]).unwrap());
    assert!(!catalog.last().unwrap().available);
    assert!(skills::assignment(
        &catalog,
        "Analyst",
        "",
        Some(&serde_json::json!([original]))
    )
    .is_err());
    assert!(skills::assignment(&catalog, "Analyst", "Market", None)
        .unwrap()
        .is_empty());
}

#[test]
fn portable_archive_preserves_command_retries_without_replaying_effects() {
    let source = TempDir::new().unwrap();
    let target = TempDir::new().unwrap();
    let workspace = TempDir::new().unwrap();
    let archive_dir = TempDir::new().unwrap();
    let runtime = Runtime::open(source.path(), false).unwrap();
    let assignment = assign(&runtime);
    let receipt = runtime.command(assignment.clone()).unwrap();
    let path = archive_dir.path().join("city.json");
    runtime.export_archive(&path).unwrap();
    let original = runtime.snapshot();
    let restored = Runtime::open(target.path(), false).unwrap();
    restored
        .import_archive(
            &path,
            &std::collections::HashMap::from([(
                original.projects[0].id.clone(),
                workspace.path().display().to_string(),
            )]),
        )
        .unwrap();
    assert_eq!(
        restored.command(assignment.clone()).unwrap().target_id,
        receipt.target_id
    );
    assert_eq!(restored.snapshot().tasks.len(), 1);
    assert!(restored.snapshot().runs.is_empty());
    assert_eq!(restored.snapshot().tasks[0].state, TaskState::Paused);
    let mut altered = assignment;
    altered.action = Action::Configure {
        settings: Settings::default(),
    };
    assert_eq!(restored.command(altered).unwrap_err().code, "duplicate_id");
}
