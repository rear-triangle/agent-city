# Agent City

Agent City is a native Mac application for managing persistent agents through a visual city. Give the mayor a goal, inspect real assignments, decide exact file operations, and accept the final result. Both Cyberpunk and Solo Leveling use the same durable runtime.

The implementation includes a Rust core, SQLite persistence, local Ollama execution, an experimental managed Codex adapter, and a Tauri desktop shell. [Build status and evidence](docs/BUILD-STATUS.md) distinguishes verified behavior from release checks that remain open. The preserved [design prototype](docs/PROTOTYPE-GUIDE.md) is available at `/prototype/`.

## Run the desktop app

The packaged Apple Silicon application is `target/release/bundle/macos/Agent City.app`. Open that bundle in Finder. A ZIP is available at `target/release/bundle/Agent City-0.2.0-arm64.zip`. This development package has a verified local ad hoc signature; it is not Developer ID signed or notarized. Its model weights are external and are not included in the application.

To run from source:

```sh
npm ci
npm run desktop
```

To build the application:

```sh
npm run desktop:build
```

The launchers prefer native Homebrew Node on this Mac. They use the isolated Rust toolchain in `.tools` when present, otherwise your installed Rust toolchain. A fresh Mac needs Node, Rust, and Xcode command-line tools. No global toolchain or existing Codex configuration is changed by the application.

## Give the mayor a goal

1. Open **New project** and choose a folder that contains your task inputs. The initial Personal project has its own empty folder.
2. Open **Settings** and check the available execution profiles. The default Ollama endpoint is `http://127.0.0.1:11434`. Installed completion models must advertise tool support.
3. Choose **Local only** to exclude hosted inference. Choosing a `codex:` profile under **Auto** explicitly permits hosted inference for subsequent work.
4. Select **Talk to mayor**. Describe the outcome and its completion criteria.
5. Open **Decisions** when a file operation needs approval. Inspect the exact path and content before selecting **Accept** or **Deny**.
6. Open **Results** to inspect outputs. Select **Accept result** on the goal when it satisfies your criteria.

Small tasks can stay with the mayor. Larger tasks can create specialists or cooperating cities. Workers retain their identities, histories, and notes after a run ends. Internal reviews release dependent work. The Chief accepts the top-level outcome separately.

Local tools currently support reading, listing, and writing project files, coordination, versioned skill references, and memory. They do not execute arbitrary shell commands or browse the web. A request requiring those capabilities needs an explicit alternative or remains incomplete. Installed instructions do not create missing tools.

## Control work and the overlay

- **Pause** holds new assignments immediately and requests interruption. **Pausing** remains visible until the execution outcome is known.
- **Continue** uses confirmed context and retained tool results. Unknown attempts require explicit reconciliation before they can continue.
- **Send direction** records a revision. Active work is interrupted before the revised continuation starts, unless the resident or city is held.
- **Ambient overlay** displays a compact, always-on-top city with click-through enabled. Use the menu-bar **AC** menu or **Option+Shift+Space** to return to Command mode. The shortcut is configurable in Settings.
- Closing the window hides it. **Hide city** leaves the runtime running. **Quit** requires active work to stop or be reconciled.
- **Quiet mode** and the macOS reduced-motion preference suppress decorative motion. The renderer stops scheduled animation when hidden.
- **Settings → Accessibility** offers text sizes from 100% to 150%, high-contrast controls, list-only view, and status announcements. These preferences stay on this device. List-only view stops scene rendering while keeping resident and work controls available.

Use the roster or house labels to open the same resident controls. History, memory, skills, profile selection, retirement, mergers, and manual mayor replacement are available there or in **Cities**. Mergers retain the source residents and put inherited notes up for review.

## Keep and restore your work

Live data normally resides in `~/Library/Application Support/dev.agentcity.desktop/live`. Demo data uses a separate `demo` directory. `AGENT_CITY_DATA` selects an alternative base directory without modifying your existing city.

SQLite is authoritative. Each resident also has an inspectable `homes/<id>/NOTES.md`, retained skill versions, and completed session exports. External note edits survive projection repair. Use **Settings → Shortcut and downloaded skills → Review edited home notes** to import an edit explicitly.

**Save database backup** creates a consistent SQLite backup containing the full private runtime history. Do not copy a running database file without its WAL. To restore a database backup, close the target app, create a fresh data base with a `live` subdirectory, and copy the backup there as `city.sqlite3`. Launch the app with `AGENT_CITY_DATA` pointing at that base. The original project folder paths must still be valid. Uncertain execution stays held.

**Export portable archive** retains city identities, lineage, task history, memories, skill versions, and inline deliverables. It includes minimal command retry records so old requests cannot duplicate work after restore. It excludes raw provider prompts, tool payloads, and approval arguments. It does not copy source folders, model weights, provider credentials, or installed executables. Treat retained notes and results as private project data.

Use **Restore archive** in a fresh city and select a workspace for every imported project. Copy source files separately. Every restored city starts held, old absolute result paths are cleared, and no archived command executes. Existing populated cities are never overwritten by this flow.

## Retain, archive, or remove history

**Settings → History retention** controls completed run-detail age and size, and retained artifact-copy size. Unresolved work keeps its context. Accepted notes and compact identities, task summaries, command retry records, and acceptance history remain. Project files are not pruned.

**Cities → Archive project** holds the project without deleting its history. Restore it there when needed, then explicitly continue its work. An archived project can be deleted after reviewing its record counts and typing its name. Keep another project before deleting the last one. Deletion removes application records and resident projections, and leaves the project workspace and independent provider history alone.

An accepted or cancelled goal offers **Review content redaction** once every assignment is finished. The preview explains what will be removed and what remains. A redacted goal cannot resume. Backups, exports, project files, and externally edited notes are not retracted by redaction or deletion.

## Run the isolated demo and checks

```sh
npm run preview:runtime
./scripts/verify
```

The browser demo runs at `http://localhost:5180`. Its private development bridge binds only to loopback and is not part of the packaged app. Its provider is deterministic and explicitly labeled Demo.

For real provider checks, see [validation procedures](docs/VALIDATION.md). For architecture and known adapter limits, see [runtime reference](docs/RUNTIME.md). The [master specification](docs/reference/MASTER-SPEC.md) remains the intended product contract. Elections and a phone companion remain future work.
