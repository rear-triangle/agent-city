# Source notes for the agent town implementation

Verified October 14, 2026. These notes support the expanded specification. They distinguish documented APIs from proposed product behavior. T3 is the intended recipient of the handoff, not a dependency of the resulting application. The user supplied the Hamlet transcription and screenshots; they establish visual and interaction references, not verified implementation details of Hamlet.

## Recommended technical direction

Use a Rust domain core in a Tauri 2 desktop shell, with TypeScript and Three.js as the first candidate for a stylized settlement viewed through an orthographic camera. This is a design recommendation for the updated visual requirements, not a measured claim that this combination is the fastest or smallest possible application. Benchmark the complete app on the user's M2 Max with 96 GB RAM, including webview processes and graphics resources. Measure local model compute separately.

Tauri uses a Rust core and operating-system webviews in separate processes. The core handles windows, IPC, and global state. Webview libraries are not bundled in the application. [Tauri process model](https://v2.tauri.app/concept/process-model/)

On macOS, Tauri uses WKWebView, whose updates follow the operating system. Test the actual minimum supported macOS version; a passing Chrome browser demo does not establish desktop compatibility. [Tauri webview versions](https://v2.tauri.app/reference/webview-versions/)

Three.js orthographic projection keeps an object's apparent size independent of distance. That supports the reference's elevated miniature-town composition. Camera changes require a projection-matrix update. [Three.js OrthographicCamera](https://threejs.org/docs/pages/OrthographicCamera.html)

Current Three.js WebGLRenderer requires WebGL2 and no longer supports WebGL1 since r163. It exposes renderer statistics, explicit disposal, and context-loss helpers. Test context creation inside WKWebView and retain accessible task controls when graphics fail. [Three.js WebGLRenderer](https://threejs.org/docs/pages/WebGLRenderer.html)

Repeated meshes with shared geometry and materials can use InstancedMesh to reduce draw calls. Its GPU resources require disposal when no longer used. Trees, fence posts, road segments, and repeated props are candidates; batching must preserve selection identity. [Three.js InstancedMesh](https://threejs.org/docs/pages/InstancedMesh.html)

Proposed rendering policy: pause drawing while hidden, bound active frame rate, lower resolution and effects in compact mode, avoid an inference call for ambient movement, and keep day/night lighting independent of work scheduling. Profile before adding post-processing, dynamic shadows for every light, or crowds. No source reviewed establishes the performance cost of the user's reference video.

## Overlay facts and remaining native tests

Tauri documents always-on-top, cursor-event ignoring, and visibility across workspaces. Each window mutation requires its own capability permission; the default window capability primarily grants getters. `setVisibleOnAllWorkspaces` is unsupported on Windows, iOS, and Android. [Tauri window API](https://v2.tauri.app/reference/javascript/api/namespacewindow/)

The configuration documentation describes `macOSPrivateApi` as a no-op in Tauri 2.12.1 and later because those APIs are always enabled. Pin the chosen version and verify its behavior instead of copying older transparency recipes. Relative app-data paths beside a macOS bundle can fail under App Translocation or a nonwritable installation location. [Tauri configuration](https://v2.tauri.app/reference/config/)

Recommended native acceptance tests are Ambient click-through, Command mode interaction, global-shortcut recovery, menu-bar recovery, focus preservation, fullscreen Spaces, Stage Manager, multiple displays, mixed scale factors, monitor disconnect, sleep/wake, and recovery after a webview crash. The existence of a window method does not prove every combination works.

An ambient window that ignores all cursor input cannot use clicking itself as the sole way back to interaction. The first implementation should expose both a global shortcut and menu-bar command. Keep filesystem paths and exported project data outside the application bundle.

## Codex integration evidence

The official app-server documentation provides thread creation/resumption, turn creation, interruption, streamed items, and approval requests. Initialize each connection before other requests. `thread/resume` reopens stored history; a subsequent `turn/start` begins continuation. `turn/interrupt` requests cancellation, with completion reporting `interrupted`.

Approvals are server requests correlated with thread, turn, item, and request IDs. Responses have provider-specific decision values. `serverRequest/resolved` confirms response or cleanup. Some permissions are turn-scoped; others can persist for a session. Dynamic tools are experimental. `thread/shellCommand` explicitly runs outside the thread sandbox.

The documentation still labels the app-server command and WebSocket transport experimental and unsupported for production workloads, despite separate stable and experimental API fields. The CLI can generate schemas matched to its installed version. [Official Codex app-server documentation](https://learn.chatgpt.com/docs/app-server)

Product consequences are proposed requirements: use a tested adapter and generated schemas; prefer a managed child process over an exposed network listener; retain provider IDs; show Pausing until interruption is confirmed; distinguish interruption from exact suspension; reconcile unresolved approvals after disconnect; do not replay an approval response blindly. A new app-server process is not proof of control over all conversations in another desktop client. Do not imply that stopping a parent turn stopped every background child or terminal.

## Model hosting versus agent execution

Ollama's tool-calling example runs tools in the client application, appends results to the conversation, and calls the model again. Its agent loop is application code. [Ollama tool calling](https://docs.ollama.com/capabilities/tool-calling)

Design consequence: an endpoint serving model responses is not sufficient to provide residents with task queues, tool permissions, cancellation, memory loading, approvals, or recovery. The local adapter needs a tested execution loop or a supported existing agent runtime. Keep model selection separate from agent identity. Keep credentials and model weights outside town exports.

## Conceptual risks to resolve explicitly

These are architectural deductions and proposed constraints, not behavior claimed by the sources above.

### Roads define broker routing, not universal isolation

The Rust broker can reject an unauthorized recipient and require a road for a broker message. It cannot prove that agents communicate only through that broker if those agents can also write shared files, invoke arbitrary subprocesses, or access unrestricted network endpoints. A shared repository is itself an information channel.

The first release should promise enforcement of town-managed messages and clearly describe the limits of workspace isolation. Stronger isolation requires independently tested filesystem, process, and network restrictions. Never treat an instruction in a prompt as a security boundary. Authentication must bind a sender to its resident identity; accepting a caller-supplied sender ID alone defeats the graph.

Separate three graphs in the domain model: authorized communication edges, task dependencies, and visible road geometry. They can refer to one another without becoming interchangeable. Rerouting the drawn path cannot grant permissions. A railroad upgrade cannot silently widen access or create a faster model.

### Persistent files are only stored memory

Files in a house do not automatically enter a model's next context. Define explicit selection, size limits, provenance, and context-loading steps. Preserve originals and version summaries. Treat downloaded documents and prior agent notes as data, not higher-priority instructions. Record which memory versions a run used. Separate stable resident identity from provider thread IDs and model choices.

### A mayor should not reason on every delivery

Use deterministic routing for ordinary delivery. Invoke the mayor for a planning or coordination decision. Otherwise a central mayor can become a latency, token-cost, and context bottleneck. Bound planning turns, delegation depth, concurrent work, retries, critique cycles, and spend. Require explicit completion evidence rather than equating a finished response with an accepted task.

### Approvals must resolve real pending actions

One approval list is a presentation layer over distinct provider requests and application policy decisions. Keep request origin, supported decisions, scope, and current validity. A denied tool action is not necessarily a failed task. A completed turn can invalidate an unanswered request. Rechecking state before applying a decision avoids acting on stale cards.

### Preserve history without unbounded growth

Archival and merges can preserve identity lineage without loading every historic session into memory or retaining sensitive content forever. Define export, retention, deletion, and redaction policies. A merged resident should be a successor with links to archived originals; do not concatenate conflicting notes and call the result reliable knowledge.

### Animation must remain a view of events

Persist and route a message independently of its courier. Rendering stalls, hidden windows, travel time, and skipped animation must not block execution. Delivery, processing, and acknowledgement are different states. Event bursts may be visually aggregated, but the road inspector must retain the individual records within the configured retention period.

### Gamification must reward useful outcomes

Raw message counts and token consumption are poor upgrade criteria. Use accepted results and deliberate collaboration milestones. Explain why an upgrade happened and allow visual customization without requiring wasteful work. Idle residents should generate no inference calls. Loaded models, storage, the renderer, and the operating system still consume resources.

## Evidence gaps

No source in this check establishes that arbitrary Mac agents expose compatible lifecycle events. No benchmark establishes a safe CPU, GPU, memory, or battery budget for the finished town. No source establishes that the reference town uses the proposed stack. No assumption about T3's internal runtime is necessary to build this app. These gaps should remain explicit in the implementation status rather than being replaced with simulated success.

## Version 2.1 visual clarification

The owner now prioritizes Cyberpunk and Solo Leveling versions and supplied four Solo Leveling System interface references. Medieval is no longer the default. The implementation must compare attractive stylized 3D with a polished 2.5D treatment before choosing on measured native cost and visual quality. The sources above establish API behavior, not which artistic direction the owner prefers. Both themes retain the evolving settlement and persistent agent behavior.
