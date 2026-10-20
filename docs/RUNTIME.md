# Runtime reference

## State and authority

`crates/city-core` owns identities, projects, cities, tasks, runs, roads, inboxes, decisions, memory, skills, results, and acceptance history. `src-tauri` exposes typed native commands and window controls. `src/app` and `src/world` render projections. Scene movement never dispatches work or delivers messages.

A new city contains one mayor. Provisioning creates a specialist and its road only when a mayor delegates useful work. A project can have four cooperating cities. A city admits twelve active residents. A goal admits twelve worker tasks, thirty messages, and twenty provider runs by default. Provider-run and concurrency limits are configurable within bounded ranges.

Commands have stable IDs and payload digests. Repeating an ID returns the original receipt. Reusing an ID for different arguments fails. SQLite commits run intent before provider dispatch. Provider acknowledgement changes Starting to Running. Stale run events cannot complete a newer attempt.

The scheduler holds one active assignment per resident and serializes overlapping project folders. Unknown attempts retain their lease. Independent projects can use the remaining concurrency slots. The default limit is two active provider turns.

## Providers and admission

Ollama uses loopback HTTP and a bounded streaming tool loop. Advertised completion and tool capabilities are checked before admission. Missing completion markers, invalid output, repeated malformed tool arguments, and tool budgets produce explicit errors. Local review and synthesis use constrained structured responses. Review decisions pass through the actual `review_result` tool and require its confirmed receipt. Ordinary local work uses the bounded tool loop.

`AGENT_CITY_OLLAMA_URL` can select another HTTP loopback origin for an isolated test host. Remote origins, credentials, and URL paths are rejected. The app never downloads models.

Local admission uses measured macOS memory statistics and conservative estimates. Loading a model reserves an estimate based on its weight size and context overhead. Reusing an already loaded model reserves additional context capacity. This is a conservative admission rule, not a claim of optimal scheduling or exact inference memory usage. Queued work with resource blockers triggers periodic readiness checks.

Auto uses an explicitly selected hosted profile when permitted, or an available compatible local model. Local only excludes hosted inference. Known terminal provider failures can continue on another local profile with the same task identity and confirmed context. Unknown remote effects remain held.

The Codex adapter was developed against installed Codex 0.163.0 and its generated experimental app-server schema. It uses correlated threads and turns, dynamic application tools, provider approval responses, and confirmed turn interruption. Native shell execution, independent subagents, apps, plugins, and external MCP servers are disabled for the app's child process through command-line overrides. An empty MCP configuration table does not remove inherited entries, so the adapter reads effective configuration and disables each entry explicitly before creating a thread. It checks the resulting configuration. Thread execution uses a read-only provider sandbox with native escalation configured off. Application file writes still use Agent City's exact decisions. Existing user configuration files are unchanged. Unsupported server requests receive an error rather than inferred permission.

The tested version passed real thread continuation and process-disconnection fixtures. The adapter still needs broader protocol, malformed-stream, and connectivity-restoration coverage before a production support claim. It monitors provider events while an application tool waits for approval; transport loss invalidates the pending decision and holds the uncertain run. Missing turn identities produce Unknown outcomes, and delayed terminal events from another turn cannot finish the current run. Default app logs do not print provider payloads. Full private SQLite backups and session exports contain task context and must be handled as project data.

## Files and decisions

File tools operate inside the project's canonical directory. Relative traversal, symlink escape, and Git metadata access are rejected. Writes bind path, exact content, and the previous file digest to one decision. Changing the file after approval invalidates the operation. A denied operation cannot be rephrased or delegated into permission for the same goal.

A write that already matches the requested content returns an unchanged receipt without another effect. Intent and outcome are distinct records. A crash between a filesystem effect and its recorded outcome leaves an uncertain run for reconciliation.

## Collaboration and lifecycle

A worker completion produces a review task for its accountable coordinator. A peer mayor can delegate within its city, integrate reviewed work, and return it to the lead mayor. A reviewer that ends without a recorded decision leaves a visible blocker. Goal acceptance belongs to the Chief. Each run records the revisions of its dependency inputs. Changing a prerequisite marks its complete downstream chain stale, holds affected active work, invalidates pending decisions, and reverses affected goal acceptances. Existing outputs and their input revision records survive.

Messages carry trusted sender identity and a historical route. Direct permitted roads take precedence. Otherwise delivery uses the appropriate mayor route. Disabled or missing roads block delivery. Duplicate message bodies within the same task and purpose coalesce. An inbox rejects additional messages after one hundred unread deliveries, with a one-thousand-message city limit. Messages retain content digests, conversations, reply causation, and route attempts with road revisions. Unconsumed messages expire after twenty-four hours. Reopening a road cannot deliver expired work. Twelve processing exchanges and the goal message budget prevent unbounded conversations. Acknowledgement is a separate retained timestamp; repeated context preparation does not change the first acknowledgement.

