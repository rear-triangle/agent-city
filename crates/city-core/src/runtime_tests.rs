use super::*;
use tempfile::TempDir;
fn setup() -> (TempDir, Arc<Runtime>, String) {
    let dir = TempDir::new().unwrap();
    let rt = Runtime::open(dir.path(), true).unwrap();
    let w = rt.snapshot();
    let receipt = rt
        .command(cmd(Action::Assign {
            project_id: w.projects[0].id.clone(),
            city_id: w.cities[0].id.clone(),
            resident_id: None,
            title: "Bounded test".into(),
            criteria: "Evidence".into(),
        }))
        .unwrap();
    (dir, rt, receipt.target_id.unwrap())
}
fn cmd(action: Action) -> Command {
    Command {
        schema_version: 1,
        id: id(),
        expected_revision: None,
        action,
    }
}
#[test]
fn pause_holds_until_confirmation_and_late_completion_cannot_finish_new_run() {
    let (_dir, rt, task) = setup();
    let run = rt.reserve().unwrap().unwrap();
    rt.acknowledge(&run.id).unwrap();
    rt.command(cmd(Action::Pause {
        resident_id: Some(run.resident_id.clone()),
        city_id: None,
    }))
    .unwrap();
    assert_eq!(rt.snapshot().tasks[0].state, TaskState::Pausing);
    assert!(rt.reserve().unwrap().is_none());
    rt.finish(
        &run.id,
        Err(CityError::new("interrupted", "confirmed")),
        true,
    )
    .unwrap();
    assert_eq!(rt.snapshot().tasks[0].state, TaskState::Paused);
    rt.command(cmd(Action::Resume {
        task_id: task.clone(),
    }))
    .unwrap();
    let continuation = rt.reserve().unwrap().unwrap();
    assert_ne!(continuation.id, run.id);
    assert_eq!(continuation.resident_id, run.resident_id);
    assert!(rt.finish(&run.id, Ok("stale".into()), false).is_err());
    assert_eq!(rt.snapshot().tasks[0].state, TaskState::Starting);
}
#[test]
fn restart_holds_uncertain_execution_and_workspace_lease() {
    let (dir, rt, _) = setup();
    let run = rt.reserve().unwrap().unwrap();
    rt.acknowledge(&run.id).unwrap();
    drop(rt);
    let rt = Runtime::open(dir.path(), true).unwrap();
    assert_eq!(rt.snapshot().tasks[0].state, TaskState::Unknown);
    assert!(rt.reserve().unwrap().is_none());
}
#[test]
fn finish_racing_with_pause_preserves_result_and_queue_hold() {
    let (_dir, rt, _) = setup();
    let run = rt.reserve().unwrap().unwrap();
    rt.acknowledge(&run.id).unwrap();
    rt.command(cmd(Action::Pause {
        resident_id: Some(run.resident_id.clone()),
        city_id: None,
    }))
    .unwrap();
    rt.finish(&run.id, Ok("confirmed result".into()), false)
        .unwrap();
    let w = rt.snapshot();
    assert_eq!(w.tasks[0].state, TaskState::Review);
    assert!(w.residents[0].held);
    assert_eq!(w.tasks[0].output, "confirmed result");
}
#[tokio::test]
async fn file_write_waits_for_exact_decision_and_records_evidence() {
    let (_dir, rt, task) = setup();
    let run = rt.reserve().unwrap().unwrap();
    rt.acknowledge(&run.id).unwrap();
    let root = rt.snapshot().projects[0].workspace.clone();
    let token = CancellationToken::new();
    let r = rt.clone();
    let run_id = run.id.clone();
    let worker = tokio::spawn(async move {
        provider::execute_tool(
            r,
            &run_id,
            "write_file",
            json!({"path":"result.txt","content":"confirmed","reason":"test output"}),
            &token,
        )
        .await
    });
    tokio::time::timeout(Duration::from_secs(3), async {
        while rt.snapshot().decisions.is_empty() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert!(!Path::new(&root).join("result.txt").exists());
    let decision = rt.snapshot().decisions[0].clone();
    assert!(rt
        .command(cmd(Action::Decide {
            decision_id: decision.id.clone(),
            allow: true,
            digest: "changed".into()
        }))
        .is_err());
    rt.command(cmd(Action::Decide {
        decision_id: decision.id,
        allow: true,
        digest: decision.digest,
    }))
    .unwrap();
    worker.await.unwrap().unwrap();
    assert_eq!(
        std::fs::read_to_string(Path::new(&root).join("result.txt")).unwrap(),
        "confirmed"
    );
    assert_eq!(
        rt.snapshot()
            .tasks
            .iter()
            .find(|t| t.id == task)
            .unwrap()
            .state,
        TaskState::Running
    );
    assert_eq!(rt.snapshot().runs[0].tool_receipts[0].state, "confirmed");
}
#[tokio::test]
async fn denial_cannot_be_rephrased_into_new_permission() {
    let (_dir, rt, _) = setup();
    let run = rt.reserve().unwrap().unwrap();
    rt.acknowledge(&run.id).unwrap();
    let token = CancellationToken::new();
    let r = rt.clone();
    let rid = run.id.clone();
    let worker = tokio::spawn(async move {
        provider::execute_tool(
            r,
            &rid,
            "write_file",
            json!({"path":"result.txt","content":"denied","reason":"first reason"}),
            &CancellationToken::new(),
        )
        .await
    });
    tokio::time::timeout(Duration::from_secs(3), async {
        while rt.snapshot().decisions.is_empty() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let d = rt.snapshot().decisions[0].clone();
    rt.command(cmd(Action::Decide {
        decision_id: d.id,
        allow: false,
        digest: d.digest,
    }))
    .unwrap();
    assert!(worker.await.unwrap().is_err());
    let result = provider::execute_tool(
        rt.clone(),
        &run.id,
        "write_file",
        json!({"path":"result.txt","content":"denied","reason":"a different reason"}),
        &token,
    )
    .await;
    assert!(result.is_err());
    assert_eq!(rt.snapshot().decisions.len(), 1);
}
#[tokio::test]
async fn sender_identity_is_bound_to_run_and_duplicate_messages_coalesce() {
    let (_dir, rt, _) = setup();
    let run = rt.reserve().unwrap().unwrap();
    rt.acknowledge(&run.id).unwrap();
    let token = CancellationToken::new();
    let delegated=provider::execute_tool(rt.clone(),&run.id,"delegate",json!({"title":"Inspect evidence","role":"Reader","instructions":"Read a local brief","criteria":"Provide evidence"}),&token).await.unwrap();
    let args = json!({"sender":"forged-chief","recipient":delegated["residentId"],"purpose":"evidence","body":"Bounded message"});
    let first = provider::execute_tool(rt.clone(), &run.id, "send_message", args.clone(), &token)
        .await
        .unwrap();
    let second = provider::execute_tool(rt.clone(), &run.id, "send_message", args, &token)
        .await
        .unwrap();
    assert_eq!(first["messageId"], second["messageId"]);
    let w = rt.snapshot();
    assert_eq!(w.messages.last().unwrap().from, run.resident_id);
    let road = w.roads[0].clone();
    rt.command(cmd(Action::SetRoad {
        road_id: road.id,
        enabled: false,
    }))
    .unwrap();
    let blocked = provider::execute_tool(
        rt.clone(),
        &run.id,
        "send_message",
        json!({"recipient":delegated["residentId"],"purpose":"evidence","body":"A new message"}),
        &token,
    )
    .await
    .unwrap();
    assert_eq!(blocked["state"], "blocked");
    let blocked_id = blocked["messageId"].as_str().unwrap().to_string();
    let current = rt.snapshot();
    let message = current
        .messages
        .iter()
        .find(|m| m.id == blocked_id)
        .unwrap();
    assert_eq!(message.digest, files::digest(b"A new message"));
    assert_eq!(message.causal_depth, 1);
    assert_eq!(message.route_attempts.len(), 1);
    assert!(message.expires_at.is_some());
    assert!(provider::execute_tool(rt.clone(),&run.id,"send_message",json!({"recipient":delegated["residentId"],"purpose":"reply","body":"Invalid reply","reply_to":first["messageId"]}),&token).await.is_err());
    rt.transact(|world| {
        world
            .messages
            .iter_mut()
            .find(|m| m.id == blocked_id)
            .unwrap()
            .expires_at = Some("2020-01-01T00:00:00Z".into());
        Ok(())
    })
    .unwrap();
    rt.command(cmd(Action::SetRoad {
        road_id: current.roads[0].id.clone(),
        enabled: true,
    }))
    .unwrap();
    assert_eq!(
        rt.snapshot()
            .messages
            .iter()
            .find(|m| m.id == blocked_id)
            .unwrap()
            .state,
        "expired"
    );
}
use std::time::Duration;

#[tokio::test]
async fn nested_city_work_returns_to_one_lead_mayor_for_owner_review() {
    let (_dir, rt, goal) = setup();
    let token = CancellationToken::new();
    let lead = rt.reserve().unwrap().unwrap();
    rt.acknowledge(&lead.id).unwrap();
    let founded = provider::execute_tool(
        rt.clone(),
        &lead.id,
        "found_city",
        json!({"name":"Development","purpose":"Build the implementation workstream"}),
        &token,
    )
    .await
    .unwrap();
    provider::execute_tool(rt.clone(),&lead.id,"delegate",json!({"title":"Build workstream","role":"Builder","instructions":"Coordinate implementation","criteria":"Reviewed code","city_id":founded["cityId"]}),&token).await.unwrap();
    rt.finish(&lead.id, Ok("Workstream assigned".into()), false)
        .unwrap();
    let peer = rt.reserve().unwrap().unwrap();
    assert_ne!(peer.resident_id, lead.resident_id);
    provider::execute_tool(rt.clone(),&peer.id,"delegate",json!({"title":"Implement component","role":"Builder","instructions":"Bounded component","criteria":"Working evidence"}),&token).await.unwrap();
    rt.finish(&peer.id, Ok("Implementation assigned".into()), false)
        .unwrap();
    let worker = rt.reserve().unwrap().unwrap();
    rt.finish(&worker.id, Ok("Component and evidence".into()), false)
        .unwrap();
    let review = rt.reserve().unwrap().unwrap();
    assert_eq!(review.resident_id, peer.resident_id);
    provider::execute_tool(
        rt.clone(),
        &review.id,
        "review_result",
        json!({"accept":true,"evidence":"Inspected component"}),
        &token,
    )
    .await
    .unwrap();
    rt.finish(&review.id, Ok("Reviewed".into()), false).unwrap();
    let integration = rt.reserve().unwrap().unwrap();
    assert_eq!(integration.task_id, peer.task_id);
    rt.finish(&integration.id, Ok("Integrated workstream".into()), false)
        .unwrap();
    let review = rt.reserve().unwrap().unwrap();
    assert_eq!(review.resident_id, lead.resident_id);
    provider::execute_tool(
        rt.clone(),
        &review.id,
        "review_result",
        json!({"accept":true,"evidence":"Inspected integrated work"}),
        &token,
    )
    .await
    .unwrap();
    rt.finish(&review.id, Ok("Reviewed".into()), false).unwrap();
    let final_run = rt.reserve().unwrap().unwrap();
    assert_eq!(final_run.task_id, goal);
    rt.finish(&final_run.id, Ok("Final outcome".into()), false)
        .unwrap();
    let w = rt.snapshot();
    assert_eq!(
        w.tasks.iter().find(|t| t.id == goal).unwrap().state,
        TaskState::Review
    );
    assert_eq!(w.cities.len(), 2);
    assert_eq!(w.residents.len(), 3);
    assert_eq!(w.messages.len(), 4);
    assert!(w.messages.iter().all(|m| m.state == "delivered"));
}
#[tokio::test]
async fn reviewer_cannot_accept_with_prose_without_a_review_receipt() {
    let (_dir, rt, _) = setup();
    let run = rt.reserve().unwrap().unwrap();
    let token = CancellationToken::new();
    provider::execute_tool(
        rt.clone(),
        &run.id,
        "delegate",
        json!({"title":"Analysis","role":"Analyst","instructions":"Analyze","criteria":"Evidence"}),
        &token,
    )
    .await
    .unwrap();
    rt.finish(&run.id, Ok("Assigned".into()), false).unwrap();
    let worker = rt.reserve().unwrap().unwrap();
    rt.finish(&worker.id, Ok("A claim".into()), false).unwrap();
    let review = rt.reserve().unwrap().unwrap();
    rt.finish(&review.id, Ok("I accepted it".into()), false)
        .unwrap();
    let w = rt.snapshot();
    assert_eq!(
        w.tasks
            .iter()
            .find(|t| t.id == review.task_id)
            .unwrap()
            .state,
        TaskState::Blocked
    );
    assert_eq!(
        w.tasks
            .iter()
            .find(|t| t.id == worker.task_id)
            .unwrap()
            .state,
        TaskState::Review
    );
    assert!(rt.reserve().unwrap().is_none());
}
#[tokio::test]
async fn skill_update_preserves_active_run_and_archived_supporting_file() {
    let (dir, rt, _) = setup();
    let root = dir.path().join("catalog/planning");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("SKILL.md"),
        "---\nname: planning\ndescription: planning and delegation\n---\nRead guide.txt",
    )
    .unwrap();
    std::fs::write(root.join("guide.txt"), "Original guidance").unwrap();
    let mut settings = rt.snapshot().settings;
    settings.skill_roots = vec![dir.path().join("catalog").display().to_string()];
    rt.command(cmd(Action::Configure { settings })).unwrap();
    rt.scan_skills().unwrap();
    let run = rt.reserve().unwrap().unwrap();
    let old = run.skill_ids[0].clone();
    std::fs::write(root.join("guide.txt"), "Changed guidance").unwrap();
    rt.scan_skills().unwrap();
    assert_ne!(rt.snapshot().residents[0].skills[0], old);
    let result = provider::execute_tool(
        rt.clone(),
        &run.id,
        "read_skill",
        json!({"skill_id":old,"path":"guide.txt"}),
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(result["content"], "Original guidance");
    assert_eq!(rt.snapshot().skills.len(), 2);
}
#[test]
fn local_only_excludes_hosted_even_when_preferred_and_capacity_is_enforced() {
    let mut world = World::empty(false);
    world.settings.policy = "local_only".into();
    let local = Profile {
        id: "small".into(),
        model: "small".into(),
        provider: "ollama".into(),
        available: true,
        tools: true,
        size: 1024,
        context_length: 16384,
        loaded: false,
        detail: String::new(),
    };
    let hosted = Profile {
        id: "codex:test".into(),
        model: "codex:test".into(),
        provider: "codex".into(),
        size: 0,
        ..local.clone()
    };
    let profiles = vec![local, hosted];
    let preferred = "codex:test".to_string();
    assert_eq!(
        resources::choose(&profiles, &world, Some(&preferred), u64::MAX)
            .unwrap()
            .provider,
        "ollama"
    );
    assert!(resources::choose(&profiles, &world, Some(&preferred), 0).is_none());
    world.settings.policy = "auto".into();
    assert_eq!(
        resources::choose(&profiles, &world, Some(&preferred), 0)
            .unwrap()
            .provider,
        "codex"
    );
}

#[test]
fn overlapping_project_folders_share_the_same_execution_lease() {
    let (_dir, rt, _) = setup();
    let w = rt.snapshot();
    let second = rt
        .command(cmd(Action::CreateProject {
            name: "Shared folder".into(),
            purpose: "Separate goal in the same checkout".into(),
            workspace: w.projects[0].workspace.clone(),
        }))
        .unwrap()
        .target_id
        .unwrap();
    let city = rt
        .snapshot()
        .cities
        .iter()
        .find(|c| c.project_id == second)
        .unwrap()
        .id
        .clone();
    rt.command(cmd(Action::Assign {
        project_id: second,
        city_id: city,
        resident_id: None,
        title: "Second writer".into(),
        criteria: "Evidence".into(),
    }))
    .unwrap();
    let run = rt.reserve().unwrap().unwrap();
    assert!(rt.reserve().unwrap().is_none());
    rt.finish(&run.id, Ok("First writer finished".into()), false)
        .unwrap();
    assert!(rt.reserve().unwrap().is_some());
}
#[test]
fn acceptance_history_keeps_reversals_and_never_duplicates_rewards() {
    let (dir, rt, goal) = setup();
    let run = rt.reserve().unwrap().unwrap();
    rt.finish(&run.id, Ok("First version".into()), false)
        .unwrap();
    rt.command(cmd(Action::Accept {
        task_id: goal.clone(),
    }))
    .unwrap();
    rt.command(cmd(Action::Accept {
        task_id: goal.clone(),
    }))
    .unwrap();
    assert_eq!(rt.snapshot().acceptance_ledger.len(), 1);
    rt.command(cmd(Action::Reopen {
        task_id: goal.clone(),
        direction: "Change the recommendation".into(),
    }))
    .unwrap();
    assert!(rt.snapshot().acceptance_ledger[0].reversed_at.is_some());
    let run = rt.reserve().unwrap().unwrap();
    rt.finish(&run.id, Ok("Second version".into()), false)
        .unwrap();
    rt.command(cmd(Action::Accept { task_id: goal })).unwrap();
    let ledger = rt.snapshot().acceptance_ledger;
    assert_eq!(ledger.len(), 2);
    assert_eq!(ledger.iter().filter(|e| e.reversed_at.is_none()).count(), 1);
    drop(rt);
    let restored = Runtime::open(dir.path(), true).unwrap().snapshot();
    assert_eq!(
        serde_json::to_value(&restored.acceptance_ledger).unwrap(),
        serde_json::to_value(&ledger).unwrap()
    );
}
#[tokio::test]
async fn merger_preserves_lineage_and_requires_review_of_inherited_memory() {
    let (_dir, rt, _) = setup();
    let run = rt.reserve().unwrap().unwrap();
    let mut ids = vec![];
    for role in ["Writer", "Editor"] {
        let child=provider::execute_tool(rt.clone(),&run.id,"delegate",json!({"title":format!("{role} output"),"role":role,"instructions":"Bounded fixture","criteria":"Evidence"}),&CancellationToken::new()).await.unwrap();
        ids.push(child["residentId"].as_str().unwrap().to_string());
    }
    rt.transact(|w| {
        for t in &mut w.tasks {
            if ids.contains(&t.resident_id) {
                t.state = TaskState::Accepted;
            }
        }
        Ok(())
    })
    .unwrap();
    for id in &ids {
        rt.command(cmd(Action::SaveMemory {
            resident_id: id.clone(),
            content: format!("Instruction from {id}"),
            base_revision: 0,
        }))
        .unwrap();
    }
    let successor = rt
        .command(cmd(Action::Merge {
            resident_ids: ids.clone(),
            name: "Curator".into(),
            role: "Editor".into(),
        }))
        .unwrap()
        .target_id
        .unwrap();
    let w = rt.snapshot();
    assert!(ids.iter().all(|id| w.residents.iter().any(|r| r.id == *id
        && r.lifecycle == "merged"
        && r.successor.as_ref() == Some(&successor))));
    assert_eq!(
        w.memories
            .iter()
            .filter(|m| m.resident_id == successor && !m.accepted)
            .count(),
        2
    );
    assert!(w.residents.iter().find(|r| r.id == successor).unwrap().held);
}

#[test]
fn loaded_profiles_reserve_context_without_charging_the_weights_twice() {
    let world = World::empty(false);
    let profile = Profile {
        id: "loaded".into(),
        model: "loaded".into(),
        provider: "ollama".into(),
        available: true,
        tools: true,
        size: 20_000_000_000,
        context_length: 16384,
        loaded: true,
        detail: String::new(),
    };
    assert!(resources::choose(
        std::slice::from_ref(&profile),
        &world,
        None,
        2 * 1024 * 1024 * 1024
    )
    .is_some());
    assert!(resources::choose(
        &[Profile {
            loaded: false,
            ..profile
        }],
        &world,
        None,
        2 * 1024 * 1024 * 1024
    )
    .is_none());
}

#[test]
fn input_revision_change_invalidates_transitive_outputs_and_keeps_run_provenance() {
    let (_dir, rt, original) = setup();
    let run = rt.reserve().unwrap().unwrap();
    rt.finish(&run.id, Ok("Input v1".into()), false).unwrap();
    rt.command(cmd(Action::Accept {
        task_id: original.clone(),
    }))
    .unwrap();
    let input_revision = rt.snapshot().tasks[0].revision;
    let middle = rt
        .transact(|world| {
            let mut task = world.tasks[0].clone();
            task.id = id();
            task.goal_id = task.id.clone();
            task.run_id = None;
            task.state = TaskState::Queued;
            task.accepted_at = None;
            task.dependencies = vec![original.clone()];
            let id = task.id.clone();
            world.tasks.push(task);
            Ok(id)
        })
        .unwrap();
    let middle_run = rt.reserve().unwrap().unwrap();
    assert_eq!(
        middle_run.input_revisions.get(&original),
        Some(&input_revision)
    );
    rt.finish(&middle_run.id, Ok("Derived v1".into()), false)
        .unwrap();
    rt.command(cmd(Action::Accept {
        task_id: middle.clone(),
    }))
    .unwrap();
    let last = rt
        .transact(|world| {
            let mut task = world.tasks[1].clone();
            task.id = id();
            task.goal_id = task.id.clone();
            task.dependencies = vec![middle.clone()];
            let id = task.id.clone();
            world.tasks.push(task);
            Ok(id)
        })
        .unwrap();
    rt.command(cmd(Action::Reopen {
        task_id: original.clone(),
        direction: "Use v2".into(),
    }))
    .unwrap();
    let world = rt.snapshot();
    assert!(world
        .tasks
        .iter()
        .filter(|task| task.id == middle || task.id == last)
        .all(|task| task.stale && task.accepted_at.is_none()));
    assert!(world
        .acceptance_ledger
        .iter()
        .filter(|entry| entry.goal_id == middle)
        .all(|entry| entry.reversed_at.is_some()));
    assert_eq!(
        world
            .runs
            .iter()
            .find(|run| run.id == middle_run.id)
            .unwrap()
            .input_revisions
            .get(&original),
        Some(&input_revision)
    );
    assert!(rt.command(cmd(Action::Accept { task_id: last })).is_err());
}

#[test]
fn detail_retention_keeps_unresolved_context_and_scrubs_finished_session_projection() {
    let (dir, rt, task) = setup();
    let run = rt.reserve().unwrap().unwrap();
    rt.transact(|world| {
        let run = world.runs.iter_mut().find(|r| r.id == run.id).unwrap();
        run.context = "PRIVATE-RETAIN-UNTIL-FINISHED".repeat(80000);
        Ok(())
    })
    .unwrap();
    let mut settings = rt.snapshot().settings;
    settings.history_mib = 1;
    rt.command(cmd(Action::Configure { settings })).unwrap();
    assert!(rt.snapshot().runs[0].payloads_retained);
    rt.finish(&run.id, Ok("Useful final summary".into()), false)
        .unwrap();
    rt.command(cmd(Action::Accept { task_id: task })).unwrap();
    let world = rt.snapshot();
    assert!(!world.runs[0].payloads_retained);
    assert!(world.runs[0].context.is_empty());
    assert_eq!(world.tasks[0].output, "Useful final summary");
    assert_eq!(world.acceptance_ledger.len(), 1);
    let session = dir
        .path()
        .join("homes")
        .join(&run.resident_id)
        .join("sessions")
        .join(format!("{}.json", run.id));
    assert!(!std::fs::read_to_string(session)
        .unwrap()
        .contains("PRIVATE-RETAIN"));
}

#[tokio::test]
async fn owner_redaction_removes_goal_content_and_generated_sessions_but_retains_acceptance() {
    let (dir, rt, goal) = setup();
    let run = rt.reserve().unwrap().unwrap();
    let secret = "FIXTURE-PRIVATE-CONTENT-9E81";
    rt.transact(|world| {
        world.runs[0].context = secret.into();
        Ok(())
    })
    .unwrap();
    provider::execute_tool(
        rt.clone(),
        &run.id,
        "remember",
        json!({"content":secret,"source":"observed fixture"}),
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(rt.redaction_preview(&goal).unwrap()["allowed"], false);
    rt.finish(&run.id, Ok(secret.into()), false).unwrap();
    rt.command(cmd(Action::Accept {
        task_id: goal.clone(),
    }))
    .unwrap();
    let session = dir
        .path()
        .join("homes")
        .join(&run.resident_id)
        .join("sessions")
        .join(format!("{}.json", run.id));
    assert!(std::fs::read_to_string(&session).unwrap().contains(secret));
    let preview = rt.redaction_preview(&goal).unwrap();
    assert!(rt
        .command(cmd(Action::RedactGoal {
            goal_id: goal.clone(),
            digest: "changed".into()
        }))
        .is_err());
    rt.command(cmd(Action::RedactGoal {
        goal_id: goal.clone(),
        digest: preview["digest"].as_str().unwrap().into(),
    }))
    .unwrap();
    assert!(!serde_json::to_string(&rt.snapshot())
        .unwrap()
        .contains(secret));
    assert!(!std::fs::read_to_string(session).unwrap().contains(secret));
    assert_eq!(rt.snapshot().acceptance_ledger.len(), 1);
    assert_eq!(rt.snapshot().tasks[0].state, TaskState::Accepted);
    assert!(rt
        .command(cmd(Action::Reopen {
            task_id: goal,
            direction: "Repeat".into()
        }))
        .is_err());
    drop(rt);
    let reopened = Runtime::open(dir.path(), true).unwrap();
    assert!(!serde_json::to_string(&reopened.snapshot())
        .unwrap()
        .contains(secret));
}

#[test]
fn terminal_provider_failure_can_continue_but_uncertain_effects_keep_the_lease() {
    for (code, expected) in [
        ("provider_error", TaskState::Queued),
        ("outcome_unknown", TaskState::Unknown),
    ] {
        let (_dir, rt, goal) = setup();
        let run = rt.reserve().unwrap().unwrap();
        rt.profiles.lock().unwrap().push(Profile {
            id: "local".into(),
            provider: "ollama".into(),
            model: "local".into(),
            available: true,
            tools: true,
            size: 1,
            context_length: 16384,
            loaded: true,
            detail: String::new(),
        });
        rt.transact(|world| {
            let prior = world.runs.iter_mut().find(|r| r.id == run.id).unwrap();
            prior.provider = "codex".into();
            prior.provider_thread = Some("opaque-hosted-thread".into());
            prior.tool_receipts.push(ToolReceipt {
                id: id(),
                name: "write_file".into(),
                arguments: json!({"path":"prior.txt"}),
                state: "confirmed".into(),
                result: json!({"digest":"known-effect-digest"}),
            });
            Ok(())
        })
        .unwrap();
        rt.finish(
            &run.id,
            Err(CityError::new(code, "Injected provider outcome")),
            false,
        )
        .unwrap();
        assert_eq!(rt.snapshot().tasks[0].state, expected);
        if expected == TaskState::Queued {
            let continuation = rt.reserve().unwrap().unwrap();
            assert_eq!(continuation.task_id, goal);
            assert_ne!(continuation.id, run.id);
            assert!(continuation.provider_thread.is_none());
            assert!(continuation.context.contains("prior.txt"));
            assert!(continuation.context.contains("known-effect-digest"));
        } else {
            assert!(rt.reserve().unwrap().is_none());
            assert!(rt.snapshot().runs[0].ended_at.is_none());
        }
    }
}
