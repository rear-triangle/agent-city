# Validation procedures

Run these commands from the repository root. Use disposable directories for provider fixtures. The real-provider checks invoke inference; the ordinary test suite does not.

## Automated checks

```sh
./scripts/verify
npm run desktop:build
```

`scripts/verify` stops at the first failure and runs formatting, all core tests, strict workspace Clippy, TypeScript, and Vite. The Vite build currently warns about the approximately 574 kB Three.js chunk. It is packaged locally and is not a network dependency.

## Real local workflow

Start your installed Ollama host with an existing completion model that supports tools. Do not point this fixture at a real project. It creates its own brief and permits only a bounded fixture write to `recommendation.txt` containing the verification code.

```sh
./scripts/cargo build -p city-core --example offline_proof --example recovery_proof
CITY_PROOF_MODEL=gemma4_e2b_q4km:latest \
  ./target/debug/examples/offline_proof .city-dev/offline-check-new
./target/debug/examples/recovery_proof .city-dev/offline-check-new
```

For a network-restricted runtime check on the tested Mac:

```sh
CITY_PROOF_MODEL=gemma4_e2b_q4km:latest \
  /usr/bin/sandbox-exec \
  -p '(version 1)(allow default)(deny network-outbound)(allow network-outbound (remote ip "localhost:*"))' \
  ./target/debug/examples/offline_proof .city-dev/offline-restricted-new
```

Verify the sandbox separately with an external connection probe. The successful recorded test used a separate Ollama host at `127.0.0.1:11435` through `AGENT_CITY_OLLAMA_URL`, with one loaded model and one parallel request. That test host was stopped afterward. The user's existing host on port 11434 was unchanged. The sandbox restricts the runtime process, not an independently running model server or the whole Mac. A physical-disconnection startup/reconnection trial remains required.

The fixture requires a mayor, a specialist, genuine file tools, accepted memory, a reviewed result, and the expected code. It records owner acceptance only after checking those observations. `recovery_proof` reopens that completed database without starting execution.

## Hosted compatibility check

With the installed Codex CLI already authenticated, select an available `codex:` profile. The following example uses the tested profile spelling; use a discovered supported profile if your installation differs.

```sh
CITY_PROOF_MODEL=codex:gpt-6.1-sol \
  ./scripts/cargo run -p city-core --example local_proof -- .city-dev/hosted-check-new
```

Inspect the log and stored run provider. A successful fallback to Ollama is useful recovery evidence but does not count as hosted compatibility. This small fixture proves a correlated file tool and result.

The control fixture pauses while an application file write awaits approval, checks the terminal acknowledgement and absence of a file effect, then continues on the same provider thread. It approves only the fixture's exact bounded write and checks its result before owner acceptance.

```sh
CITY_PROOF_MODEL=codex:gpt-6.1-sol \
  ./scripts/cargo run -p city-core --example hosted_control_proof -- .city-dev/hosted-control-new
```

The recorded disconnection check uses the same example with `CITY_PROOF_DISCONNECT_PID` set to a PID file written by a test-only Codex wrapper. Point `AGENT_CITY_CODEX` to that wrapper. The wrapper writes its own PID, then uses `exec` to launch the installed Codex binary with unchanged arguments. The fixture verifies that the PID belongs to its own direct child before terminating it at the pending write decision. It requires an Unknown task, an invalidated approval, one retained attempt, and no file effect. Never supply the PID of an unrelated Codex process.

The restricted profile disables native shell, independent subagents, apps, plugins, and configured external MCP servers in the child process. It verifies the effective configuration and uses a read-only provider sandbox with native escalation off. Application file-write approval is separate from native provider escalation. The real control and disconnection fixtures passed on Codex 0.163.0; additional protocol versions and connectivity-loss points remain open.

## Retention and portable data

The automated suite covers these cases without inference:

- Expired completed-run payloads disappear from both SQLite state and generated session files. Unresolved runs retain their context.
- Goal redaction clears its content and associated detail while preserving acceptance accounting and retry metadata.
- Project deletion requires a current scope digest, removes its resident projections, and preserves files in the external project workspace.
- Archive restore remaps workspaces, holds dispatch, clears provider sessions, and preserves command retry records without replaying effects.
- Archive validation rejects broken identities and relationships even when a modified file has a matching checksum.

Logical deletion does not securely erase prior disk blocks, external files, provider history, exports, or backups. Inspect the UI's scope preview before exercising deletion on real data. Use disposable projects for validation.

## Native checks

Launch the built app with a separate data base:

```sh
AGENT_CITY_DATA="$PWD/.city-dev/native-validation" \
  "target/release/bundle/macos/Agent City.app/Contents/MacOS/agent-city"
```

1. Create a disposable project, choose an installed local model, and request an output based on an actual file. Inspect and resolve the exact write request. Review the result, accept the goal, quit, and reopen. Confirm the same identities, note, artifact, and one acceptance entry.
2. While typing in another application, enter Ambient mode. Check focus retention and click-through. Recover Command mode through both the menu-bar menu and the configured shortcut. Repeat after hiding and closing the window.
3. Move the window to a second display and disconnect that display. Confirm it returns to visible bounds. Repeat with changed scaling and both window modes.
4. Sleep during active work and during a pending approval. Wake and verify that dispatch waits for reconciliation. Confirm no duplicate write and no stale approval becomes accepted.
5. Exercise quiet mode, reduced motion, all lighting presets, both themes, and compact and expanded sizes. Confirm state and identities persist. Verify distinct reading, writing, delivery, waiting, and home-return behavior.
6. Complete the same inspection and approval flow using keyboard navigation and a screen reader. Test graphics context loss while retaining operable DOM controls. In Settings → Accessibility, test 150% text, high contrast, and list-only view. Reload and verify that preferences persist. Close each panel with Escape and confirm that focus returns to its opener. Repeat after a live update and accepting a result.
7. Export a portable archive. Restore into a fresh city, remap every project, and confirm held dispatch, preserved lineage, and no automatic execution. Check a missing external artifact explicitly.

Record the application version, OS, display scale, selected mode, provider versions, steps, observed results, and screenshots. Existing images in `docs/evidence` mostly document the preserved prototype; they are not final-runtime acceptance evidence.

## Performance and long sessions

Identify the exact app host and its associated WebContent, GPU, and networking processes by launch time. Do not include another application's WebKit processes or omit this application's children. Use Activity Monitor's energy and physical-memory measurements alongside the sampler where needed.

```sh
python3 scripts/measure-processes.py \
  --pids APP_PID WEB_CONTENT_PID GPU_PID NETWORK_PID \
  --seconds 600 --label quiet-visible \
  --output .city-dev/quiet-visible.json
```

Replace the PID placeholders with numbers. The sampler sums process RSS, which includes shared pages, and computes CPU deltas with one logical core equal to 100%. It fails if a tracked process exits. Independently verify visibility and mode; the label alone is not evidence. Separate inference process usage from application overhead.

Repeat the master specification's workloads and budgets. The eight-hour mixed trial must include work, idle periods, theme and window changes, provider loss, and sleep/wake. A ten-minute idle sample cannot substitute for it.

The core-only fixture is:

```sh
./scripts/cargo run -p city-core --example core_benchmark -- .city-dev/benchmark-new
```

It measures fifty resident records and 500 durable commands with no inference. It does not exercise WebGL, real traffic, native focus, or 100 events per second.
