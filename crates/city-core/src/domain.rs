use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
pub fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct World {
    pub schema_version: u32,
    pub revision: u64,
    pub demo: bool,
    pub projects: Vec<Project>,
    pub cities: Vec<City>,
    pub residents: Vec<Resident>,
    pub tasks: Vec<Task>,
    pub runs: Vec<Run>,
    pub roads: Vec<Road>,
    pub messages: Vec<Message>,
    pub decisions: Vec<Decision>,
    pub memories: Vec<Memory>,
    pub skills: Vec<Skill>,
    pub results: Vec<Deliverable>,
    pub events: Vec<Event>,
    pub settings: Settings,
    #[serde(default)]
    pub acceptance_ledger: Vec<Acceptance>,
    #[serde(default)]
    pub deleted_homes: Vec<String>,
}
impl World {
    pub fn empty(demo: bool) -> Self {
        Self {
            schema_version: 1,
            revision: 0,
            demo,
            projects: vec![],
            cities: vec![],
            residents: vec![],
            tasks: vec![],
            runs: vec![],
            roads: vec![],
            messages: vec![],
            decisions: vec![],
            memories: vec![],
            skills: vec![],
            results: vec![],
            events: vec![],
            settings: Settings::default(),
            acceptance_ledger: vec![],
            deleted_homes: vec![],
        }
    }
    pub fn event(&mut self, kind: &str, subject: &str, summary: impl Into<String>) {
        self.events.push(Event {
            id: id(),
            at: now(),
            kind: kind.into(),
            subject: subject.into(),
            summary: summary.into(),
        });
        if self.events.len() > 300 {
            self.events.drain(..self.events.len() - 300);
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Acceptance {
    pub id: String,
    pub goal_id: String,
    pub task_revision: u64,
    pub accepted_at: String,
    pub reversed_at: Option<String>,
}
fn default_history_days() -> u32 {
    7
}
fn default_history_mib() -> u32 {
    50
}
fn default_artifact_mib() -> u32 {
    500
}
fn default_true() -> bool {
    true
}
fn default_shortcut() -> String {
    "Alt+Shift+Space".into()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme: String,
    pub policy: String,
    pub preferred_model: Option<String>,
    pub concurrency: usize,
    pub max_turns: usize,
    pub max_tool_steps: usize,
    pub skill_roots: Vec<String>,
    pub quiet: bool,
    pub clock_mode: String,
    pub lighting: String,
    #[serde(default = "default_shortcut")]
    pub shortcut: String,
    #[serde(default = "default_history_days")]
    pub history_days: u32,
    #[serde(default = "default_history_mib")]
    pub history_mib: u32,
    #[serde(default = "default_artifact_mib")]
    pub artifact_mib: u32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "cyber".into(),
            policy: "auto".into(),
            preferred_model: None,
            concurrency: 2,
            max_turns: 20,
            max_tool_steps: 16,
            skill_roots: vec![],
            quiet: false,
            clock_mode: "local".into(),
            lighting: "day".into(),
            shortcut: default_shortcut(),
            history_days: default_history_days(),
            history_mib: default_history_mib(),
            artifact_mib: default_artifact_mib(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub purpose: String,
    pub workspace: String,
    pub lead_mayor: String,
    #[serde(default)]
    pub archived: bool,
    pub revision: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct City {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub purpose: String,
    pub mayor_id: String,
    pub epoch: u64,
    pub held: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resident {
    pub id: String,
    pub city_id: String,
    pub name: String,
    pub role: String,
    pub lifecycle: String,
    pub held: bool,
    pub skills: Vec<String>,
    pub ancestors: Vec<String>,
    pub successor: Option<String>,
    pub model: Option<String>,
    pub revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Queued,
    Starting,
    Running,
    Waiting,
    Pausing,
    Paused,
    Unknown,
    Blocked,
    Review,
    Accepted,
    Failed,
    Cancelled,
}
impl TaskState {
    pub fn executing(&self) -> bool {
        matches!(
            self,
            Self::Starting | Self::Running | Self::Pausing | Self::Unknown
        )
    }
    pub fn finished(&self) -> bool {
        matches!(self, Self::Accepted | Self::Cancelled)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub city_id: String,
    pub resident_id: String,
    pub goal_id: String,
    pub parent_id: Option<String>,
    pub kind: String,
    pub title: String,
    pub instructions: String,
    pub criteria: String,
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub assigned_skills: Option<Vec<String>>,
    pub state: TaskState,
    pub revision: u64,
    pub run_id: Option<String>,
    pub checkpoint: String,
    pub blocker: Option<String>,
    pub output: String,
    pub created_at: String,
    pub accepted_at: Option<String>,
    pub stale: bool,
    #[serde(default)]
    pub redacted: bool,
    pub cancel_requested: bool,
    pub pending_direction: Option<String>,
    pub reviewed_task: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    pub id: String,
    pub task_id: String,
    pub resident_id: String,
    pub provider: String,
    pub model: String,
    pub state: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub context: String,
    pub messages: Vec<Value>,
    pub steps: usize,
    pub tool_receipts: Vec<ToolReceipt>,
    pub epoch: u64,
    #[serde(default)]
    pub provider_thread: Option<String>,
    #[serde(default)]
    pub provider_turn: Option<String>,
    #[serde(default)]
    pub skill_ids: Vec<String>,
    #[serde(default)]
    pub failure_code: Option<String>,
    #[serde(default)]
    pub input_revisions: std::collections::BTreeMap<String, u64>,
    #[serde(default = "default_true")]
    pub payloads_retained: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolReceipt {
    pub id: String,
    pub name: String,
    pub arguments: Value,
    pub state: String,
    pub result: Value,
}
fn default_road_style() -> String {
    "auto".into()
}
fn road_rule_version() -> u32 {
    1
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Road {
    pub id: String,
    pub from: String,
    pub to: String,
    pub enabled: bool,
    pub revision: u64,
    #[serde(default = "default_road_style")]
    pub presentation: String,
    #[serde(default = "road_rule_version")]
    pub rule_version: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    #[serde(default)]
    pub goal_id: String,
    #[serde(default)]
    pub city_id: String,
    #[serde(default)]
    pub digest: String,
    #[serde(default)]
    pub conversation_id: String,
    #[serde(default)]
    pub causation_id: Option<String>,
    #[serde(default)]
    pub causal_depth: u32,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub route_attempts: Vec<serde_json::Value>,
    pub id: String,
    pub from: String,
    pub to: String,
    pub task_id: String,
    pub purpose: String,
    pub body: String,
    pub route: Vec<String>,
    pub state: String,
    pub at: String,
    #[serde(default)]
    pub acknowledged_at: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Decision {
    pub id: String,
    pub task_id: String,
    pub run_id: String,
    pub resident_id: String,
    pub action: String,
    pub arguments: Value,
    pub digest: String,
    pub reason: String,
    pub scope: String,
    pub state: String,
    pub revision: u64,
    pub epoch: u64,
    pub expires_at: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Memory {
    pub id: String,
    pub resident_id: String,
    pub kind: String,
    pub content: String,
    pub source: String,
    pub revision: u64,
    pub accepted: bool,
    pub created_at: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub source: String,
    pub digest: String,
    pub content: String,
    pub files: Vec<(String, String)>,
    pub available: bool,
    pub limitation: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deliverable {
    pub id: String,
    pub project_id: String,
    pub task_id: String,
    pub name: String,
    pub content: String,
    pub path: Option<String>,
    pub digest: String,
    pub version: u64,
    pub state: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: String,
    pub at: String,
    pub kind: String,
    pub subject: String,
    pub summary: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub schema_version: u32,
    pub id: String,
    pub expected_revision: Option<u64>,
    #[serde(flatten)]
    pub action: Action,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    ArchiveProject {
        project_id: String,
        archived: bool,
    },
    RedactGoal {
        goal_id: String,
        digest: String,
    },
    DeleteProject {
        project_id: String,
        digest: String,
    },
    CreateProject {
        name: String,
        purpose: String,
        workspace: String,
    },
    Assign {
        project_id: String,
        city_id: String,
        resident_id: Option<String>,
        title: String,
        criteria: String,
    },
    Pause {
        resident_id: Option<String>,
        city_id: Option<String>,
    },
    ContinueCity {
        city_id: String,
    },
    Resume {
        task_id: String,
    },
    Cancel {
        task_id: String,
    },
    Steer {
        task_id: String,
        direction: String,
    },
    Accept {
        task_id: String,
    },
    Reopen {
        task_id: String,
        direction: String,
    },
    Decide {
        decision_id: String,
        allow: bool,
        digest: String,
    },
    SaveMemory {
        resident_id: String,
        content: String,
        base_revision: u64,
    },
    AcceptMemory {
        memory_id: String,
        accept: bool,
    },
    ReviseMemory {
        memory_id: String,
        content: String,
        base_revision: u64,
    },
    Retire {
        resident_id: String,
    },
    Recall {
        resident_id: String,
    },
    Rename {
        resident_id: String,
        name: String,
    },
    SetModel {
        resident_id: String,
        model: Option<String>,
    },
    ReplaceMayor {
        city_id: String,
        resident_id: String,
    },
    Merge {
        resident_ids: Vec<String>,
        name: String,
        role: String,
    },
    SetRoad {
        road_id: String,
        enabled: bool,
    },
    SetRoadPresentation {
        road_id: String,
        presentation: String,
    },
    Configure {
        settings: Settings,
    },
    Reconcile {
        task_id: String,
        confirmed_stopped: bool,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
    pub command_id: String,
    pub revision: u64,
    pub target_id: Option<String>,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub model: String,
    pub provider: String,
    pub available: bool,
    pub tools: bool,
    pub size: u64,
    pub context_length: u64,
    #[serde(default)]
    pub loaded: bool,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandRecord {
    pub id: String,
    pub digest: String,
    pub receipt: Receipt,
}
