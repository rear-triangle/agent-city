# Build status

Updated October 18, 2026. Version 0.2.0 is a working local development build for Apple Silicon. It is not yet a completed release under the [master specification](reference/MASTER-SPEC.md#55-definition-of-done).

The build contains actual managed execution, persistence, approvals, and a native shell. It can complete a bounded local workflow with the runtime's external networking blocked. Native overlay qualification, long-session testing, and some product requirements remain open.

The current bundle is approximately 13 MiB and has a verified local ad hoc signature. It is not Developer ID signed or notarized. The [package manifest](evidence/app-icon-2026-10-18/package.json) records the ZIP checksum, architecture, and checks.

## Milestones

| Milestone | Implemented | Evidence and remaining work |
| --- | --- | --- |
| M0 integration | Rust core, typed commands, SQLite, Tauri, installed Ollama and managed Codex adapters | Native build and real local/hosted file-tool proofs pass. Real Codex interruption, same-thread continuation, application approval, and process-disconnection checks pass. Native escalation and broader protocol-version coverage are outside the restricted profile. |
| M1 shared lifecycle | Durable command deduplication, scheduling, workspace leases, pause/continue, approvals, accepted memory, versioned skills, result review | 32 automated tests pass. Real offline delegation, approved write, review, owner acceptance, and recovery pass. Broader storage-failure and transport-race injection remains open. |
| M2 native overlay | Tray, configurable shortcut, command/ambient/hidden modes, saved bounds, display clamping, guarded quit | Native Command and Ambient windows were observed. The owner confirmed shortcut recovery. The owner confirmed menu recovery after restart, then reported that TextEdit could not receive typing during the overlay test. The focus fix passes native state checks, and the owner confirmed TextEdit typing works with the corrected overlay open. Display removal and sleep/wake remain unverified. |
| M3 city presentation | Cyberpunk and Solo Leveling architecture, persistent homes, real state projections, roads, couriers, compact inspector, keyboard-accessible DOM controls | Browser demo workflows, persistent accessibility preferences, graphics recovery, and twenty theme switches pass the checks recorded below. Final visual rubric, screen-reader audit, and native resource gates remain open. |
| M4 collaboration | Task-driven specialists and city mayors, review dependencies, scoped messaging, retirement/recall, manual mayor replacement, lineage-preserving mergers | Runtime tests cover nested city work, trusted routing, reviewed handoffs, and mergers. No real-provider multi-city end-to-end qualification yet. |
| M5 personal preview | Portable archive, database backup, workspace remapping, accepted-goal ledger, retention, archival/deletion, content redaction, local assets, packaging | Archive, command-retry preservation, recovery, retention, redaction, and project-deletion tests pass. New browser captures are recorded. Full acceptance coverage and the eight-hour trial remain open. |

## Verified behavior

- The automated suite passes 18 runtime tests, one provider protocol test, and 13 integration tests. Rust formatting, strict Clippy, TypeScript, and the production web build pass. See [verification output](evidence/runtime-2026-10-17/verification.txt).
- The offline fixture used Ollama 0.34.2 with installed `gemma4_e2b_q5km:latest` weights in the final regression. The scheduler selected Q5 under the local-only policy, despite a Q4 preference. An earlier workflow also passed on `gemma4_e2b_q4km:latest`. A local mayor delegated to an analyst. The analyst read a real brief, requested an exact write, wrote `recommendation.txt` after the fixture driver approved it, and proposed a useful note. The mayor recorded review acceptance, synthesized the correct recommendation, and the fixture driver accepted the goal. See [workflow output](evidence/runtime-2026-10-17/offline-workflow.txt).
- That runtime ran under a macOS sandbox profile denying outbound networking except loopback. An external connectivity probe failed inside the profile. A separate loopback Ollama process used existing weights. This proves the tested runtime workflow without external networking; it is not a whole-machine disconnected trial.
- Reopening the completed offline database retained the accepted goal, its four completed runs, exact tool receipts, accepted notes, one active acceptance entry, and the output file without starting a scheduler or provider. See [recovery output](evidence/runtime-2026-10-17/recovery.txt).
- Installed Codex 0.163.0 completed a real file read through the application's dynamic tool and returned the correct `CEDAR-417` evidence. The child process had native shell and independent subagent features disabled. See [hosted output](evidence/runtime-2026-10-17/hosted-read.txt). This is a narrow compatibility proof, not validation of every adapter control.
- The restricted Codex profile passed [interruption and continuation](evidence/runtime-2026-10-17/hosted-control.txt): two turns shared one thread, no file effect occurred before approval, and exactly one approved write completed after continuation.
- A separate [disconnection fixture](evidence/runtime-2026-10-17/hosted-disconnection.txt) killed only its own app-server while a write awaited approval. The task became Unknown, the approval was invalidated, and no file effect or automatic replacement occurred.
- Codex child processes disable configured external MCP servers, apps, plugins, shell execution, and independent subagents. Effective configuration is checked before the profile becomes available. Its provider sandbox is read-only with native escalation configured off. Agent City handles application file-write decisions in its own runtime. User configuration files and credentials are unchanged.
- Tests cover completed-detail retention without stripping unresolved context, explicit goal-content redaction, scope previews, deletion of resident projections while preserving project files, and preservation of command retry records across portable restore. Completed workflow archives round-trip successfully; broken record references are rejected even when the archive checksum is recomputed.
- A fifty-resident core fixture processed 500 rename commands with no inference. After adding retention, median command latency was 11.367 ms, p95 was 12.036 ms, and the view snapshot was 23,360 bytes. See [benchmark output](evidence/runtime-2026-10-17/core-benchmark.txt). This does not qualify fifty-resident rendering or burst ingestion.
- A ten-minute idle sample of the native host, WebContent, GPU, and networking processes averaged 0.083% CPU and 198.36 MiB summed RSS, with 254.69 MiB peak RSS. One logical core is 100%. Window visibility could not be verified. See [measurement summary](evidence/runtime-2026-10-17/native-idle.json). The sample used the preceding 0.2.0 development bundle, before the final input-provenance, privacy, provider, and messaging changes. It is not a measurement of every change in the current bundle.

## UI checks on October 18

- Native Command and compact Ambient windows were observed after the Mac was unlocked. The owner pressed Option+Shift+Space and confirmed that the full window returned. Targeted automation key delivery did not trigger the global shortcut.
- The owner confirmed that menu recovery works after a repeat following restart. The owner subsequently reported that TextEdit could not receive typing during the Ambient test. Native diagnostics reproduced retained active-app and key-window status. The macOS transition now relinquishes focus and restores the overlay after AppKit finishes hiding. Settled native flags and a direct screenshot pass. The owner confirmed TextEdit typing works with the corrected overlay open; direct clicks through city pixels remain unverified. See [focus regression evidence](evidence/ambient-focus-2026-10-18/README.md).
- Browser tests completed a delegated demo goal, review, and owner acceptance in list-only view. The scene frame count stayed at five throughout that workflow. Text size at 150%, high contrast, list-only view, and disabled announcements persisted after reload.
- Live refreshes and result acceptance retained panel focus. A separate native WebKit mouse-activation issue left focus in a hidden panel after Escape. The matching browser reproduction failed before explicit activation focus was added and passed afterward. The final native build also passed the Escape-to-Settings focus check.
- Forced WebGL context loss left resident and goal controls usable and displayed a graphics-paused notice. Context restoration removed the notice and resumed rendering.
- Twenty alternating theme changes retained stable per-theme geometry and texture counts in the two-resident browser demo. This does not qualify native memory growth or larger cities.
- [UI observations and browser captures](evidence/ui-2026-10-18/observations.json) record the scope of these checks. Browser screenshot text is blurred by the capture path; native text was readable in direct inspection.

## Acceptance coverage

The following entries map evidence to specification IDs. They do not mark an entire acceptance case passed when only part was exercised.

| Acceptance IDs | Available evidence |
| --- | --- |
| AT-03, 04, 07, 08, 09, 10, 13, 14, 39, 53, 65, 71 | Automated command, interruption-race, uncertain-restart, single-instance, and workspace-lease tests. The real hosted fixtures also cover AT-09/10 for the tested app-server version. This does not qualify every transport or provider. |
| AT-15, 16, 64, 68 | Acceptance ledger deduplication, reversal, restart, and transitive input-revision invalidation tests. Visual settlement stages are implemented but their final visual qualification is open. |
| AT-17 through 22, 63, 66, 78, 83 | Tests exercise trusted sender binding, duplicate coalescing, blocked routing, review handoffs, nested city work, and overlapping workspace leases. Not every route-change race or semantic duplicate assignment is covered. |
| AT-29, 30, 32, 34 | Memory ownership, external-note preservation, projection repair on reopen, and immutable active skill versions. Complete retrieval/conflict diagnostics remain broader than these tests. |
| AT-36, 38, 72, 73 | Exact file-write approval, changed digest rejection, denial propagation, and the offline fixture's real write. Hosted application-tool approvals pass the real control fixture. Native provider escalation is configured off in the shipped profile. |
| AT-51, 52 | Portable remapping, held import, lineage preservation, tamper detection, traversal rejection, and command-retry preservation without replay. Import into a fresh city preserves source identities; collision-free merging into an existing city is unsupported. |
| AT-57, 58, 76, 80, 81, 82 | Implemented leadership/lifecycle and resource/skill selection. Tests cover merger lineage, local-only policy, loaded-model admission, and archived skill versions; real lifecycle/model transition coverage is partial. |
| AT-43 through 50, 55, 56, 62, 67, 69, 70, 77 | UI and native behavior is implemented in varying scope. Earlier interactive checks are useful development evidence, but the final native, accessibility, privacy, visual, and long-session checks are pending. |

AT-01's seven-resident startup conflicts with the later owner-confirmed mayor-only requirement. This implementation follows AT-71 and section 2.6. AT-59 and AT-60 are explicitly future election work. All other cases without direct evidence remain open.

## Product work still open

- Qualify retention under a long mixed workload. Completed raw details expire by age and size, retained artifact copies have a size limit, and project deletion and goal redaction have explicit previews. Compact identities, summaries, receipts, and acceptance records intentionally persist and can grow. The application does not create an unbounded private diagnostic log file.
- Expand cross-city prerequisite mutation coverage. Runs now retain input revision maps, and a tested transitive invalidation reverses affected acceptance entries and marks downstream outputs stale.
- Qualify the road appearance controls visually. Each road now retains an automatic, fixed-road, or pinned-transit setting and progression rule version 1; presentation changes do not alter permissions.
- Extend stress coverage for message delivery. Messages now carry reply correlation, digests, route-attempt revisions, and 24-hour expiry. Limits are 100 unread messages per resident, 1,000 per city, twelve processing exchanges, and thirty messages per goal. Inbox insertion is synchronous and atomic; blocked routes retry when topology changes. There is no separate distributed inbox-claim service.
- Extend policy customization beyond the current workspace grants, road enable/disable controls, resident profiles, and shared project budgets. Skills support declared `requires_tools`, conservative helper/dependency checks, explicit version selection by mayors, and scope-aware automatic selection. This is not a general package installer or executable dependency resolver.
- Add full observed-agent mode if required. This build manages its own local/Codex runs and does not claim control of unrelated sessions.
- Complete physical connectivity-loss/reconnection, every cancellation mode, malformed-stream fault injection, and additional installed-provider versions. Known terminal failure preserves confirmed effects for a new provider; unknown outcomes retain the lease. Core tests cover both branches. The real hosted disconnect test covers one critical transport-loss point.
- Finish native mode tests, screen-reader and keyboard audits, the remaining missing-artifact UI checks, new screenshots, and the visual comparison requested by the specification.
- Measure each native presentation mode, twenty-resident traffic, fifty-resident aggregation, 100 events per second, repeated switches, and an eight-hour mixed session with sleep/wake.

Local execution intentionally has no arbitrary shell or web tool. It supports bounded file analysis and writing plus coordination and memory. A role title or downloaded skill cannot supply an absent capability. The application is not yet qualified as a general autonomous coding environment.

## Current verification limits

Native window automation and the T3 browser preview now work after unlocking the Mac and restarting the development preview. Native automation cannot access the AC status menu, and its targeted key delivery does not trigger the global shortcut. The owner confirmed both shortcut and menu recovery manually. The reported Ambient keyboard failure has a native fix with passing window-state checks. The owner confirmed keyboard input in TextEdit after the fix. Direct mouse click-through remains unverified; targeted automation delivers input directly to controls.

The packaged build and [validation procedure](VALIDATION.md) are available for the remaining local checks. No eight-hour trial or complete release acceptance is claimed.
