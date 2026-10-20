//! Development-only demo bridge. Never linked into the desktop application.
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::post,
    Json, Router,
};
use city_core::*;
use serde_json::{json, Value};
use std::sync::Arc;
#[derive(Clone)]
struct App {
    runtime: Arc<Runtime>,
    token: String,
}
async fn dispatch(
    State(app): State<App>,
    Path(method): Path<String>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if headers.get("x-city-token").and_then(|v| v.to_str().ok()) != Some(app.token.as_str()) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"message":"This development bridge requires its private token."})),
        );
    }
    let result: Result<Value> = match method.as_str() {
        "snapshot" => Ok(json!(app.runtime.view_snapshot())),
        "submit" => serde_json::from_value::<Command>(body["command"].clone())
            .map_err(CityError::from)
            .and_then(|c| app.runtime.command(c))
            .map(|r| json!(r)),
        "result_content" => app
            .runtime
            .snapshot()
            .results
            .into_iter()
            .find(|r| Some(r.id.as_str()) == body["resultId"].as_str())
            .map(|r| json!(r.content))
            .ok_or_else(|| CityError::new("not_found", "Result not found.")),
        "profiles" => Ok(json!([])),
        "progress" => app
            .runtime
            .progress(body["goalId"].as_str().unwrap_or_default()),
        _ => Err(CityError::new(
            "native_only",
            "This action requires the native app.",
        )),
    };
    match result {
        Ok(v) => (StatusCode::OK, Json(v)),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!(e))),
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    let token = std::env::var("CITY_PREVIEW_TOKEN")
        .expect("Run scripts/preview-runtime to supply a private token");
    assert!(token.len() >= 32);
    let rt = Runtime::open(std::path::Path::new(".city-dev/browser-demo"), true)?;
    rt.start();
    let app = Router::new()
        .route("/city-api/{method}", post(dispatch))
        .with_state(App { runtime: rt, token });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:4783").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
