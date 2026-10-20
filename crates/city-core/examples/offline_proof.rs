use city_core::*;
use std::{path::PathBuf, time::Duration};
#[tokio::main]
async fn main() -> Result<()> {
    let directory = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .expect("Provide a disposable data directory");
    let rt = Runtime::open(&directory, false)?;
    let w = rt.snapshot();
    if std::env::var("CITY_PROOF_RECONCILE").as_deref() == Ok("1") {
        for task in w.tasks.iter().filter(|t| t.state == TaskState::Unknown) {
            let run = w
                .runs
                .iter()
                .find(|r| Some(&r.id) == task.run_id.as_ref())
                .unwrap();
            if run.provider != "ollama"
                || run.tool_receipts.iter().any(|r| {
                    r.state == "intent" || !["read_file", "list_files"].contains(&r.name.as_str())
                })
            {
                return Err(CityError::new(
                    "proof_requires_inspection",
                    "Only confirmed read-only fixture runs can be reconciled by this test helper.",
                ));
            }
            rt.command(Command {
                schema_version: 1,
                id: id(),
                expected_revision: None,
                action: Action::Reconcile {
                    task_id: task.id.clone(),
                    confirmed_stopped: true,
                },
            })?;
            rt.command(Command {
                schema_version: 1,
                id: id(),
                expected_revision: None,
                action: Action::Resume {
                    task_id: task.id.clone(),
                },
            })?;
        }
    }
    let workspace = PathBuf::from(&w.projects[0].workspace);
    std::fs::write(workspace.join("brief.txt"),"Project Cedar. Option A costs 20 units and takes 2 days. Option B costs 35 units and takes 1 day. The owner values cost more than speed. Verification code: CEDAR-417.")?;
    let mut offline = w.settings.clone();
    offline.policy = "local_only".into();
    rt.command(Command {
        schema_version: 1,
        id: id(),
        expected_revision: None,
        action: Action::Configure {
            settings: offline.clone(),
        },
    })?;
    let profiles = rt.refresh_profiles().await?;
    println!(
        "Installed completion/tool profiles: {}",
        profiles.iter().filter(|p| p.tools).count()
    );
    let mut settings = offline;
    settings.preferred_model = std::env::var("CITY_PROOF_MODEL").ok();
    rt.command(Command {
        schema_version: 1,
        id: id(),
        expected_revision: None,
        action: Action::Configure { settings },
    })?;
    let receipt = if let Some(task) = w.tasks.iter().find(|t| t.id == t.goal_id) {
        Receipt {
            command_id: id(),
            revision: w.revision,
            target_id: Some(task.id.clone()),
            status: "existing".into(),
        }
    } else {
        rt.command(Command{schema_version:1,id:id(),expected_revision:None,action:Action::Assign{project_id:w.projects[0].id.clone(),city_id:w.cities[0].id.clone(),resident_id:None,title:"Delegate one task to an Analyst: read brief.txt using read_file, recommend the option for the stated priority with costs, tradeoff and verification code, write recommendation.txt using write_file, and remember a useful note using remember. End your planning turn after delegating. When reviewed results return, give the Chief the recommendation including the verification code.".into(),criteria:"Read the actual local file. Include the verification code, correct costs, recommendation, and tradeoff.".into()}})?
    };
    let task_id = receipt.target_id.unwrap();
    rt.start();
    let mut updates = rt.subscribe();
    let verified = tokio::time::timeout(Duration::from_secs(600), async {
        loop {
            let w = rt.snapshot();
            for d in w.decisions.iter().filter(|d| d.state == "pending") {
                let allow = d.action == "write_file"
                    && d.arguments["path"] == "recommendation.txt"
                    && d.arguments["content"]
                        .as_str()
                        .is_some_and(|s| s.contains("CEDAR-417") && s.len() < 8000);
                rt.command(Command {
                    schema_version: 1,
                    id: id(),
                    expected_revision: Some(d.revision),
                    action: Action::Decide {
                        decision_id: d.id.clone(),
                        allow,
                        digest: d.digest.clone(),
                    },
                })?;
            }
            let t = w.tasks.iter().find(|t| t.id == task_id).unwrap();
            if matches!(
                t.state,
                TaskState::Review | TaskState::Accepted | TaskState::Blocked | TaskState::Failed
            ) {
                println!(
                    "Task state: {:?}\nOutput: {}\nBlocker: {:?}",
                    t.state, t.output, t.blocker
                );
                let tools: Vec<_> = w
                    .runs
                    .iter()
                    .flat_map(|r| &r.tool_receipts)
                    .map(|r| (&r.name, &r.state))
                    .collect();
                println!("Observed tools: {:?}", tools);
                return Ok::<bool, CityError>(
                    matches!(t.state, TaskState::Review | TaskState::Accepted)
                        && w.residents.len() >= 2
                        && std::fs::read_to_string(workspace.join("recommendation.txt"))
                            .is_ok_and(|s| s.contains("CEDAR-417"))
                        && w.memories.iter().any(|m| m.accepted)
                        && t.output.contains("CEDAR-417")
                        && tools.iter().any(|(name, state)| {
                            name.as_str() == "read_file" && state.as_str() == "confirmed"
                        }),
                );
            }
            let _ = updates.recv().await;
        }
    })
    .await
    .map_err(|_| CityError::new("timeout", "Offline proof exceeded ten minutes."))??;
    if !verified {
        return Err(CityError::new(
            "proof_failed",
            "The actual file evidence was not established.",
        ));
    }
    rt.command(Command {
        schema_version: 1,
        id: id(),
        expected_revision: None,
        action: Action::Accept {
            task_id: task_id.clone(),
        },
    })?;
    println!("Owner acceptance recorded for the verified fixture goal {task_id}.");
    Ok(())
}
