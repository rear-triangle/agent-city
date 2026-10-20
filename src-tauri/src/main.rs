mod window_state;
use city_core::{Command, Profile, Receipt, Runtime, World};
use std::{path::PathBuf, sync::Arc};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
type ApiResult<T> = Result<T, city_core::CityError>;
#[tauri::command]
fn redaction_preview(
    goal_id: String,
    runtime: tauri::State<'_, Arc<Runtime>>,
) -> ApiResult<serde_json::Value> {
    runtime.redaction_preview(&goal_id)
}
#[tauri::command]
fn deletion_preview(
    project_id: String,
    runtime: tauri::State<'_, Arc<Runtime>>,
) -> ApiResult<serde_json::Value> {
    runtime.deletion_preview(&project_id)
}
#[tauri::command]
fn cleanup_status(
    retry: bool,
    runtime: tauri::State<'_, Arc<Runtime>>,
) -> ApiResult<serde_json::Value> {
    runtime.cleanup_status(retry)
}
#[tauri::command]
fn snapshot(runtime: tauri::State<'_, Arc<Runtime>>) -> World {
    runtime.view_snapshot()
}
#[tauri::command]
fn submit(
    app: tauri::AppHandle,
    command: Command,
    runtime: tauri::State<'_, Arc<Runtime>>,
) -> ApiResult<Receipt> {
    let old = runtime.snapshot().settings.shortcut;
    let next = if let city_core::Action::Configure { settings } = &command.action {
        settings.shortcut.clone()
    } else {
        old.clone()
    };
    if next != old {
        app.global_shortcut()
            .register(next.as_str())
            .map_err(|e| city_core::CityError::new("shortcut_unavailable", e.to_string()))?;
    }
    let result = runtime.command(command);
    if next != old {
        if result.is_ok() {
            let _ = app.global_shortcut().unregister(old.as_str());
        } else {
            let _ = app.global_shortcut().unregister(next.as_str());
        }
    }
    result
}
#[tauri::command]
async fn profiles(runtime: tauri::State<'_, Arc<Runtime>>) -> ApiResult<Vec<Profile>> {
    runtime.refresh_profiles().await
}
#[tauri::command]
fn progress(
    goal_id: String,
    runtime: tauri::State<'_, Arc<Runtime>>,
) -> ApiResult<serde_json::Value> {
    runtime.progress(&goal_id)
}
#[tauri::command]
fn scan_skills(runtime: tauri::State<'_, Arc<Runtime>>) -> ApiResult<usize> {
    runtime.scan_skills()
}
#[tauri::command]
fn backup(path: String, runtime: tauri::State<'_, Arc<Runtime>>) -> ApiResult<()> {
    runtime.backup(std::path::Path::new(&path))
}
#[tauri::command]
fn home_edits(runtime: tauri::State<'_, Arc<Runtime>>) -> ApiResult<serde_json::Value> {
    runtime.home_edits()
}
#[tauri::command]
fn import_home_edit(
    resident_id: String,
    digest: String,
    base_revision: u64,
    runtime: tauri::State<'_, Arc<Runtime>>,
) -> ApiResult<()> {
    runtime.import_home_edit(&resident_id, &digest, base_revision)
}
#[tauri::command]
fn export_archive(path: String, runtime: tauri::State<'_, Arc<Runtime>>) -> ApiResult<()> {
    runtime.export_archive(std::path::Path::new(&path))
}
#[tauri::command]
fn inspect_archive(
    path: String,
    runtime: tauri::State<'_, Arc<Runtime>>,
) -> ApiResult<serde_json::Value> {
    runtime.inspect_archive(std::path::Path::new(&path))
}
#[tauri::command]
fn import_archive(
    path: String,
    workspaces: std::collections::HashMap<String, String>,
    runtime: tauri::State<'_, Arc<Runtime>>,
) -> ApiResult<()> {
    runtime.import_archive(std::path::Path::new(&path), &workspaces)
}
#[tauri::command]
fn result_content(result_id: String, runtime: tauri::State<'_, Arc<Runtime>>) -> ApiResult<String> {
    runtime
        .snapshot()
        .results
        .into_iter()
        .find(|r| r.id == result_id)
        .map(|r| r.content)
        .ok_or_else(|| city_core::CityError::new("not_found", "Result not found."))
}
#[tauri::command]
fn open_result(
    result_id: String,
    reveal: bool,
    runtime: tauri::State<'_, Arc<Runtime>>,
) -> ApiResult<()> {
    let world = runtime.snapshot();
    let result = world
        .results
        .iter()
        .find(|r| r.id == result_id)
        .ok_or_else(|| city_core::CityError::new("not_found", "Result not found."))?;
    let path = result.path.as_ref().ok_or_else(|| {
        city_core::CityError::new(
            "inline_result",
            "This result is available inside Agent City.",
        )
    })?;
    let project = world
        .projects
        .iter()
        .find(|p| p.id == result.project_id)
        .unwrap();
    let canonical = std::fs::canonicalize(path).map_err(|_| city_core::CityError::new("missing_artifact", "The external artifact is missing or inaccessible. Its recorded version remains in Results; it has not been regenerated."))?;
    let root = std::fs::canonicalize(&project.workspace)?;
    if !canonical.starts_with(root) {
        return Err(city_core::CityError::new(
            "denied_policy",
            "The result now points outside its approved project.",
        ));
    }
    let mut command = std::process::Command::new("/usr/bin/open");
    if reveal {
        command.arg("-R");
    }
    command.arg(canonical).spawn()?;
    Ok(())
}
fn command_mode(window: &tauri::WebviewWindow) -> Result<(), String> {
    window_state::mode(window, "command")
}
#[tauri::command]
fn window_mode(mode: String, window: tauri::WebviewWindow) -> Result<(), String> {
    window_state::mode(&window, &mode)
}
#[tauri::command]
fn quit(app: tauri::AppHandle, runtime: tauri::State<'_, Arc<Runtime>>) -> ApiResult<()> {
    if runtime
        .snapshot()
        .tasks
        .iter()
        .any(|t| t.state.executing() || t.state == city_core::TaskState::Waiting)
    {
        return Err(city_core::CityError::new(
            "active_work",
            "Pause and reconcile active work before quitting, or hide the city to keep it running.",
        ));
    }
    if let Some(window) = app.get_webview_window("main") {
        app.state::<window_state::Placement>().save(&window);
    }
    app.exit(0);
    Ok(())
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _, event| {
                    if event.state() == ShortcutState::Pressed {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = command_mode(&w);
                        }
                    }
                })
                .build(),
        )
        .setup(|app| {
            let demo = std::env::var("AGENT_CITY_DEMO").as_deref() == Ok("1");
            let base = std::env::var_os("AGENT_CITY_DATA")
                .map(PathBuf::from)
                .unwrap_or(app.path().app_data_dir()?);
            std::fs::create_dir_all(&base)?;
            app.manage(window_state::Placement::load(base.join("window.json")));
            if let Some(w) = app.get_webview_window("main") {
                window_state::initialize_native(&w)?;
                let _ = w.state::<window_state::Placement>().set_mode(&w, "command");
            }
            let runtime = Runtime::open(&base.join(if demo { "demo" } else { "live" }), demo)?;
            if !demo && runtime.snapshot().settings.skill_roots.is_empty() {
                let mut settings = runtime.snapshot().settings;
                if let Some(home) = std::env::var_os("HOME") {
                    settings.skill_roots = [".agents/skills", ".codex/skills"]
                        .iter()
                        .map(|p| PathBuf::from(&home).join(p))
                        .filter(|p| p.is_dir())
                        .map(|p| p.display().to_string())
                        .collect();
                    runtime.command(Command {
                        schema_version: 1,
                        id: city_core::id(),
                        expected_revision: None,
                        action: city_core::Action::Configure { settings },
                    })?;
                    let _ = runtime.scan_skills();
                }
            }
            let mut events = runtime.subscribe();
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                while events.recv().await.is_ok() {
                    let _ = handle.emit("city:revision", ());
                }
            });
            let runner = runtime.clone();
            tauri::async_runtime::spawn(async move {
                runner.start();
            });
            app.manage(runtime);
            let show = MenuItem::with_id(app, "show", "Open Agent City", true, None::<&str>)?;
            let ambient = MenuItem::with_id(app, "ambient", "Ambient overlay", true, None::<&str>)?;
            let hide = MenuItem::with_id(app, "hide", "Hide city", true, None::<&str>)?;
            let reset = MenuItem::with_id(app, "reset", "Reset position", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit Agent City…", true, None::<&str>)?;
            let pause = MenuItem::with_id(app, "pause", "Pause all cities", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &ambient, &hide, &reset, &pause, &quit])?;
            TrayIconBuilder::new()
                .title("AC")
                .tooltip("Agent City")
                .menu(&menu)
                .on_menu_event(|app, event| {
                    if let Some(w) = app.get_webview_window("main") {
                        match event.id().as_ref() {
                            "show" => {
                                let _ = command_mode(&w);
                            }
                            "ambient" => {
                                let _ = window_mode("ambient".into(), w);
                            }
                            "hide" => {
                                let _ = window_state::mode(&w, "hide");
                            }
                            "pause" => {
                                let rt = app.state::<Arc<Runtime>>();
                                for c in rt.snapshot().cities {
                                    let _ = rt.command(Command {
                                        schema_version: 1,
                                        id: city_core::id(),
                                        expected_revision: None,
                                        action: city_core::Action::Pause {
                                            city_id: Some(c.id),
                                            resident_id: None,
                                        },
                                    });
                                }
                            }
                            "reset" => {
                                let _ = command_mode(&w);
                                let _ = w.center();
                                window_state::clamp(&w);
                                app.state::<window_state::Placement>().save(&w);
                            }
                            "quit" => {
                                let _ = command_mode(&w);
                                let _ = w.emit("city:quit_requested", ());
                            }
                            _ => {}
                        }
                    }
                })
                .build(app)?;
            if let Err(e) = app.global_shortcut().register(
                app.state::<Arc<Runtime>>()
                    .snapshot()
                    .settings
                    .shortcut
                    .as_str(),
            ) {
                eprintln!("Shortcut unavailable; use menu bar: {e}");
            }
            let monitor_app = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    if let Some(w) = monitor_app.get_webview_window("main") {
                        window_state::clamp(&w);
                    }
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if let Some(w) = window.app_handle().get_webview_window("main") {
                    let _ = window_state::mode(&w, "hide");
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            snapshot,
            deletion_preview,
            redaction_preview,
            cleanup_status,
            submit,
            profiles,
            progress,
            scan_skills,
            backup,
            export_archive,
            home_edits,
            import_home_edit,
            inspect_archive,
            import_archive,
            open_result,
            result_content,
            window_mode,
            quit
        ])
        .build(tauri::generate_context!())
        .expect("Agent City could not start")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                let runtime = app.state::<Arc<Runtime>>();
                if runtime
                    .snapshot()
                    .tasks
                    .iter()
                    .any(|t| t.state.executing() || t.state == city_core::TaskState::Waiting)
                {
                    api.prevent_exit();
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = command_mode(&w);
                        let _ = w.emit("city:quit_requested", ());
                    }
                } else if let Some(w) = app.get_webview_window("main") {
                    app.state::<window_state::Placement>().save(&w);
                }
            }
        });
}
