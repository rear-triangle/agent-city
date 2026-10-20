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
    let workspace = PathBuf::from(&w.projects[0].workspace);
    std::fs::write(workspace.join("brief.txt"),"Project Cedar. Option A costs 20 units and takes 2 days. Option B costs 35 units and takes 1 day. The owner values cost more than speed. Verification code: CEDAR-417.")?;
    let profiles = rt.refresh_profiles().await?;
    println!(
        "Installed completion/tool profiles: {}",
        profiles.iter().filter(|p| p.tools).count()
    );
    let mut settings = w.settings.clone();
    settings.preferred_model = std::env::var("CITY_PROOF_MODEL").ok();
    rt.command(Command {
        schema_version: 1,
        id: id(),
        expected_revision: None,
        action: Action::Configure { settings },
    })?;
    let receipt=rt.command(Command{schema_version:1,id:id(),expected_revision:None,action:Action::Assign{project_id:w.projects[0].id.clone(),city_id:w.cities[0].id.clone(),resident_id:None,title:"Read brief.txt using read_file. Recommend the better option for its stated priority, quote the verification code, and explain the tradeoff. This is small enough to do directly.".into(),criteria:"Read the actual local file. Include the verification code, correct costs, recommendation, and tradeoff.".into()}})?;
    let task_id = receipt.target_id.unwrap();
    rt.start();
    let mut updates = rt.subscribe();
    tokio::time::timeout(Duration::from_secs(240), async {
        loop {
            let w = rt.snapshot();
            let t = w.tasks.iter().find(|t| t.id == task_id).unwrap();
            if matches!(
                t.state,
                TaskState::Review | TaskState::Blocked | TaskState::Failed
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
                return t.state == TaskState::Review
                    && t.output.contains("CEDAR-417")
                    && tools.iter().any(|(name, state)| {
                        name.as_str() == "read_file" && state.as_str() == "confirmed"
                    });
            }
            let _ = updates.recv().await;
        }
    })
    .await
    .map_err(|_| CityError::new("timeout", "Local proof exceeded four minutes."))?
    .then_some(())
    .ok_or_else(|| {
        CityError::new(
            "proof_failed",
            "The actual file evidence was not established.",
        )
    })
}
