//! Codex 0.163 app-server adapter. The dynamic-tool protocol is experimental.
use crate::{runtime::run_task, *};
use serde_json::{json, Value};
use std::{collections::VecDeque, process::Stdio, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{Child, ChildStdin, ChildStdout},
};
use tokio_util::sync::CancellationToken;

fn started_turn_id(response: &Value) -> Result<String> {
    response["turn"]["id"]
        .as_str()
        .filter(|id| !id.is_empty())
        .map(String::from)
        .ok_or_else(|| CityError::new("outcome_unknown", "Codex may have started a turn but did not supply its identity. Reconcile the thread before continuing."))
}
fn event_for_turn(message: &Value, thread_id: &str, turn_id: &str) -> Result<bool> {
    let p = &message["params"];
    if p.get("threadId").is_some_and(|v| v != thread_id)
        || p.get("turnId").is_some_and(|v| v != turn_id)
    {
        return Ok(false);
    }
    if message["method"] == "turn/completed" {
        let id = p["turn"]["id"].as_str().filter(|id| !id.is_empty()).ok_or_else(|| CityError::new("outcome_unknown", "Codex reported a terminal event without a turn identity. Reconcile the thread before continuing."))?;
        return Ok(id == turn_id);
    }
    Ok(true)
}

struct Rpc {
    child: Child,
    input: ChildStdin,
    output: Lines<BufReader<ChildStdout>>,
    pending: VecDeque<Value>,
    sequence: u64,
}
impl Rpc {
    async fn open() -> Result<Self> {
        let executable = std::env::var_os("AGENT_CITY_CODEX")
            .map(std::path::PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME")
                    .map(|h| std::path::PathBuf::from(h).join(".local/bin/codex"))
                    .filter(|p| p.is_file())
            })
            .unwrap_or_else(|| "codex".into());
        let mut overrides: Vec<String> = [
            "features.multi_agent=false",
            "features.multi_agent_v2=false",
            "features.shell_tool=false",
            "features.unified_exec=false",
            "features.apps=false",
            "features.plugins=false",
            "features.skill_mcp_dependency_install=false",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        // An empty mcp_servers table merges with existing config; disable each configured entry explicitly.
        for attempt in 0..2 {
            let mut command = tokio::process::Command::new(&executable);
            for value in &overrides {
                command.arg("-c").arg(value);
            }
            let mut child = command
                .args(["app-server", "--stdio"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .kill_on_drop(true)
                .spawn()
                .map_err(|_| {
                    CityError::new(
                        "provider_unavailable",
                        "The installed Codex app-server could not start.",
                    )
                })?;
            let input = child.stdin.take().unwrap();
            let output = BufReader::new(child.stdout.take().unwrap()).lines();
            let mut rpc = Self {
                child,
                input,
                output,
                pending: VecDeque::new(),
                sequence: 0,
            };
            rpc.request("initialize",json!({"clientInfo":{"name":"agent_city","version":"0.2.0"},"capabilities":{"experimentalApi":true}})).await?;
            rpc.send(json!({"method":"initialized","params":{}}))
                .await?;
            let config = rpc
                .request("config/read", json!({"includeLayers":false}))
                .await?;
            let enabled: Vec<String> = config["config"]["mcp_servers"]
                .as_object()
                .into_iter()
                .flatten()
                .filter(|(_, entry)| entry["enabled"] != false)
                .map(|(name, _)| name.clone())
                .collect();
            if config["config"]["features"]["apps"] != false
                || config["config"]["features"]["plugins"] != false
            {
                return Err(CityError::new(
                    "provider_incompatible",
                    "External app and plugin tools could not be disabled for this profile.",
                ));
            }
            if enabled.is_empty() {
                return Ok(rpc);
            }
            rpc.stop().await;
            if attempt == 1 {
                return Err(CityError::new(
                    "provider_incompatible",
                    "External MCP tools could not be disabled for the application-only profile.",
                ));
            }
            for name in enabled {
                if name.is_empty()
                    || !name
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
                {
                    return Err(CityError::new("provider_incompatible","An external MCP configuration key cannot be isolated with this CLI version."));
                }
                overrides.push(format!("mcp_servers.{name}.enabled=false"));
            }
        }
        Err(CityError::new(
            "provider_incompatible",
            "Could not prepare the restricted Codex profile.",
        ))
    }
    async fn send(&mut self, value: Value) -> Result<()> {
        self.input
            .write_all(format!("{}\n", serde_json::to_string(&value)?).as_bytes())
            .await
            .map_err(|_| {
                CityError::new(
                    "outcome_unknown",
                    "Could not confirm delivery to the Codex process.",
                )
            })?;
        self.input.flush().await.map_err(|_| {
            CityError::new(
                "outcome_unknown",
                "Could not confirm delivery to the Codex process.",
            )
        })?;
        Ok(())
    }
    async fn read(&mut self) -> Result<Value> {
        let line = tokio::time::timeout(Duration::from_secs(180), self.output.next_line())
            .await
            .map_err(|_| {
                CityError::new(
                    "outcome_unknown",
                    "Codex did not confirm its execution outcome before the transport timeout.",
                )
            })?
            .map_err(|_| {
                CityError::new(
                    "outcome_unknown",
                    "The Codex event stream disconnected before its outcome was confirmed.",
                )
            })?
            .ok_or_else(|| {
                CityError::new(
                    "outcome_unknown",
                    "The Codex process disconnected. Reconcile its run before retrying.",
                )
            })?;
        if line.len() > 1024 * 1024 {
            return Err(CityError::new(
                "outcome_unknown",
                "Codex emitted an oversized event; reconcile the active turn.",
            ));
        }
        serde_json::from_str(&line).map_err(|_| {
            CityError::new(
                "outcome_unknown",
                "Codex emitted malformed protocol data; reconcile the active turn.",
            )
        })
    }
    async fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        self.sequence += 1;
        let id = self.sequence;
        self.send(json!({"id":id,"method":method,"params":params}))
            .await?;
        loop {
            let message = self.read().await?;
            if message["id"] == id && message.get("method").is_none() {
                if let Some(e) = message.get("error") {
                    return Err(CityError::new(
                        "provider_incompatible",
                        format!(
                            "{method}: {}",
                            e["message"].as_str().unwrap_or("Protocol request failed")
                        ),
                    ));
                }
                return Ok(message["result"].clone());
            }
            if self.pending.len() >= 128 {
                return Err(CityError::new("outcome_unknown","The provider event queue exceeded its limit. Reconcile the thread before continuing."));
            }
            self.pending.push_back(message);
        }
    }
    async fn next(&mut self) -> Result<Value> {
        if let Some(v) = self.pending.pop_front() {
            Ok(v)
        } else {
            self.read().await
        }
    }
    async fn stop(&mut self) {
        let _ = self.child.kill().await;
        let _ = self.child.wait().await;
    }
}
pub async fn profiles() -> Result<Vec<Profile>> {
    let mut rpc = Rpc::open().await?;
    let account = rpc
        .request("account/read", json!({"refreshToken":false}))
        .await?;
    let authenticated = !account["account"].is_null();
    let models = rpc.request("model/list", json!({"limit":50})).await?;
    let result=models["data"].as_array().unwrap_or(&vec![]).iter().filter_map(|m|m["model"].as_str()).map(|model|Profile{id:format!("codex:{model}"),model:format!("codex:{model}"),provider:"codex".into(),available:authenticated,tools:true,size:0,context_length:0,loaded:false,detail:"Installed Codex app-server; experimental dynamic tools. Hosted selection must be explicit. Native shell and independent subagents are disabled. External MCP servers, apps, and plugins are disabled in this child process. The provider sandbox is read-only with no native escalation; application file writes use Agent City decisions.".into()}).collect();
    rpc.stop().await;
    Ok(result)
}
pub async fn run(rt: Arc<Runtime>, run: &Run, cancel: CancellationToken) -> Result<String> {
    let w = rt.snapshot();
    let index = run_task(&w, &run.id)?;
    let task = &w.tasks[index];
    let project = w.projects.iter().find(|p| p.id == task.project_id).unwrap();
    let mut rpc = Rpc::open().await?;
    let dynamic:Vec<Value>=provider::tools_for_run(&w,run).into_iter().map(|v|json!({"type":"function","name":format!("city_{}",v["function"]["name"].as_str().unwrap()),"description":v["function"]["description"],"inputSchema":v["function"]["parameters"]})).collect();
    let model = run.model.strip_prefix("codex:").unwrap_or(&run.model);
    let thread = if let Some(thread) = &run.provider_thread {
        rpc.request("thread/resume",json!({"threadId":thread,"cwd":project.workspace,"model":model,"approvalPolicy":"never","sandbox":"read-only"})).await?
    } else {
        rpc.request("thread/start",json!({"model":model,"cwd":project.workspace,"runtimeWorkspaceRoots":[project.workspace],"approvalPolicy":"never","approvalsReviewer":"user","sandbox":"read-only","developerInstructions":format!("{}\nUse city_ application tools for coordination, files, and memory. Do not spawn independent subagents or background processes. All delegation goes through city_delegate. The app owns dispatch. Do not use commands to bypass denied application tools.",run.context),"dynamicTools":dynamic,"ephemeral":false})).await?
    };
    let thread_id = thread["thread"]["id"]
        .as_str()
        .ok_or_else(|| {
            CityError::new(
                "provider_incompatible",
                "Codex did not supply a thread identity.",
            )
        })?
        .to_string();
    rt.transact(|w| {
        run_task(w, &run.id)?;
        w.runs
            .iter_mut()
            .find(|r| r.id == run.id)
            .unwrap()
            .provider_thread = Some(thread_id.clone());
        Ok(())
    })?;
    if cancel.is_cancelled() {
        rpc.stop().await;
        return Err(CityError::new(
            "interrupted",
            "Interrupted before starting a Codex turn.",
        ));
    }
    let prompt = format!(
        "{}\nCurrent task: {}\nInstructions: {}\nCriteria: {}\nLast confirmed checkpoint: {}",
        run.context, task.title, task.instructions, task.criteria, task.checkpoint
    );
    let started = rpc
        .request(
            "turn/start",
            json!({"threadId":thread_id,"input":[{"type":"text","text":prompt}],"model":model}),
        )
        .await?;
    let turn_id = started_turn_id(&started)?;
    rt.transact(|w| {
        run_task(w, &run.id)?;
        w.runs
            .iter_mut()
            .find(|r| r.id == run.id)
            .unwrap()
            .provider_turn = Some(turn_id.clone());
        Ok(())
    })?;
    rt.acknowledge(&run.id)?;
    let mut interrupted = false;
    let mut output = String::new();
    loop {
        let message = if interrupted {
            rpc.next().await?
        } else {
            tokio::select! {_ = cancel.cancelled()=>{
                rpc.request("turn/interrupt",json!({"threadId":thread_id,"turnId":turn_id})).await.map_err(|_|CityError::new("outcome_unknown","Codex did not acknowledge the interruption request."))?;interrupted=true;continue;
            },message=rpc.next()=>message?}
        };
        let method = message["method"].as_str().unwrap_or_default();
        let p = &message["params"];
        if !event_for_turn(&message, &thread_id, &turn_id)? {
            continue;
        }
        match method {
            "item/tool/call" => {
                let tool = p["tool"]
                    .as_str()
                    .unwrap_or_default()
                    .strip_prefix("city_")
                    .unwrap_or("");
                let invocation = provider::execute_tool(
                    rt.clone(),
                    &run.id,
                    tool,
                    p["arguments"].clone(),
                    &cancel,
                );
                tokio::pin!(invocation);
                let mut deferred = VecDeque::new();
                let result = loop {
                    tokio::select! {
                        result=&mut invocation=>break result,
                        message=rpc.next()=>{
                            let message=message?;
                            if message["method"]=="turn/completed" && message["params"]["threadId"]==thread_id && message["params"]["turn"]["id"]==turn_id {
                                return Err(CityError::new("outcome_unknown", "The provider ended its turn while an application operation was unresolved. Inspect recorded effects before continuing."));
                            }
                            if deferred.len()+rpc.pending.len()>=128{return Err(CityError::new("outcome_unknown","The provider event queue exceeded its bound while an application operation was pending."));}
                            deferred.push_back(message);
                        }
                    }
                };
                deferred.append(&mut rpc.pending);
                rpc.pending = deferred;
                let success = result.is_ok();
                let value = match result {
                    Ok(v) => v,
                    Err(e) => json!(e),
                };
                rpc.send(json!({"id":message["id"],"result":{"contentItems":[{"type":"inputText","text":serde_json::to_string(&value)?}],"success":success}})).await?;
            }
            "item/commandExecution/requestApproval" | "item/fileChange/requestApproval" => {
                let allowed = approval(
                    rt.clone(),
                    run,
                    p.clone(),
                    method,
                    &cancel,
                    &mut rpc,
                    &message["id"],
                )
                .await?;
                rpc.send(json!({"id":message["id"],"result":{"decision":if allowed{"accept"}else{"decline"}}})).await?;
            }
            "item/agentMessage/delta" => {
                if let Some(delta) = p["delta"].as_str() {
                    output.push_str(delta);
                }
            }
            "item/completed" => {
                if p["item"]["type"] == "agentMessage" {
                    if let Some(text) = p["item"]["text"].as_str() {
                        output = text.into();
                    }
                }
            }
            "turn/completed" => {
                let status = p["turn"]["status"].as_str().unwrap_or("unknown");
                rpc.stop().await;
                return match status {
                    "completed" => Ok(output),
                    "interrupted" => Err(CityError::new(
                        "interrupted",
                        "Codex confirmed interruption.",
                    )),
                    "failed" => Err(CityError::new(
                        "provider_error",
                        p["turn"]["error"]["message"]
                            .as_str()
                            .unwrap_or("Codex turn failed."),
                    )),
                    _ => Err(CityError::new(
                        "outcome_unknown",
                        "Codex reported an unrecognized terminal state.",
                    )),
                };
            }
            "serverRequest/resolved" => {}
            _ => {
                if message.get("id").is_some() && !method.is_empty() {
                    rpc.send(json!({"id":message["id"],"error":{"code":-32601,"message":"Agent City does not support this request. No permission has been granted."}})).await?;
                }
            }
        }
        if output.len() > 128 * 1024 {
            return Err(CityError::new(
                "outcome_unknown",
                "Codex output exceeded its bound. Inspect the source thread before continuing.",
            ));
        }
    }
}

async fn approval(
    rt: Arc<Runtime>,
    run: &Run,
    args: Value,
    method: &str,
    cancel: &CancellationToken,
    rpc: &mut Rpc,
    request_id: &Value,
) -> Result<bool> {
    let decision_id = id();
    let digest = files::digest(serde_json::to_vec(&args)?.as_slice());
    rt.transact(|w| {
        let i = run_task(w, &run.id)?;
        let task = w.tasks[i].clone();
        w.tasks[i].state = TaskState::Waiting;
        w.decisions.push(Decision {
            id: decision_id.clone(),
            task_id: task.id,
            run_id: run.id.clone(),
            resident_id: run.resident_id.clone(),
            action: method.into(),
            scope: args["command"]
                .as_str()
                .or(args["cwd"].as_str())
                .unwrap_or("Inspect exact provider request")
                .into(),
            reason: args["reason"]
                .as_str()
                .unwrap_or("Codex requests permission for this operation.")
                .into(),
            arguments: args.clone(),
            digest,
            revision: 0,
            epoch: run.epoch,
            state: "pending".into(),
            expires_at: (chrono::Utc::now() + chrono::Duration::hours(24)).to_rfc3339(),
        });
        Ok(())
    })?;
    let mut updates = rt.subscribe();
    loop {
        let w = rt.snapshot();
        let decision = w.decisions.iter().find(|d| d.id == decision_id).unwrap();
        match decision.state.as_str() {
            "allowed" => {
                rt.transact(|w| {
                    let i = run_task(w, &run.id)?;
                    if w.tasks[i].state == TaskState::Waiting {
                        w.tasks[i].state = TaskState::Running;
                    }
                    Ok(())
                })?;
                return Ok(true);
            }
            "denied" | "invalidated" | "expired" => {
                rt.transact(|w| {
                    let i = run_task(w, &run.id)?;
                    if w.tasks[i].state == TaskState::Waiting {
                        w.tasks[i].state = TaskState::Running;
                    }
                    Ok(())
                })?;
                return Ok(false);
            }
            _ => {}
        }
        tokio::select! {_=cancel.cancelled()=>return Ok(false),_=updates.recv()=>{}, message=rpc.read()=>{
            let message=message?;let resolved=message["method"]=="serverRequest/resolved"&&message["params"]["requestId"]==*request_id;
            let terminal=message["method"]=="turn/completed"&&message["params"]["turn"]["id"]==args["turnId"];
            if resolved||terminal{rt.transact(|w|{if let Some(d)=w.decisions.iter_mut().find(|d|d.id==decision_id){d.state="invalidated".into();d.revision+=1;}Ok(())})?;if terminal{rpc.pending.push_back(message);}return Ok(false);}
            if rpc.pending.len()>=128{return Err(CityError::new("outcome_unknown","Approval event queue exceeded its limit."));}rpc.pending.push_back(message);
        },_=tokio::time::sleep(Duration::from_secs(86400))=>return Ok(false)}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn uncorrelated_turns_cannot_finish_or_retry_active_work() {
        assert_eq!(
            started_turn_id(&json!({"turn": {}})).unwrap_err().code,
            "outcome_unknown"
        );
        assert_eq!(
            started_turn_id(&json!({"turn":{"id":"current"}})).unwrap(),
            "current"
        );
        let late = json!({"method":"turn/completed","params":{"threadId":"thread","turn":{"id":"previous","status":"completed"}}});
        assert!(!event_for_turn(&late, "thread", "current").unwrap());
        assert!(!event_for_turn(&late, "another-thread", "previous").unwrap());
        assert!(event_for_turn(&late, "thread", "previous").unwrap());
        let missing = json!({"method":"turn/completed","params":{"threadId":"thread","turn":{"status":"completed"}}});
        assert_eq!(
            event_for_turn(&missing, "thread", "current")
                .unwrap_err()
                .code,
            "outcome_unknown"
        );
    }
}
