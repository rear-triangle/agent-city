//! Real hosted interruption, thread continuation, and application approval fixture.
use city_core::*;
use std::{path::PathBuf, time::Duration};
fn command(action: Action) -> Command {
    Command {
        schema_version: 1,
        id: id(),
        expected_revision: None,
        action,
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    let directory = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .expect("Provide a new disposable fixture directory");
    if directory.join("city.sqlite3").exists() {
        return Err(CityError::new(
            "existing_fixture",
            "Use a fresh fixture directory.",
        ));
    }
    let rt = Runtime::open(&directory, false)?;
    let world = rt.snapshot();
    let workspace = PathBuf::from(&world.projects[0].workspace);
    std::fs::write(workspace.join("brief.txt"),"Write this exact verification code into result.txt: CONTROL-842. This is a disposable control fixture.")?;
    let model = std::env::var("CITY_PROOF_MODEL").unwrap_or_else(|_| "codex:gpt-6.1-sol".into());
    if !rt
        .refresh_profiles()
        .await?
        .iter()
        .any(|p| p.model == model && p.available)
    {
        return Err(CityError::new(
            "unavailable",
            "The requested hosted profile is unavailable.",
        ));
    }
    let mut settings = world.settings;
    settings.preferred_model = Some(model);
    rt.command(command(Action::Configure { settings }))?;
    let goal=rt.command(command(Action::Assign{project_id:world.projects[0].id.clone(),city_id:world.cities[0].id.clone(),resident_id:None,title:"Read brief.txt using city_read_file, then write result.txt using city_write_file with the verification code. This is a small direct task. Do not delegate. Report the code after the actual file write succeeds.".into(),criteria:"The actual result.txt must contain CONTROL-842 and the final response must report that code.".into()}))?.target_id.unwrap();
    let mut updates = rt.subscribe();
    rt.start();
    let mut phase = 0;
    let mut original_thread = None;
    tokio::time::timeout(Duration::from_secs(480),async {
        loop {
            let w=rt.snapshot();let task=w.tasks.iter().find(|t|t.id==goal).unwrap();
            if w.runs.iter().any(|r|r.provider!="codex"){return Err(CityError::new("fallback_not_proof","A fallback is not evidence of hosted control."));}
            if phase==0&&w.decisions.iter().any(|d|d.state=="pending"&&d.action=="write_file") {
                original_thread=w.runs.last().unwrap().provider_thread.clone();
                if let Ok(pid_path)=std::env::var("CITY_PROOF_DISCONNECT_PID") {
                    let pid=std::fs::read_to_string(pid_path)?.trim().parse::<u32>().map_err(|_|CityError::new("invalid_pid","Invalid fixture PID."))?;
                    let owner=std::process::Command::new("/bin/ps").args(["-p",&pid.to_string(),"-o","ppid="]).output()?;
                    if String::from_utf8_lossy(&owner.stdout).trim()!=std::process::id().to_string(){return Err(CityError::new("unowned_pid","Refusing to interrupt a process not owned by this fixture."));}
                    if !std::process::Command::new("/bin/kill").args(["-KILL",&pid.to_string()]).status()?.success(){return Err(CityError::new("kill_failed","Fixture process did not stop."));}
                    phase=3;println!("Stopped this fixture's own Codex app-server while the exact write awaited approval.");
                }else{rt.command(command(Action::Pause{resident_id:Some(task.resident_id.clone()),city_id:None}))?;phase=1;println!("Pause requested while an exact file write awaited approval.");}
            }else if phase==1&&matches!(task.state,TaskState::Paused|TaskState::Review) {
                if workspace.join("result.txt").exists(){return Err(CityError::new("unexpected_effect","A file was written before approval."));}
                let run=w.runs.last().unwrap();if run.ended_at.is_none(){return Err(CityError::new("unconfirmed_pause","No terminal run acknowledgement."));}
                println!("First turn ended as {}. No file effect occurred.",run.state);
                if task.state==TaskState::Review{rt.command(command(Action::Steer{task_id:goal.clone(),direction:"Continue the original approved scope. Request the exact result.txt write again.".into()}))?;}
                rt.command(command(Action::Resume{task_id:goal.clone()}))?;phase=2;
            }else if phase==2 {
                for decision in w.decisions.iter().filter(|d|d.state=="pending") {
                    let allow=decision.action=="write_file"&&decision.arguments["path"]=="result.txt"&&decision.arguments["content"].as_str().is_some_and(|s|s.contains("CONTROL-842")&&s.len()<2000);
                    rt.command(Command{schema_version:1,id:id(),expected_revision:Some(decision.revision),action:Action::Decide{decision_id:decision.id.clone(),allow,digest:decision.digest.clone()}})?;
                }
                if task.state==TaskState::Review {
                    if !task.output.contains("CONTROL-842")||!std::fs::read_to_string(workspace.join("result.txt"))?.contains("CONTROL-842"){return Err(CityError::new("missing_result","The resumed turn did not establish the expected evidence."));}
                    if w.runs.len()!=2||original_thread.is_none()||w.runs.iter().any(|r|r.provider_thread!=original_thread){return Err(CityError::new("wrong_thread","Continuation did not use the original thread exactly once."));}
                    if w.decisions.iter().filter(|d|d.state=="allowed").count()!=1{return Err(CityError::new("wrong_effect_count","Expected exactly one approved write."));}
                    rt.command(command(Action::Accept{task_id:goal.clone()}))?;
                    println!("Passed: two correlated hosted turns on one thread, acknowledged stop without a write, one resumed approved write, correct file/result evidence, and owner acceptance.");return Ok(());
                }
            }
            if phase==3&&task.state==TaskState::Unknown {
                if workspace.join("result.txt").exists()||w.runs.len()!=1||w.decisions.iter().any(|d|d.state=="pending"){return Err(CityError::new("unsafe_disconnect","Disconnected work did not retain one uncertain attempt with invalidated approvals and no file effect."));}
                println!("Passed: disconnection produced Unknown, invalidated the approval, retained the sole run without retry, and made no file change.");return Ok(());
            }
            if matches!(task.state,TaskState::Unknown|TaskState::Blocked|TaskState::Failed){return Err(CityError::new("control_failed",format!("{:?}: {:?}",task.state,task.blocker)));}
            if phase==3 {let _=tokio::time::timeout(Duration::from_secs(15),updates.recv()).await.map_err(|_|CityError::new("disconnect_timeout","Provider disconnection was not reconciled within fifteen seconds."))?;} else {let _=updates.recv().await;}
        }
    }).await.map_err(|_|CityError::new("timeout","Hosted control fixture exceeded eight minutes."))?
}
