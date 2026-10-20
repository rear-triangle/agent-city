use crate::{
    runtime::{bounded, create_city, new_task, run_task},
    *,
};
use futures_util::StreamExt;
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};
use tokio_util::sync::CancellationToken;

fn local_url() -> Result<String> {
    let value =
        std::env::var("AGENT_CITY_OLLAMA_URL").unwrap_or_else(|_| "http://127.0.0.1:11434".into());
    let url = reqwest::Url::parse(&value)
        .map_err(|_| CityError::new("invalid_endpoint", "Invalid local model endpoint."))?;
    if url.scheme() != "http"
        || !matches!(
            url.host_str(),
            Some("127.0.0.1") | Some("localhost") | Some("[::1]")
        )
        || !url.username().is_empty()
        || url.password().is_some()
        || url.path() != "/"
        || url.query().is_some()
    {
        return Err(CityError::new(
            "invalid_endpoint",
            "Local execution requires an HTTP loopback origin without credentials or a path.",
        ));
    }
    Ok(value.trim_end_matches('/').to_string())
}
pub async fn local_profiles() -> Result<Vec<Profile>> {
    let local = local_url()?;
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(5))
        .build()?;
    let data: Value = client
        .get(format!("{local}/api/tags"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let loaded: Value = client
        .get(format!("{local}/api/ps"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let mut profiles = vec![];
    for model in data["models"].as_array().unwrap_or(&vec![]) {
        let name = model["name"].as_str().unwrap_or_default();
        let details: Value = client
            .post(format!("{local}/api/show"))
            .json(&json!({"model":name}))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let caps = details["capabilities"].as_array();
        let tools = caps.is_some_and(|c| c.iter().any(|v| v == "tools"))
            && caps.is_some_and(|c| c.iter().any(|v| v == "completion"));
        let context = details["model_info"]
            .as_object()
            .and_then(|o| o.iter().find(|(k, _)| k.ends_with(".context_length")))
            .and_then(|(_, v)| v.as_u64())
            .unwrap_or(8192);
        profiles.push(Profile {
            id: format!("ollama:{name}"),
            model: name.into(),
            provider: "ollama".into(),
            available: true,
            tools,
            size: model["size"].as_u64().unwrap_or(0),
            context_length: context,
            loaded: loaded["models"]
                .as_array()
                .is_some_and(|models| models.iter().any(|m| m["name"] == name)),
            detail: if tools {
                "Installed locally; tool support advertised. Each run validates tool output."
            } else {
                "Text or embedding profile; not admitted for managed tool work."
            }
            .into(),
        });
    }
    Ok(profiles)
}
fn schema(name: &str, description: &str, properties: Value, required: Vec<&str>) -> Value {
    json!({"type":"function","function":{"name":name,"description":description,"parameters":{"type":"object","properties":properties,"required":required,"additionalProperties":false}}})
}
pub fn tools_schema() -> Vec<Value> {
    vec![
    schema("list_files","List files within the approved project folder.",json!({"path":{"type":"string"}}),vec!["path"]),
    schema("read_file","Read a UTF-8 file within the approved project folder, at most 128 KiB.",json!({"path":{"type":"string"}}),vec!["path"]),
    schema("list_skills","List locally approved skill version IDs, descriptions, and missing capabilities. Listing instructions grants no tools.",json!({}),vec![]),
    schema("read_skill","Read an archived supporting file from an assigned skill version. This never executes scripts or grants tools.",json!({"skill_id":{"type":"string"},"path":{"type":"string"}}),vec!["skill_id","path"]),
    schema("write_file","Propose one exact file write. Waits for a Chief decision. Parent folder must already exist.",json!({"path":{"type":"string"},"content":{"type":"string"},"reason":{"type":"string"}}),vec!["path","content","reason"]),
    schema("delegate","Mayor only. Assign a distinct bounded task to a specialist. Reuses an idle resident with this role. End planning after delegation; reviewed results trigger your continuation.",json!({"title":{"type":"string"},"role":{"type":"string"},"model":{"type":"string","description":"Optional installed model name. Hosted profiles require the Chief to have enabled hosted inference."},"instructions":{"type":"string"},"criteria":{"type":"string"},"skill_ids":{"type":"array","items":{"type":"string"},"description":"Optional approved version IDs from list_skills."},"dependencies":{"type":"array","description":"Existing worker task IDs returned by delegate, never filenames. Omit when independent.","items":{"type":"string"}},"city_id":{"type":"string","description":"Optional existing cooperating city in this project. Its mayor coordinates the workstream."}}),vec!["title","role","instructions","criteria"]),
    schema("send_message","Send a message to a resident using permitted roads. Sender identity comes from the runtime.",json!({"recipient":{"type":"string"},"purpose":{"type":"string"},"body":{"type":"string"},"reply_to":{"type":"string","description":"Optional ID of a received message this answers."}}),vec!["recipient","purpose","body"]),
    schema("remember","Propose a task note with provenance. Cannot replace a Chief instruction.",json!({"content":{"type":"string"},"source":{"type":"string"}}),vec!["content","source"]),
    schema("review_result","Only the assigned reviewer may accept or reject its worker's result. Explain evidence and unresolved issues.",json!({"accept":{"type":"boolean"},"evidence":{"type":"string"}}),vec!["accept","evidence"]),
    schema("manage_residents","Mayor only. Retire an idle specialist, recall a retired specialist, or merge two idle specialists into a held successor. All sources stay archived. Inherited notes need Chief review.",json!({"action":{"type":"string","enum":["retire","recall","merge"]},"resident_ids":{"type":"array","items":{"type":"string"}},"name":{"type":"string"},"role":{"type":"string"}}),vec!["action","resident_ids"]),
    schema("found_city","Mayor only. Create a cooperating city within this project for a substantial separate workstream. No inference starts merely from founding a city.",json!({"name":{"type":"string"},"purpose":{"type":"string"},"skill_ids":{"type":"array","items":{"type":"string"}}}),vec!["name","purpose"]),
]
}
pub fn tools_for_run(world: &World, run: &Run) -> Vec<Value> {
    let task = world.tasks.iter().find(|t| t.id == run.task_id).unwrap();
    let mayor = world
        .cities
        .iter()
        .any(|c| c.id == task.city_id && c.mayor_id == run.resident_id);
    tools_schema()
        .into_iter()
        .filter(|schema| {
            let name = schema["function"]["name"].as_str().unwrap_or_default();
            if ["read_file", "list_files", "read_skill", "list_skills"].contains(&name) {
                return true;
            }
            if task.kind == "review" {
                return name == "review_result";
            }
            if name == "manage_residents" {
                return mayor;
            }
            if ["delegate", "found_city"].contains(&name) {
                return mayor && ["goal", "work"].contains(&task.kind.as_str());
            }
            name != "review_result"
        })
        .collect()
}
fn arg<'a>(args: &'a Value, name: &str) -> Result<&'a str> {
    args[name]
        .as_str()
        .ok_or_else(|| CityError::new("invalid_tool", format!("Missing text argument: {name}")))
}
pub async fn local_run(rt: Arc<Runtime>, run: &Run, cancel: CancellationToken) -> Result<String> {
    let local = local_url()?;
    let client = reqwest::Client::builder()
        .no_proxy()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(180))
        .build()?;
    let limit = rt.snapshot().settings.max_tool_steps;
    let tools = tools_for_run(&rt.snapshot(), run);
    let mut messages = run.messages.clone();
    let reviewing = rt
        .snapshot()
        .tasks
        .iter()
        .any(|t| t.id == run.task_id && t.kind == "review");
    let synthesizing =
        rt.snapshot().tasks.iter().any(|t| {
            t.id == run.task_id && ["finalize", "finalize_work"].contains(&t.kind.as_str())
        });
    if synthesizing {
        messages.push(json!({"role":"user","content":"Return only a JSON object with result (the final answer based on the reviewed artifacts and source evidence) and limitations (remaining limitations, or an empty string). Do not perform any additional operations in this synthesis turn."}));
    }
    if reviewing {
        messages.push(json!({"role":"user","content":"Review the recorded worker artifacts against the task criteria using the supplied confirmed source excerpts. Return only a JSON object with accept (boolean) and evidence (string). The runtime records this object through review_result. Accept only if the actual evidence supports the criteria. Do not request new delegation."}));
    }
    let mut answer = String::new();
    let mut invalid_finishes = 0;
    for step in 0..limit {
        if serde_json::to_vec(&messages)?.len() > 36000 {
            return Err(CityError::new("context_limit","The retained conversation exceeds this local profile's conservative input budget. Required instructions were not dropped. Review the checkpoint and reduce the input scope."));
        }
        let mut request_body = json!({"model":run.model,"messages":messages,"stream":true,"think":false,"keep_alive":"5m","options":{"num_ctx":16384,"num_predict":4096,"temperature":0.2}});
        if reviewing {
            request_body["format"] = json!({"type":"object","properties":{"accept":{"type":"boolean"},"evidence":{"type":"string"}},"required":["accept","evidence"],"additionalProperties":false});
        } else if synthesizing {
            request_body["format"] = json!({"type":"object","properties":{"result":{"type":"string","minLength":1},"limitations":{"type":"string"}},"required":["result","limitations"],"additionalProperties":false});
        } else {
            request_body["tools"] = json!(tools);
        }
        let request = client
            .post(format!("{local}/api/chat"))
            .json(&request_body)
            .send();
        let response = tokio::select! {_ = cancel.cancelled()=>return Ok(answer),r=request=>r?}
            .error_for_status()?;
        rt.acknowledge(&run.id)?;
        let mut stream = response.bytes_stream();
        let mut buffer = Vec::new();
        let mut content = String::new();
        let mut calls = vec![];
        let mut done = false;
        loop {
            let chunk = tokio::select! {_ = cancel.cancelled()=>{rt.checkpoint(&run.id,messages.clone(),step)?;return Ok(content);},chunk=stream.next()=>chunk};
            let Some(chunk) = chunk else {
                break;
            };
            let chunk = chunk?;
            buffer.extend_from_slice(&chunk);
            if buffer.len() > 1024 * 1024 {
                return Err(CityError::new(
                    "provider_incompatible",
                    "A local response exceeded the event limit.",
                ));
            }
            while let Some(end) = buffer.iter().position(|b| *b == b'\n') {
                let line: Vec<u8> = buffer.drain(..=end).collect();
                if line.iter().all(|b| b.is_ascii_whitespace()) {
                    continue;
                }
                let event: Value = serde_json::from_slice(&line)?;
                if event["error"].is_string() {
                    return Err(CityError::new(
                        "provider_error",
                        event["error"].as_str().unwrap(),
                    ));
                }
                if let Some(text) = event["message"]["content"].as_str() {
                    content.push_str(text);
                }
                if let Some(tools) = event["message"]["tool_calls"].as_array() {
                    calls.extend(tools.clone());
                }
                if event["done"].as_bool() == Some(true) {
                    done = true;
                }
                if content.len() > 128 * 1024 || calls.len() > 16 {
                    return Err(CityError::new(
                        "provider_incompatible",
                        "The model exceeded its output or tool-call limit.",
                    ));
                }
            }
        }
        if !done {
            return Err(CityError::new("provider_disconnected","The local host closed the response before confirming completion. Confirmed tool results are retained."));
        }
        let mut assistant = json!({"role":"assistant","content":content});
        if !calls.is_empty() {
            assistant["tool_calls"] = json!(calls);
        }
        messages.push(assistant);
        rt.checkpoint(&run.id, messages.clone(), step + 1)?;
        if calls.is_empty() {
            if synthesizing {
                if let Ok(answer) = serde_json::from_str::<Value>(&content) {
                    if let (Some(result), Some(limitations)) =
                        (answer["result"].as_str(), answer["limitations"].as_str())
                    {
                        if !result.trim().is_empty() {
                            return Ok(if limitations.trim().is_empty() {
                                result.into()
                            } else {
                                format!("{result}\n\nLimitations: {limitations}")
                            });
                        }
                    }
                }
            }
            if reviewing {
                if let Ok(decision) = serde_json::from_str::<Value>(&content) {
                    if decision["accept"].is_boolean() && decision["evidence"].is_string() {
                        execute_tool(
                            rt.clone(),
                            &run.id,
                            "review_result",
                            decision.clone(),
                            &cancel,
                        )
                        .await?;
                        return Ok(decision["evidence"].as_str().unwrap().into());
                    }
                }
            }
            let world = rt.snapshot();
            let task = world.tasks.iter().find(|t| t.id == run.task_id).unwrap();
            let bad_tool_calls = world
                .runs
                .iter()
                .find(|r| r.id == run.id)
                .unwrap()
                .tool_receipts
                .iter()
                .filter(|r| {
                    r.state == "failed"
                        && ["invalid_tool", "invalid_dependency"]
                            .iter()
                            .any(|code| r.result["error"] == *code)
                })
                .count();
            if bad_tool_calls >= 2 {
                return Err(CityError::new("provider_incompatible","The model repeatedly supplied invalid tool arguments. Confirmed effects are retained for a compatible profile."));
            }
            let needs_review = task.kind == "review"
                && !world
                    .runs
                    .iter()
                    .find(|r| r.id == run.id)
                    .unwrap()
                    .tool_receipts
                    .iter()
                    .any(|r| r.name == "review_result" && r.state == "confirmed");
            let handed_off = ["goal", "work"].contains(&task.kind.as_str())
                && world.tasks.iter().any(|child| {
                    child.parent_id.as_ref() == Some(&task.id) && child.kind == "work"
                })
                && world
                    .runs
                    .iter()
                    .find(|attempt| attempt.id == run.id)
                    .unwrap()
                    .tool_receipts
                    .iter()
                    .any(|receipt| receipt.name == "delegate" && receipt.state == "confirmed");
            if handed_off && (content.trim().is_empty() || content.contains("<|tool")) {
                return Ok(
                    "Delegated assignments have been recorded. Waiting for reviewed results."
                        .into(),
                );
            }
            if needs_review || content.trim().is_empty() || content.contains("<|tool") {
                invalid_finishes += 1;
                if invalid_finishes >= 2 {
                    return Err(CityError::new("provider_incompatible","The model ended twice without a valid result or required review decision. Confirmed effects are retained."));
                }
                messages.push(json!({"role":"user","content":if needs_review{"You are the assigned reviewer. Inspect the worker evidence already in context and call review_result with accept true or false and specific evidence before finishing. File tool paths must be relative to the project folder, such as brief.txt, never absolute."}else{"Return a readable result using the actual observed tool evidence. Do not emit protocol control tokens."}}));
                continue;
            }
            return Ok(content);
        }
        answer = content;
        for call in calls {
            if cancel.is_cancelled() {
                return Ok(answer);
            }
            let name = arg(&call["function"], "name")?;
            let args = call["function"]["arguments"].clone();
            let result = execute_tool(rt.clone(), &run.id, name, args, &cancel).await;
            let value = match result {
                Ok(v) => v,
                Err(e) => json!({"error":e.code,"message":e.message}),
            };
            messages.push(
                json!({"role":"tool","tool_name":name,"content":serde_json::to_string(&value)?}),
            );
            rt.checkpoint(&run.id, messages.clone(), step + 1)?;
        }
    }
    Err(CityError::new(
        "budget_exhausted",
        "The tool-step limit was reached. Confirmed work is saved; inspect it before continuing.",
    ))
}
pub async fn execute_tool(
    rt: Arc<Runtime>,
    run_id: &str,
    name: &str,
    args: Value,
    cancel: &CancellationToken,
) -> Result<Value> {
    if serde_json::to_vec(&args)?.len() > 128 * 1024 {
        return Err(CityError::new(
            "size_limit",
            "Tool arguments are too large.",
        ));
    }
    let w = rt.snapshot();
    let index = run_task(&w, run_id)?;
    let task = w.tasks[index].clone();
    if task.state == TaskState::Pausing || cancel.is_cancelled() {
        return Err(CityError::new("interrupted", "Work is held."));
    }
    let project = w.projects.iter().find(|p| p.id == task.project_id).unwrap();
    let receipt_id = id();
    rt.transact(|w| {
        run_task(w, run_id)?;
        w.runs
            .iter_mut()
            .find(|r| r.id == run_id)
            .unwrap()
            .tool_receipts
            .push(ToolReceipt {
                id: receipt_id.clone(),
                name: name.into(),
                arguments: args.clone(),
                state: "intent".into(),
                result: Value::Null,
            });
        Ok(())
    })?;
    let result:Result<Value>=async { match name {
        "read_skill"=>{
            let skill_id=arg(&args,"skill_id")?;let path=arg(&args,"path")?;
            let run=w.runs.iter().find(|r|r.id==run_id).unwrap();
            if !run.skill_ids.iter().any(|id|id==skill_id){Err(CityError::new("denied_policy","This skill version is not assigned to this run."))}
            else {w.skills.iter().find(|s|s.id==skill_id).and_then(|s|s.files.iter().find(|(p,_)|p==path)).map(|(_,content)|json!({"content":content,"skillVersion":skill_id})).ok_or_else(||CityError::new("missing_dependency","This supporting file was not retained in the skill package."))}
        },
        "list_files"=>files::list(&project.workspace,arg(&args,"path")?).map(|entries|json!({"entries":entries})),
        "read_file"=>files::read(&project.workspace,arg(&args,"path")?).map(|text|json!({"content":text})),
        "write_file"=>write_with_decision(rt.clone(),run_id,&task,&project.workspace,args.clone(),cancel).await,
        "delegate"=>rt.transact(|w|delegate(w,run_id,&args)),
        "send_message"=>rt.transact(|w|route(w,run_id,&args)),
        "list_skills"=>{let mut seen=std::collections::HashSet::new();Ok(json!(rt.snapshot().skills.iter().rev().filter(|skill|seen.insert(skill.source.clone())).take(50).map(|skill|json!({"id":skill.id,"name":skill.name,"description":skill.description.chars().take(200).collect::<String>(),"available":skill.available,"limitation":skill.limitation})).collect::<Vec<_>>()))},
        "remember"=>rt.transact(|w|{
            let i=run_task(w,run_id)?;let resident=w.tasks[i].resident_id.clone();let content=arg(&args,"content")?;bounded(content,8000)?;
            let note=Memory{id:id(),resident_id:resident,kind:"task_note".into(),content:content.into(),source:format!("Run {run_id}: {}",arg(&args,"source")?),revision:w.revision+1,accepted:false,created_at:now()};let note_id=note.id.clone();w.memories.push(note);Ok(json!({"id":note_id,"state":"proposed","message":"Retained as proposed knowledge; Chief instructions remain authoritative."}))}),
        "review_result"=>rt.transact(|w|{
            let i=run_task(w,run_id)?;let review=w.tasks[i].clone();let target=review.reviewed_task.ok_or_else(||CityError::new("denied_policy","This run is not assigned to review a result."))?;
            let j=w.tasks.iter().position(|t|t.id==target&&t.state==TaskState::Review&&!t.stale).ok_or_else(||CityError::new("stale_result","The result is no longer awaiting this review."))?;
            let accepted=args["accept"].as_bool().ok_or_else(||CityError::new("invalid_tool","Provide accept as a boolean."))?;let evidence=arg(&args,"evidence")?;bounded(evidence,8000)?;
            let attempts=w.tasks.iter().filter(|t|t.reviewed_task.as_ref()==Some(&target)).count();
            w.tasks[j].state=if accepted{TaskState::Accepted}else if attempts<3{TaskState::Queued}else{TaskState::Blocked};w.tasks[j].blocker=(!accepted).then(||evidence.into());w.tasks[j].revision+=1;
            if !accepted&&attempts<3{w.tasks[j].instructions.push_str(&format!("\nCoordinator revision request: {evidence}. Inspect confirmed artifacts and sources, revise the result, and return specific evidence."));}
            if let Some(result)=w.results.iter_mut().rev().find(|r|r.task_id==target){result.state=if accepted{"reviewed"}else{"revision_needed"}.into();}
            if accepted {let source_run=w.tasks[j].run_id.clone().unwrap_or_default();for note in &mut w.memories {if note.kind=="task_note"&&note.source.starts_with(&format!("Run {source_run}:")){note.accepted=true;}}}
            w.event("internal_review",&target,evidence);Ok(json!({"accepted":accepted,"taskId":target}))}),
        "manage_residents"=>rt.transact(|w|{
            mayor_authority(w,run_id)?;let i=run_task(w,run_id)?;let city=w.tasks[i].city_id.clone();
            let ids:Vec<String>=args["resident_ids"].as_array().ok_or_else(||CityError::new("invalid_tool","Provide resident_ids."))?.iter().filter_map(Value::as_str).map(String::from).collect();
            if ids.is_empty()||ids.iter().any(|id|!w.residents.iter().any(|r|&r.id==id&&r.city_id==city)){return Err(CityError::new("denied_policy","Only residents in this mayor's city can be managed."));}
            let action=match arg(&args,"action")?{"retire" if ids.len()==1=>Action::Retire{resident_id:ids[0].clone()},"recall" if ids.len()==1=>Action::Recall{resident_id:ids[0].clone()},"merge"=>Action::Merge{resident_ids:ids,name:arg(&args,"name")?.into(),role:arg(&args,"role")?.into()},_=>return Err(CityError::new("invalid_tool","Invalid resident action."))};
            let target=crate::runtime::apply_owner(w,&action,None)?;Ok(json!({"targetId":target,"confirmed":true}))
        }),
        "found_city"=>rt.transact(|w|{
            mayor_authority(w,run_id)?;let i=run_task(w,run_id)?;let project=w.tasks[i].project_id.clone();
            if w.cities.iter().filter(|c|c.project_id==project).count()>=4{return Err(CityError::new("budget_exhausted","This project has reached its four-city limit."));}
            let name=arg(&args,"name")?;let purpose=arg(&args,"purpose")?;bounded(name,100)?;bounded(purpose,4000)?;
            let selected=skills::assignment(&w.skills,"Mayor",purpose,args.get("skill_ids"))?;
            let old=w.tasks[i].resident_id.clone();let (city,mayor)=create_city(w,&project,name,purpose);w.residents.iter_mut().find(|r|r.id==mayor).unwrap().skills=selected;w.roads.push(Road{id:id(),from:old,to:mayor.clone(),enabled:true,revision:0,presentation:"auto".into(),rule_version:1});Ok(json!({"cityId":city,"mayorId":mayor}))}),
        _=>Err(CityError::new("unsupported_tool",format!("Tool {name} is not available in this profile."))),
    }}.await;
    let value = match &result {
        Ok(v) => v.clone(),
        Err(e) => json!({"error":e.code,"message":e.message}),
    };
    rt.transact(|w| {
        let i = run_task(w, run_id)?;
        if w.tasks[i].state == TaskState::Waiting
            && !w
                .decisions
                .iter()
                .any(|d| d.run_id == run_id && d.state == "pending")
        {
            w.tasks[i].state = TaskState::Running;
        }
        w.tasks[i].checkpoint = format!(
            "{}: {}",
            name,
            serde_json::to_string(&value)?
                .chars()
                .take(1600)
                .collect::<String>()
        );
        let run = w.runs.iter_mut().find(|r| r.id == run_id).unwrap();
        let receipt = run
            .tool_receipts
            .iter_mut()
            .find(|r| r.id == receipt_id)
            .unwrap();
        receipt.state = if result.is_ok() {
            "confirmed"
        } else {
            "failed"
        }
        .into();
        receipt.result = value;
        w.event(
            "tool_result",
            &task.id,
            format!(
                "{name}: {}",
                if result.is_ok() {
                    "confirmed"
                } else {
                    "failed"
                }
            ),
        );
        Ok(())
    })?;
    result
}
fn mayor_authority(w: &World, run_id: &str) -> Result<()> {
    let i = run_task(w, run_id)?;
    let t = &w.tasks[i];
    let c = w.cities.iter().find(|c| c.id == t.city_id).unwrap();
    let r = w.runs.iter().find(|r| r.id == run_id).unwrap();
    if t.resident_id != c.mayor_id || r.epoch != c.epoch {
        return Err(CityError::new(
            "denied_policy",
            "Only the current mayor can coordinate this city.",
        ));
    }
    Ok(())
}
fn delegate(w: &mut World, run_id: &str, args: &Value) -> Result<Value> {
    mayor_authority(w, run_id)?;
    let i = run_task(w, run_id)?;
    let parent = w.tasks[i].clone();
    if ["review", "finalize", "finalize_work"].contains(&parent.kind.as_str()) {
        return Err(CityError::new(
            "denied_policy",
            "Finish the assigned review or synthesis using existing results.",
        ));
    }
    let title = arg(args, "title")?;
    let role = arg(args, "role")?;
    let instructions = arg(args, "instructions")?;
    let criteria = arg(args, "criteria")?;
    for (s, n) in [
        (title, 500),
        (role, 80),
        (instructions, 8000),
        (criteria, 4000),
    ] {
        bounded(s, n)?;
    }
    if w.tasks
        .iter()
        .filter(|t| t.goal_id == parent.goal_id && t.kind == "work")
        .count()
        >= 12
    {
        return Err(CityError::new(
            "budget_exhausted",
            "The goal has reached its task limit.",
        ));
    }
    if let Some(existing) = w.tasks.iter().find(|t| {
        t.project_id == parent.project_id
            && t.title.trim().eq_ignore_ascii_case(title.trim())
            && !t.state.finished()
    }) {
        return Err(CityError::new(
            "assignment_conflict",
            format!(
                "An existing assignment already owns this work: {}",
                existing.id
            ),
        ));
    }
    let deps: Vec<String> = args["dependencies"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect()
        })
        .unwrap_or_default();
    if deps.iter().any(|id| {
        !w.tasks
            .iter()
            .any(|t| t.id == *id && t.goal_id == parent.goal_id && t.kind == "work")
    }) {
        return Err(CityError::new(
            "invalid_dependency",
            "Dependencies must reference existing worker tasks in this goal.",
        ));
    }
    let destination = args["city_id"]
        .as_str()
        .unwrap_or(&parent.city_id)
        .to_string();
    let city = w
        .cities
        .iter()
        .find(|c| c.id == destination && c.project_id == parent.project_id)
        .ok_or_else(|| CityError::new("denied_policy", "Delegation must stay in this project."))?
        .clone();
    let cross_city = city.id != parent.city_id;
    let selected_skills = skills::assignment(
        &w.skills,
        if cross_city { "Mayor" } else { role },
        &format!("{} {title} {instructions}", city.purpose),
        args.get("skill_ids"),
    )?;
    let existing = w.residents.iter().find(|r| {
        r.city_id == parent.city_id
            && r.role.eq_ignore_ascii_case(role)
            && r.lifecycle != "merged"
            && !w
                .tasks
                .iter()
                .any(|t| t.resident_id == r.id && !t.state.finished())
    });
    let resident = if cross_city {
        city.mayor_id.clone()
    } else if let Some(r) = existing {
        let rid = r.id.clone();
        w.residents
            .iter_mut()
            .find(|r| r.id == rid)
            .unwrap()
            .lifecycle = "active".into();
        rid
    } else {
        if w.residents
            .iter()
            .filter(|r| r.city_id == parent.city_id && r.lifecycle == "active")
            .count()
            >= 12
        {
            return Err(CityError::new(
                "budget_exhausted",
                "This city has reached its resident limit.",
            ));
        }
        let rid = id();
        let skills = skills::select(&w.skills, role);
        w.residents.push(Resident {
            id: rid.clone(),
            city_id: parent.city_id.clone(),
            name: format!("{} {}", role, w.residents.len()),
            role: role.into(),
            lifecycle: "active".into(),
            held: false,
            skills,
            ancestors: vec![],
            successor: None,
            model: None,
            revision: 0,
        });
        w.roads.push(Road {
            id: id(),
            from: rid.clone(),
            to: parent.resident_id.clone(),
            enabled: true,
            presentation: "auto".into(),
            rule_version: 1,
            revision: 0,
        });
        rid
    };
    w.residents
        .iter_mut()
        .find(|r| r.id == resident)
        .unwrap()
        .skills = selected_skills.clone();
    if let Some(model) = args["model"].as_str() {
        bounded(model, 200)?;
        if model.starts_with("codex:")
            && (w.settings.policy != "auto"
                || !w
                    .settings
                    .preferred_model
                    .as_ref()
                    .is_some_and(|m| m.starts_with("codex:")))
        {
            return Err(CityError::new(
                "denied_policy",
                "The Chief has not enabled hosted inference for this project.",
            ));
        }
        w.residents
            .iter_mut()
            .find(|r| r.id == resident)
            .unwrap()
            .model = Some(model.into());
    }
    let mut task = new_task(
        &parent.project_id,
        &destination,
        &resident,
        title,
        instructions,
        criteria,
        Some(parent.goal_id.clone()),
    );
    task.assigned_skills = Some(selected_skills);
    task.parent_id = Some(parent.id.clone());
    task.kind = "work".into();
    task.dependencies = deps;
    let task_id = task.id.clone();
    w.tasks.push(task);
    let msg = json!({"recipient":resident,"purpose":"assignment","body":instructions});
    let delivery = route(w, run_id, &msg)?;
    if delivery["state"] != "delivered" {
        return Err(CityError::new(
            "route_blocked",
            "The assignment cannot reach its owner through an enabled road.",
        ));
    }
    w.event("delegated", &task_id, format!("Assigned {title} to {role}"));
    Ok(json!({"taskId":task_id,"residentId":resident,"state":"queued"}))
}
pub(crate) fn route(w: &mut World, run_id: &str, args: &Value) -> Result<Value> {
    let i = run_task(w, run_id)?;
    let task = w.tasks[i].clone();
    let sender = task.resident_id.clone();
    let recipient = arg(args, "recipient")?;
    let purpose = arg(args, "purpose")?;
    let body = arg(args, "body")?;
    bounded(body, 16000)?;
    bounded(purpose, 200)?;
    if recipient == sender {
        return Err(CityError::new(
            "invalid_route",
            "A resident cannot send to itself.",
        ));
    }
    let target = w
        .residents
        .iter()
        .find(|r| r.id == recipient && r.lifecycle == "active")
        .ok_or_else(|| CityError::new("invalid_route", "Recipient is not active."))?;
    let target_city = w.cities.iter().find(|c| c.id == target.city_id).unwrap();
    if target_city.project_id != task.project_id {
        return Err(CityError::new(
            "denied_policy",
            "Cross-project messages require a separate explicit grant.",
        ));
    }
    if let Some(m) = w.messages.iter().find(|m| {
        m.from == sender
            && m.to == recipient
            && m.task_id == task.id
            && m.body == body
            && m.purpose == purpose
    }) {
        return Ok(json!({"messageId":m.id,"state":m.state}));
    }
    if w.messages
        .iter()
        .filter(|m| {
            w.tasks
                .iter()
                .any(|t| t.id == m.task_id && t.goal_id == task.goal_id)
        })
        .count()
        >= 30
    {
        return Err(CityError::new(
            "budget_exhausted",
            "The goal reached its message budget.",
        ));
    }
    let route = permitted_route(w, &sender, recipient);
    if w.messages
        .iter()
        .filter(|m| {
            m.to == recipient
                && m.state == "delivered"
                && m.acknowledged_at.is_none()
                && !message_expired(m)
        })
        .count()
        >= 100
    {
        return Err(CityError::new(
            "mailbox_full",
            "The recipient has 100 unread messages. Wait for it to consume its inbox.",
        ));
    }
    let city_pending = w
        .messages
        .iter()
        .filter(|m| {
            m.state == "delivered"
                && m.acknowledged_at.is_none()
                && !message_expired(m)
                && w.residents
                    .iter()
                    .any(|r| r.id == m.to && r.city_id == target.city_id)
        })
        .count();
    if city_pending >= 1000 {
        return Err(CityError::new(
            "mailbox_full",
            "The receiving city has 1,000 unread messages.",
        ));
    }
    let parent = if let Some(reply) = args["reply_to"].as_str() {
        Some(
            w.messages
                .iter()
                .find(|m| {
                    m.id == reply
                        && m.to == sender
                        && m.state == "delivered"
                        && !message_expired(m)
                        && w.tasks
                            .iter()
                            .any(|t| t.id == m.task_id && t.goal_id == task.goal_id)
                })
                .ok_or_else(|| {
                    CityError::new(
                        "invalid_causation",
                        "Reply only to a received, current message in this goal.",
                    )
                })?,
        )
    } else {
        w.messages.iter().rev().find(|m| {
            m.to == sender
                && m.acknowledged_at.is_some()
                && w.tasks
                    .iter()
                    .any(|t| t.id == m.task_id && t.goal_id == task.goal_id)
        })
    };
    let depth = parent.map_or(1, |m| m.causal_depth.saturating_add(1));
    if depth > 12 {
        return Err(CityError::new("causal_limit","This conversation reached twelve processing exchanges. Ask the Chief to review the goal before starting another exchange."));
    }
    let attempt = route_attempt(
        w,
        &route,
        if route.is_empty() {
            "blocked"
        } else {
            "delivered"
        },
    );
    let message = Message {
        goal_id: task.goal_id.clone(),
        city_id: target.city_id.clone(),
        digest: files::digest(body.as_bytes()),
        conversation_id: parent
            .map(|m| m.conversation_id.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(id),
        causation_id: parent.map(|m| m.id.clone()),
        causal_depth: depth,
        expires_at: Some((chrono::Utc::now() + chrono::Duration::hours(24)).to_rfc3339()),
        route_attempts: vec![attempt],
        id: id(),
        from: sender,
        to: recipient.into(),
        task_id: task.id,
        purpose: purpose.into(),
        body: body.into(),
        route: route.clone(),
        state: if route.is_empty() {
            "blocked"
        } else {
            "delivered"
        }
        .into(),
        at: now(),
        acknowledged_at: None,
    };
    let result = json!({"messageId":message.id,"state":message.state,"route":route});
    w.messages.push(message);
    Ok(result)
}
pub(crate) fn permitted_route(w: &World, sender: &str, recipient: &str) -> Vec<String> {
    let Some(a) = w
        .residents
        .iter()
        .find(|r| r.id == sender && r.lifecycle == "active")
    else {
        return vec![];
    };
    let Some(b) = w
        .residents
        .iter()
        .find(|r| r.id == recipient && r.lifecycle == "active")
    else {
        return vec![];
    };
    let ca = w.cities.iter().find(|c| c.id == a.city_id).unwrap();
    let cb = w.cities.iter().find(|c| c.id == b.city_id).unwrap();
    if ca.project_id != cb.project_id {
        return vec![];
    }
    let road = |a: &str, b: &str| {
        w.roads
            .iter()
            .find(|r| r.enabled && ((r.from == a && r.to == b) || (r.to == a && r.from == b)))
            .map(|r| r.id.clone())
    };
    if let Some(direct) = road(sender, recipient) {
        return vec![direct];
    }
    let mut chain = vec![
        sender,
        ca.mayor_id.as_str(),
        cb.mayor_id.as_str(),
        recipient,
    ];
    chain.dedup();
    let mut route = vec![];
    for pair in chain.windows(2) {
        match road(pair[0], pair[1]) {
            Some(id) => route.push(id),
            None => return vec![],
        }
    }
    route
}
async fn write_with_decision(
    rt: Arc<Runtime>,
    run_id: &str,
    task: &Task,
    workspace: &str,
    args: Value,
    cancel: &CancellationToken,
) -> Result<Value> {
    let relative = arg(&args, "path")?;
    let content = arg(&args, "content")?;
    let reason = arg(&args, "reason")?;
    bounded(content, 128 * 1024)?;
    bounded(reason, 1000)?;
    let path = files::workspace_path(workspace, relative, false)?;
    let before = if path.exists() {
        Some(files::digest(&std::fs::read(&path)?))
    } else {
        None
    };
    let digest = files::digest(
        serde_json::to_vec(&json!({"content":content,"before":before,"path":path}))?.as_slice(),
    );
    if before.as_deref() == Some(files::digest(content.as_bytes()).as_str()) {
        return Ok(
            json!({"path":path,"digest":before,"unchanged":true,"message":"The file already contains this exact content. No write was performed."}),
        );
    }
    let decision_id = id();
    rt.transact(|w| {
        let i = run_task(w, run_id)?;
        if w.decisions.iter().any(|d| {
            w.tasks
                .iter()
                .any(|t| t.id == d.task_id && t.goal_id == task.goal_id)
                && d.action == "write_file"
                && d.digest == digest
                && d.state == "denied"
        }) {
            return Err(CityError::new(
                "denied_policy",
                "The Chief already denied this exact operation for this goal.",
            ));
        }
        let epoch = w
            .cities
            .iter()
            .find(|c| c.id == task.city_id)
            .unwrap()
            .epoch;
        w.decisions.push(Decision {
            id: decision_id.clone(),
            task_id: task.id.clone(),
            run_id: run_id.into(),
            resident_id: task.resident_id.clone(),
            action: "write_file".into(),
            arguments: args.clone(),
            digest: digest.clone(),
            reason: reason.into(),
            scope: path.display().to_string(),
            state: "pending".into(),
            revision: 0,
            epoch,
            expires_at: (chrono::Utc::now() + chrono::Duration::hours(24)).to_rfc3339(),
        });
        w.tasks[i].state = TaskState::Waiting;
        Ok(())
    })?;
    let mut updates = rt.subscribe();
    loop {
        let world = rt.snapshot();
        let decision = world
            .decisions
            .iter()
            .find(|d| d.id == decision_id)
            .unwrap();
        match decision.state.as_str() {
            "allowed" => break,
            "denied" | "invalidated" | "expired" => {
                return Err(CityError::new(
                    "denied_policy",
                    "The file write was not authorized. Use an already permitted alternative.",
                ))
            }
            _ => {}
        }
        tokio::select! {_ = cancel.cancelled()=>return Err(CityError::new("interrupted","The file write was interrupted before execution.")), _=updates.recv()=>{}, _=tokio::time::sleep(Duration::from_secs(86400))=>return Err(CityError::new("expired","The file decision expired."))}
    }
    if cancel.is_cancelled() {
        return Err(CityError::new(
            "interrupted",
            "The file write was interrupted before execution.",
        ));
    }
    let checked = files::workspace_path(workspace, relative, false)?;
    let current = if checked.exists() {
        Some(files::digest(&std::fs::read(&checked)?))
    } else {
        None
    };
    if checked != path || current != before {
        return Err(CityError::new(
            "stale_file",
            "The file changed after the request. A fresh decision is required.",
        ));
    }
    files::atomic_write(&checked, content.as_bytes())?;
    rt.transact(|w| {
        let i = run_task(w, run_id)?;
        if w.tasks[i].state == TaskState::Waiting {
            w.tasks[i].state = TaskState::Running;
        }
        let version = w
            .results
            .iter()
            .filter(|r| {
                r.project_id == task.project_id
                    && r.path.as_deref() == Some(checked.to_str().unwrap())
            })
            .count() as u64
            + 1;
        w.results.push(Deliverable {
            id: id(),
            project_id: task.project_id.clone(),
            task_id: task.id.clone(),
            name: relative.into(),
            content: content.into(),
            path: Some(checked.display().to_string()),
            digest: files::digest(content.as_bytes()),
            version,
            state: "draft".into(),
        });
        Ok(())
    })?;
    Ok(json!({"written":relative,"bytes":content.len(),"digest":files::digest(content.as_bytes())}))
}
pub async fn demo_run(rt: Arc<Runtime>, run: &Run, cancel: CancellationToken) -> Result<String> {
    rt.acknowledge(&run.id)?;
    tokio::select! {_=cancel.cancelled()=>return Ok(String::new()),_=tokio::time::sleep(Duration::from_millis(800))=>{}}
    let w = rt.snapshot();
    let task = w.tasks.iter().find(|t| t.id == run.task_id).unwrap();
    if task.kind == "goal" {
        execute_tool(rt.clone(),&run.id,"delegate",json!({"title":"Inspect the sample brief","role":"Analyst","instructions":"Produce a short recommendation from the supplied demo brief.","criteria":"State a recommendation and its limitation."}),&cancel).await?;
        Ok("Demo plan delegated to an analyst. This is simulated work.".into())
    } else if task.kind == "review" {
        execute_tool(rt.clone(),&run.id,"review_result",json!({"accept":true,"evidence":"Demo review confirms the fixture includes a recommendation and limitation. No real model was called."}),&cancel).await?;
        Ok("Demo internal review complete.".into())
    } else {
        Ok("Demo result: start with a bounded local workflow and retain its evidence. Limitation: this is a deterministic simulation, not provider execution. Connect an installed local model for real work.".into())
    }
}

pub(crate) fn message_expired(message: &Message) -> bool {
    if message.acknowledged_at.is_some() {
        return false;
    }
    let deadline = message
        .expires_at
        .as_ref()
        .and_then(|at| chrono::DateTime::parse_from_rfc3339(at).ok())
        .or_else(|| {
            chrono::DateTime::parse_from_rfc3339(&message.at)
                .ok()
                .map(|at| at + chrono::Duration::hours(24))
        });
    deadline.is_some_and(|at| at < chrono::Utc::now())
}
fn route_attempt(world: &World, route: &[String], state: &str) -> Value {
    json!({"at":now(),"state":state,"worldRevision":world.revision,"roads":route.iter().filter_map(|id|world.roads.iter().find(|road|road.id==*id)).map(|road|json!({"id":road.id,"revision":road.revision})).collect::<Vec<_>>()})
}
pub(crate) fn retry_blocked_messages(world: &mut World) {
    for i in 0..world.messages.len() {
        let message = &world.messages[i];
        if message.state != "blocked" {
            continue;
        }
        if message_expired(message) {
            world.messages[i].state = "expired".into();
            continue;
        }
        let route = permitted_route(world, &message.from, &message.to);
        if route.is_empty() {
            continue;
        }
        let pending = world
            .messages
            .iter()
            .filter(|m| {
                m.to == message.to
                    && m.state == "delivered"
                    && m.acknowledged_at.is_none()
                    && !message_expired(m)
            })
            .count();
        let city = world
            .residents
            .iter()
            .find(|r| r.id == message.to)
            .map(|r| &r.city_id);
        let city_pending = world
            .messages
            .iter()
            .filter(|m| {
                m.state == "delivered"
                    && m.acknowledged_at.is_none()
                    && !message_expired(m)
                    && world
                        .residents
                        .iter()
                        .any(|r| r.id == m.to && Some(&r.city_id) == city)
            })
            .count();
        if pending >= 100 || city_pending >= 1000 {
            continue;
        }
        let attempt = route_attempt(world, &route, "delivered");
        world.messages[i].route = route;
        world.messages[i].state = "delivered".into();
        world.messages[i].route_attempts.push(attempt);
    }
}
