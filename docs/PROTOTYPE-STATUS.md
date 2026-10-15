# Prototype status

The user requested a prototype for approval before the full build. This branch answers which layout, visual treatment, and interaction direction to carry forward. No production milestone is complete. Owner approval is pending.

## Scope

The prototype is TypeScript, DOM controls, CSS, and a Canvas 2D settlement. It has no runtime library dependencies. Vite 8.3.4 and TypeScript 7.0.2 are pinned development dependencies. Both fonts are local assets.

Three layouts share the same in-memory state. Cyberpunk and Shadow domain have different architecture, resident presentation, couriers, ground treatment, and interface framing. Each includes three authored appearance stages. The scene caches its static artwork. Couriers run for a finite trace at a 24 fps cap in expanded views and a 12 fps cap in the compact view. Quiet mode has no animation loop. A visibility listener stops scheduled rendering when the document is hidden.

This browser prototype does not implement a Rust core, Tauri shell, SQLite, a provider adapter, durable routing, persistent memory, OS window controls, or native diagnostics. It never launches inference or changes project files through its controls. Source edits and dependency installation are development work performed by the implementation agent.

## Milestones

| Milestone | Status |
| --- | --- |
| Design prototype | Implemented. Awaiting owner approval. |
| M0 environment and provider proof | Deferred until prototype approval. Node verified; provider and native toolchains not probed. |
| M1 domain core and durable state | Not started. The demo state is disposable. |
| M2 communication and memory | UI concepts only. No durable broker or context preparation. |
| M3 visual slices | Two 2.5D art directions available for review. Native budgets and 3D comparison pending. |
| M4 native workflow | Not started. Browser compact composition only. |
| M5 personal preview | Not started. Native packaging, durability, accessibility audit, and resource trials pending. |
| M6 and M7 | Deferred as specified. |

## Verification record

The 12 supplied references were inspected in a contact sheet generated with Pillow through `uv run --with pillow python`.

Checks were performed on the development preview through T3 Code's collaborative browser at 1280 × 800 and at a 390 × 844 CSS viewport. These observations describe the browser prototype only.

| Check | Observed result |
| --- | --- |
| Open all three layouts | A, B, and C render and update the URL parameter. |
| Switch visual themes | Both themes render distinct buildings and panels. |
| Allow a permission, review, and accept | `approval=allowed`, `phase=accepted`, `accepted=3`, and `stage=1`. |
| Queue a task directly for Vector | One `TASK-004` record appears with the chosen title, resident `analyst`, and status `Queued`. |
| Save a memory note, then change theme | The added note, seven resident IDs, queued task, accepted count, and appearance stage remain unchanged. |
| Pause and continue town | Active and waiting residents first show Pausing, then Paused. Continue restores the two sample working residents. |
| Quiet mode | Renderer reports `scheduledFrames=false`. This is browser instrumentation, not a CPU measurement. |
| Phone layout | Document width equals viewport width. Header actions and footer task controls remain inside the viewport. The roster scrolls horizontally. |
| Page errors during completed checks | No Vite error overlay or application console errors observed. |
| Additional denial-flow check | T3 browser automation host disconnected before the check completed. Deny behavior remains unverified in the browser. |
| Final focus-preservation adjustment | Typechecked and built after the browser disconnect. Browser recheck pending. |

The preview host reported no available automation host during the final optional check. Its instruction was not to retry. No alternate browser system was used.

### Commands

Commands ran from the nested Git repository at `/Users/alvarobalkowski/Documents/agent-city/agent-city` unless stated otherwise.

| Command | Outcome |
| --- | --- |
| `unzip -l /Users/alvarobalkowski/.t3/userdata/attachments/9a463261-9248-4816-b59b-77928e0b85e5-548eca6e-88e7-4d62-b6e0-cd3a2731043c-zip.zip` | Listed the master spec, prompts, technical notes, and 12 images. |
| `unzip -q /Users/alvarobalkowski/.t3/userdata/attachments/9a463261-9248-4816-b59b-77928e0b85e5-548eca6e-88e7-4d62-b6e0-cd3a2731043c-zip.zip -d /tmp/agent-town-handoff` | Extracted the supplied reference material. |
| `git checkout -b prototype/agent-town` | Created an isolated prototype branch in the initially empty repository. |
| `npm install --save-dev --save-exact vite typescript` | Initial attempt failed because the environment selected an obsolete Intel Node. |
| `PATH=/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin npm install --save-dev --save-exact vite typescript` | Installed 17 packages. Audit reported zero vulnerabilities. |
| `PATH=/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin npm run prototype` | Started Vite on port 5173. |
| `./prototype --help` | Verified the one-command native Node launcher. |
| `PATH=/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin npm run build` | TypeScript check and production build pass after final code edits. |