The road inspector derives useful-collaboration progress from distinct acknowledged results of accepted tasks. Twelve qualifying exchanges establish the visual transit treatment. The owner can retain a fixed road style or pin a transit style. Each road retains rule version 1. These presentation choices change no permission. Settlement stages derive from accepted goals. The acceptance ledger retains reversals when the Chief reopens a result.

Retirement preserves history. Recall restores the same identity. A merger creates a new successor, keeps the source residents archived, and retains inherited memory as proposed notes. Manual mayor replacement requires reconciled execution, advances the leadership epoch, and leaves the former mayor as a resident. Agent elections are not implemented.

## Memory, skills, and recovery

Chief instructions are accepted memory. Agents can propose notes but cannot replace Chief instructions. Internal result review or owner acceptance can accept associated task notes. External `NOTES.md` changes are preserved and require explicit import.

The skill catalog reads configured local directories without following symlinks. It retains versioned `SKILL.md` content and bounded UTF-8 supporting files. Runs record their assigned skill IDs. Rescanning does not change an active run's references. Package text cannot grant tools. Packages with recognized unverified external dependencies remain unavailable. Packages can declare an inline `requires_tools` list in their front matter. Unknown tools, executable helpers, and recognized external dependencies make a package unavailable. Mayors can list the catalog and select version IDs for delegation or city founding. Automatic selection also considers the workstream. Delegated tasks retain their assigned versions, and runs retain the exact versions they consumed. Detection remains conservative, not a general package installer. Capability restrictions are rechecked on startup and import.

SQLite uses WAL and full synchronization. An instance lock prevents two schedulers using one directory. Startup invalidates unresolved decisions and holds attempts whose outcome was not recorded. A long system suspension also holds dispatch while interruption reconciles. These mechanisms do not promise exact process freezing.

Portable archives validate format, digest, identities, dependencies, and paths before import. Restore requires a fresh city and explicit workspace remapping. The default archive excludes raw provider payloads. Full database backups retain those payloads and the original folder paths.

## Retention, archival, and privacy

Settings defaults retain completed raw run details for seven days within a fifty-MiB detail budget and artifact copies within five hundred MiB. Maintenance runs on database commits and startup. Unresolved work keeps its context even when that exceeds the detail budget. A goal is eligible only after all its assignments are accepted or cancelled. Pruning keeps identities, outcomes, tool names, digests, compact task summaries, accepted memory, and acceptance history. Generated session projections are rewritten after detail expiry. External project files are unchanged. The event table has age, byte, and count limits. Compact records and command deduplication records remain longer and can grow.

Archiving a project holds its cities and preserves history. Restoring it keeps dispatch held until explicitly continued. Deletion requires an archived, reconciled project, another retained project, and a current scope digest. Its preview lists affected record counts. Database records and generated resident folders are removed; the project workspace, independent provider history, backups, and prior exports remain. Cleanup tombstones make an interrupted folder cleanup retryable.

Finished-goal redaction has its own digest-bound preview. It clears task text, raw run payloads, message text, associated notes, approval details, and retained result content while preserving identities, tool names, digests, outcomes, and acceptance history. Redacted goals cannot resume. Externally edited notes and exported copies are outside that redaction. These operations provide logical deletion, not secure-erasure guarantees.

Portable format 2 includes minimal command retry records in the checksum alongside the world. This detects corruption; it does not authenticate the archive's author. Import validates record relationships and commits the retry records and restored world in the same transaction. No command body is exported or executed. Legacy format 1 remains readable but has no command retry records. Full SQLite backup remains the complete local recovery format.

## UI and native shell

The UI receives snapshots without provider prompts, skill bodies, or raw tool payloads. Full deliverable content loads on demand. It has DOM controls for tasks, decisions, results, residents, and camera actions alongside the Three.js scene.

The renderer draws at most twenty individual homes. Larger fixture populations remain available through the roster. Normal motion is capped near thirty frames per second, and ambient motion near twelve. Resting scenes render on changes and coarse clock updates. Hidden and reduced-motion modes suppress scheduled animation.

The native shell supplies tray recovery, a configurable global shortcut, click-through ambient mode, saved placement, monitor clamping, and guarded quitting. Native focus, monitor removal, real sleep/wake, and full process memory measurements remain distinct validation requirements.

## Compatibility sources

- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)
- [Ollama chat API](https://docs.ollama.com/api/chat)
- [Codex app-server](https://learn.chatgpt.com/docs/app-server)

Installed generated protocol definitions were also used to verify Codex request and notification shapes. Those generated local files are development evidence under `.tools` and are not shipped.
