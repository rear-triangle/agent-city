//! Reopen an already completed offline fixture without starting a scheduler or provider.
use city_core::*;
use std::path::PathBuf;
fn main() -> Result<()> {
    let directory = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .expect("Provide the completed offline fixture directory");
    if !directory.join("city.sqlite3").is_file() {
        return Err(CityError::new(
            "missing_fixture",
            "The completed fixture database must already exist.",
        ));
    }
    let runtime = Runtime::open(&directory, false)?;
    let world = runtime.snapshot();
    let goal = world
        .tasks
        .iter()
        .find(|task| {
            task.id == task.goal_id
                && task.state == TaskState::Accepted
                && task.output.contains("CEDAR-417")
        })
        .ok_or_else(|| {
            CityError::new(
                "missing_acceptance",
                "The offline fixture goal is not accepted.",
            )
        })?;
    if world
        .acceptance_ledger
        .iter()
        .filter(|entry| entry.goal_id == goal.id && entry.reversed_at.is_none())
        .count()
        != 1
        || world.runs.iter().any(|run| run.ended_at.is_none())
        || !world.memories.iter().any(|memory| memory.accepted)
    {
        return Err(CityError::new(
            "recovery_failed",
            "Acceptance, memory, or completed runs did not survive.",
        ));
    }
    let project = world
        .projects
        .iter()
        .find(|project| project.id == goal.project_id)
        .unwrap();
    if !std::fs::read_to_string(PathBuf::from(&project.workspace).join("recommendation.txt"))?
        .contains("CEDAR-417")
    {
        return Err(CityError::new(
            "missing_artifact",
            "The verified artifact did not survive.",
        ));
    }
    for name in [
        "delegate",
        "read_file",
        "write_file",
        "remember",
        "review_result",
    ] {
        if !world
            .runs
            .iter()
            .flat_map(|run| &run.tool_receipts)
            .any(|receipt| receipt.name == name && receipt.state == "confirmed")
        {
            return Err(CityError::new(
                "missing_receipt",
                format!("Missing confirmed {name} receipt."),
            ));
        }
    }
    println!("Recovered accepted fixture goal {} with {} completed runs, retained notes, exact tool receipts, one active acceptance, and recommendation.txt. No scheduler or provider was started.", goal.id, world.runs.len());
    Ok(())
}