The final production build contains about 63 kB of JavaScript and 52 kB of CSS before compression, plus 59 kB of already-compressed WOFF2 fonts. JavaScript, CSS, HTML, favicon, and fonts total approximately 94 kB with text compression. These are asset sizes, not application memory or CPU measurements.

## Acceptance mapping

Every full-release acceptance gate remains pending. “UI simulation” below means the behavior can be reviewed in this prototype, not that the production requirement passes.

| ID | Scenario | Prototype evidence or pending work |
| --- | --- | --- |
| AT-01 | Create the starter town | Seven residents, homes, and six mayor routes are displayed. No persistence. |
| AT-02 | Rename a resident and switch its model while idle | Pending full build. No release evidence in this prototype. |
| AT-03 | Assign a goal with valid scope | Assignment queues an in-memory task. No durable command or dispatch. |
| AT-04 | Repeat the same assignment command | Pending full build. No release evidence in this prototype. |
| AT-05 | Submit a plan with a dependency cycle | Pending full build. No release evidence in this prototype. |
| AT-06 | Reach the concurrency limit | Pending full build. No release evidence in this prototype. |
| AT-07 | Pause during an active turn | Pause displays Pausing and then a simulated acknowledgement. |
| AT-08 | Finish naturally while pause is pending | Pending full build. No release evidence in this prototype. |
| AT-09 | Lose interruption acknowledgement | Pending full build. No release evidence in this prototype. |
| AT-10 | Resume an interrupted task | Continue restores the previous demo activity. No provider context. |
| AT-11 | Pause a parent with an active child | Pending full build. No release evidence in this prototype. |
| AT-12 | Pause the town | Town pause holds the simulated active and waiting residents. |
| AT-13 | Deliver a late completion for an old run | Pending full build. No release evidence in this prototype. |
| AT-14 | Provider turn ends successfully | The sample result requires separate owner acceptance. |
| AT-15 | Accept a goal twice, replay, and restart | Repeated acceptance clicks are ignored after the phase becomes accepted. Restart behavior is unimplemented. |
| AT-16 | Reopen an accepted goal | Pending full build. No release evidence in this prototype. |
| AT-17 | Send through a valid direct road | Direct roads and sample exchanges can be inspected. No broker enforcement. |
| AT-18 | Send without a direct road but valid mayor roads | Relay policy is explained in the route inspector. Routing is unimplemented. |
| AT-19 | Send without any permitted route | Pending full build. No release evidence in this prototype. |
| AT-20 | Spoof a sender in tool arguments | Pending full build. No release evidence in this prototype. |
| AT-21 | Disable a road while messages are queued | Pending full build. No release evidence in this prototype. |
| AT-22 | Duplicate message delivery | Pending full build. No release evidence in this prototype. |
| AT-23 | Crash after recipient provider submission | Pending full build. No release evidence in this prototype. |
| AT-24 | Fill a mailbox | Pending full build. No release evidence in this prototype. |
| AT-25 | Create a ping-pong exchange | Pending full build. No release evidence in this prototype. |
| AT-26 | Send twelve empty acknowledgements | Pending full build. No release evidence in this prototype. |
| AT-27 | Meet the useful-collaboration rule | Pending full build. No release evidence in this prototype. |
| AT-28 | Stop or hide the renderer during delivery | Courier rendering is separate from demo state changes. No real delivery. |
| AT-29 | Restart after a committed memory update before file replacement | Pending full build. No release evidence in this prototype. |
| AT-30 | Externally edit notes based on an old revision | Pending full build. No release evidence in this prototype. |
| AT-31 | Run a new task after a useful note was accepted | Pending full build. No release evidence in this prototype. |
| AT-32 | A model summary contradicts an owner instruction | Pending full build. No release evidence in this prototype. |
| AT-33 | A retrieved document asks for wider permissions | Pending full build. No release evidence in this prototype. |
| AT-34 | Update a skill during an active run | Pending full build. No release evidence in this prototype. |
| AT-35 | Decide a real provider approval | Allow once resolves a simulated request and advances the demo result. |
| AT-36 | Approve a stale or altered request | Pending full build. No release evidence in this prototype. |
| AT-37 | Provider cleans up an unresolved approval | Pending full build. No release evidence in this prototype. |
| AT-38 | Deny one specialist's request | Deny branch is implemented. Its final browser check was interrupted by a disconnected preview host. |
| AT-39 | Crash after sending a launch before recording its response | Pending full build. No release evidence in this prototype. |
| AT-40 | Storage becomes full or read-only | Pending full build. No release evidence in this prototype. |
| AT-41 | Frontend misses a snapshot patch | Pending full build. No release evidence in this prototype. |
| AT-42 | Provider emits an unsupported event | Pending full build. No release evidence in this prototype. |
| AT-43 | Enter Ambient mode while typing elsewhere | Pending full build. No release evidence in this prototype. |
| AT-44 | Use the shortcut and menu-bar recovery | Pending full build. No release evidence in this prototype. |
| AT-45 | Remove the display containing the overlay | Pending full build. No release evidence in this prototype. |
| AT-46 | Sleep and wake during active work | Pending full build. No release evidence in this prototype. |
| AT-47 | Lose WebGL context | Pending full build. No release evidence in this prototype. |
| AT-48 | Enable reduced motion | Quiet mode removes traveling couriers and schedules no animation frames. OS preference is read on startup. |
| AT-49 | Change the day/night preset | Day, dusk, and night change presentation only. |
| AT-50 | Change themes repeatedly | Switching themes retains in-memory resident identity, tasks, notes, approvals, and progression. |
| AT-51 | Export and import into a clean installation | Pending full build. No release evidence in this prototype. |
| AT-52 | Import an archive with path traversal or invalid digest | Pending full build. No release evidence in this prototype. |
| AT-53 | Start a second app instance on the same data directory | Pending full build. No release evidence in this prototype. |
| AT-54 | Switch to an observed external agent | Pending full build. No release evidence in this prototype. |
| AT-55 | Seed test secrets in provider payloads | Pending full build. No release evidence in this prototype. |
| AT-56 | Inspect a house using only keyboard and screen reader | DOM navigation, roster, form labels, tabs, Escape, and dialog focus trap are implemented. Full assistive-technology validation is pending. |
| AT-57 | Replace the mayor manually | Pending full build. No release evidence in this prototype. |
| AT-58 | Merge two residents later | Pending full build. No release evidence in this prototype. |
| AT-59 | Run a seven-resident election later | Pending full build. No release evidence in this prototype. |
| AT-60 | Spawn a resident during an open election | Pending full build. No release evidence in this prototype. |
| AT-61 | Send across towns later | Pending full build. No release evidence in this prototype. |
| AT-62 | Leave the town idle for an hour | Courier animation stops after the finite demo trace. No model calls exist. Native idle profiling is pending. |
| AT-63 | Complete an internal specialist task | Pending full build. No release evidence in this prototype. |
| AT-64 | Revise an accepted prerequisite | Pending full build. No release evidence in this prototype. |
| AT-65 | Queue two mutating runs against one checkout | Pending full build. No release evidence in this prototype. |
| AT-66 | Mayor delegates and waits for results | Pending full build. No release evidence in this prototype. |
| AT-67 | Switch between Cyberpunk and Solo Leveling | The two procedural art kits and panels change together over shared demo state. |
| AT-68 | Advance and reopen qualifying accepted goals | Accepting the sample outcome changes stage 0 to stage 1. All three appearance stages are inspectable. Persistence is unimplemented. |
| AT-69 | Inspect Solo Leveling controls at compact and expanded sizes | Blue System framing is present in expanded and compact layouts. Readability requires owner review. |
| AT-70 | Select the rendering approach | 2.5D prototype only. Representative 3D comparison and native measurements are pending. |

## Decision to carry forward

The owner has not chosen a layout or approved the art. Keep this implementation on `prototype/agent-town`. After approval, record the selected composition and requested changes, then implement the production contracts separately. The 2.5D rendering experiment does not settle the spec's 3D-versus-2.5D native comparison.

## Remaining visual and native checks

The owner still needs to review the final artwork, compact crop, and appearance stages. Native always-on-top, focus behavior, click-through, keyboard recovery, display changes, Spaces, sleep and wake, provider interruption, and CPU and memory budgets have no passing evidence. No provider or native capability is presented as ready.
