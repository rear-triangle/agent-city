# Agent City prototype status

Revision 9 refines character geometry and makes agent activity the default inspector, with direct Skills and Memory access. Revision 8 adds grab-based map panning, orbit tilt, and a collapsed navigation guide. Revision 7 adds persistent Chief visibility, click-to-teleport travel, compact house menus, and Solo Leveling resident names with Igris as mayor. Revision 6 applies the owner’s typography choices: Friend or Foe Tall BB for Solo Leveling, Rajdhani for primary Cyberpunk text, and Orbitron for secondary metadata. Revision 5 refines the Solo Leveling Player menus and gives the Chief a grander isolated sanctuary with teleport travel. Revision 4 applied the supplied Cyberpunk menu, street, and apartment references. Revision 3 added the Chief, indoor work, intentional trips, scripted speech, and revised anime System menus. Revision 2 addressed the owner's eight criticisms and the request for a system that can evolve or devolve as needed. This remains a design prototype. Production work and every native acceptance gate await approval.

## Corrections and behavior

| Owner feedback | Revised prototype |
| --- | --- |
| Name | Agent City throughout the product UI and package metadata. Town remains the name for a project settlement inside the city network. |
| Additional towns | Create, name, and switch towns. Residents, notes, queues, approvals, demand, and stage are stored separately in memory. |
| More depth | Actual orthographic 3D with real geometry, shadows, orbit, zoom, and follow camera. |
| Visible agents | Residents work indoors with furnished cutaways. Nyx makes a finite example errand. A message robot follows generated roads for handoffs. The Chief teleports directly from an isolated sanctuary. |
| Time-based change | Local computer time is the default. Gradual lighting has a 90-second preview and an explicit fixed-light override. The clock runs independently of task pause. |
| Full themes | Cyberpunk uses primary Rajdhani and secondary Orbitron, red frames, cyan selections, angular buttons, framed toggles, and hexagonal skill displays. Solo Leveling uses locally installed Friend or Foe Tall BB with an Abel fallback, translucent blue System windows, luminous rails, inset menu labels, geometric linework, shadow armor, and different architecture. |
| Generated roads | A grid pathfinder routes around home and annex footprints. Seeded layout changes and population changes rebuild geometry. Permission relationships do not come from the drawn path. |
| Substantial characters | Original 3D bodies have articulated limbs, coats or capes, armor, boots, visors or helmets, and role equipment. Portraits use matching visual cues. |
| Evolution and devolution | Work-arrival and work-settlement fixtures add or retire homes and annexes. Recall preserves identity and notes. Consolidation excludes the mayor and residents with unfinished work. Inactivity causes no penalty. |

## Runtime and limits

The prototype uses TypeScript, DOM controls, CSS, and Three.js 0.186.1. Vite 8.3.4, TypeScript 7.0.2, and Three.js types 0.186.0 are pinned development dependencies. Three.js is the only runtime library. Four bundled open-font files total about 92 kB. Friend or Foe Tall BB uses the owner’s installed local face; its commercial binary is not redistributed.

Repeated static primitives are instanced. Visible motion is capped at 24 fps in expanded views and 12 fps in the compact view. Indoor work without a cutaway falls back to two-second updates for optional chatter; settled scenes with no chatter stop motion callbacks. Quiet mode stops those animation callbacks. Changes and coarse clock updates can still redraw. Fixed lighting has no periodic renderer clock update. Hidden documents stop rendering callbacks and clock timers.

Towns, layouts, and notes are actual interactive prototype state. They are not durable. Reload resets them. Task starts, demand events, approvals, and work results are simulated. No provider process or model is launched.

The demand buttons replay example workload events. This is not a production automatic staffing policy. They demonstrate the physical response and retained history. The demo does not infer success from animation or grant permissions when capacity grows.

No Rust core, Tauri shell, SQLite, real provider adapter, durable broker, native overlay, or application-wide resource measurement is implemented. The larger 3D bundle is a deliberate prototype tradeoff for the requested depth. The lightweight target is not considered passed.

## Milestones

| Milestone | Status |
| --- | --- |
| Design prototype revision 9 | Interactive and browser checked. Awaiting owner review. |
| M0 integration proof | Deferred until design approval. |
| M1 durable domain | Not started. In-memory prototype state only. |
| M2 communication and memory | Inspectors and geometry only. Durable authority and context preparation pending. |
| M3 visual slices | Both 3D themes available. The previous 2.5D source is retained for comparison. Native resource and visual gates pending. |
| M4 native workflow | Not started. Browser composition is not a native overlay. |
| M5 personal preview | Not started. Packaging, durability, full accessibility audit, and resource trials pending. |
| M6 and M7 | Production features deferred. Multi-town composition has been brought forward for this prototype at the owner's request. |

## Revision 9 characters and activity

The default resident inspector now leads with its task, current status, and latest exchange. Running task titles come from the task record. Waiting, planned work, and resting states are labeled separately. Skills and Memory have direct footer buttons; that footer stays outside the scrolling activity region. Look inside is a small secondary text action. The details arrow retains queue, equipment, history, and interruption controls. Newly queued work returns to the activity view, where Run next demo task updates the displayed task and status.

Character construction now uses smooth ellipsoids, tapered helmets, rounded limbs, facial features, hair, coat panels, and role accessories. Named shadows have different silhouettes. Original SVG portraits use matching cues. Construction geometry is shared, and static parts merge per joint and material. The Chief's visibility pass now retains depth between its own body and facial parts instead of flattening all overlapping geometry. Room focus starts from the resident's facing direction, with existing obstruction avoidance.

| Browser check | Observed outcome |
| --- | --- |
| Activity hierarchy | The reader showed its task, Reading status, and latest received exchange. Look inside appeared in the footer as a secondary text action. |
| Direct details | Skills opened Source extraction and Citation tracking. Memory opened the same two stored notes. Back to activity restored the compact work view. |
| Queue | Assigning Review the new character silhouettes returned to the activity view with one queued task. Run next demo task changed its record to Running and displayed that exact title. |
| Small viewport | At 390 × 844, both themes kept Skills and Memory visible outside the scroller. The critic's waiting reason and Review request action remained available. Checked buttons and document width had no horizontal overflow. |
| Characters | All eight starter characters were reviewed together in a temporary browser character sheet. Tusk and a Cyberpunk reader were checked in room cutaways with the Chief visible. The sheet is evidence, not another product screen. |
| Quiet mode | A fixed-light 1.1-second interval added zero rendered frames. The Chief remained visible. |
| Build | TypeScript and Vite passed. The existing renderer-chunk warning remains. |

Model counts below describe all seven starter residents plus the Chief, before the scene's extra Chief marker ring. Mesh count is not a frame-rate measurement.

| Model cohort | Previous meshes | Revised meshes | Previous triangles | Revised triangles |
| --- | ---: | ---: | ---: | ---: |
| Cyberpunk | 223 | 219 | 23,372 | 50,915 |
| Solo Leveling | 238 | 208 | 23,384 | 36,077 |

Higher polygon counts improve the silhouettes and faces. Most actors remain hidden indoors, and no character shadow maps or texture downloads were added. The temporary eight-character shadow sheet rendered 209 calls and 36,079 triangles including its two-triangle floor. The sheet used neutral review lighting, so it does not establish in-app performance. Native memory, CPU, and GPU gates remain pending.

## Revision 8 camera checks

Normal dragging pans across the ground plane. The point under the initial grab remains under the cursor through the gesture, using the active camera orientation and zoom. Right-drag and Option/Alt-drag orbit horizontally and tilt vertically. The Move map guide exposes a selectable left-drag Orbit mode for trackpads and touchscreens. Scroll and existing controls zoom; Reset view restores the default position, angle, tilt, and zoom.

A five-pixel drag threshold keeps short clicks available for Chief teleport and house selection. Camera dragging releases resident follow without closing the cutaway. Pointer cancellation and lost capture end the gesture. Theme changes and route-marking changes retain the current camera position; entering another town resets the view.

| Check | Observed outcome |
| --- | --- |
| Grab alignment | Browser pointer-event replay moved a projected house by 120 × 40 pixels for a 120 × 40 pixel pan, within floating-point precision. Yaw stayed unchanged. |
| Pan after orbit | A subsequent -90 × 65 pixel pan at a changed yaw and tilt produced the same projected displacement. |
| Zoomed pan | At 120% zoom, an 85 × -45 pixel drag produced the matching screen displacement. |
| Orbit | Right-button and Alt-modified replay changed yaw and elevation. Selecting Orbit in the guide also changed ordinary left-drag behavior and updated the hint. |
| Tilt bounds | Large vertical gestures stopped at 0.35 and 1.25 radians above the ground plane. |
| No unintended travel | Pan, orbit, and cancelled drag left the Chief's world position unchanged and did not open an inspector. |
| Camera continuity | Toggling route markings kept the panned target. Reset view restored target [0, 1.5, 0], yaw 0.67, default tilt, and 100% zoom. |
| Narrow guide | At 390 × 844, mode buttons remained visible above the compact house menu. Help text scrolls inside the small guide and document width does not overflow. |
| Build | TypeScript and Vite passed. The existing renderer-chunk warning remains. |

These checks exercised the browser event handlers with synthetic pointer events; pointer capture was stubbed only during replay because synthetic events have no native active pointer. Normal browser clicks and guide buttons were also exercised. Physical mouse and trackpad feel remains subject to owner review. This is still a throwaway approval prototype, with no new runtime dependency or durable camera state.

## Revision 7 interaction checks

House selection now starts with a small menu containing the name, role, activity, current task, Look inside, and task assignment. A pending request adds its review action. More details reveals the existing complete inspector; Less detail collapses it. The Chief also has a compact entry menu, with the full Player window one click away. Mobile brief menus reserve a separate area for the world.

The Chief starts on the sanctuary terrace, remains rendered through building occlusion, and has a static ground ring. World clicks teleport the same character without creating agents, tasks, or roads. House clicks use the entrance and open its menu. The location marker remains visible at the camera edge or in a covering inspector's header. Theme and layout changes preserve the Chief's position in the current town.

| Check | Observed outcome |
| --- | --- |
| Open ground | A canvas click moved the Chief from the sanctuary to a ground point immediately. No task was created and no Chief road appeared. |
| House selection | A 3D house click and a house-label click selected the reader, moved the Chief to x=0, z=-3.15, and opened the compact menu. |
| Cutaway | Look inside revealed the reader or mayor working indoors; the Chief stayed visible at the entrance. |
| Detail expansion | More details restored Work, Equipment, Memory, Skills, and History in Solo Leveling. The reader's two existing notes remained intact. |
| Theme change | Solo Leveling showed Igris, Kaisel, Tusk, Iron, Shadow Knight 01, Tank, and Beru. Cyberpunk restored Atlas, Nyx, Sage, Vector, Quill, Echo, and Vex. The Chief's position, resident IDs, and notes were retained. |
| Growth | Expanding to the 12-agent prototype cap produced 12 unique Solo Leveling names, including numbered Knight, Mage, and Assassin aliases. |
| Paused and Quiet | Ground teleport worked with both enabled. The Chief remained visible, tasks stayed empty, accepted outcomes stayed at two, and a 1.1-second fixed-light interval added zero rendered frames. |
| Layouts and narrow view | Compact menus were checked in A, B, and C. Both themes were checked at 1440 × 900 and 390 × 844 without horizontal document overflow. Checked house-menu buttons did not overflow their text bounds. |
| Build | TypeScript and Vite passed. The existing renderer-chunk warning remains. |

The named-shadow selection follows the owner's anime scope through Beru. The [official Season 2 shadow visuals](https://sololeveling-anime.net/special/shadows-visual/) provided the character roster reference; English aliases follow the owner's naming direction. Generic numbered names avoid extending beyond that scope. Roles and character models remain this prototype's representations.

This revision adds no runtime library, model, image, or font. Revision 7 transfer sizes were approximately 684 kB JavaScript before compression, 183 kB compressed, 24 kB compressed CSS, and 92 kB bundled open fonts. Native resource gates remain pending. Earlier observations below describe the behavior of their respective revisions.

## Revision 6 typography checks

The browser loaded the installed Friend or Foe Tall BB face for Solo Leveling and the bundled Orbitron face for Cyberpunk secondary text. Cyberpunk actions and main content retain Rajdhani. Building signs wait for fonts before drawing. Abel remains the portable fallback when the owner’s installed font is unavailable.

At 1440 × 900 and 390 × 844, both themes had no horizontal document overflow. The inspected buttons and captions had no text overflow. The Player window was visually reviewed at both sizes. TypeScript and Vite passed; the existing renderer-chunk warning remains. Earlier observations below retain the fonts used at the time.

## Revision 5 browser observations

The latest Chief instruction supersedes the earlier road-based owner visits. The Chief now has a separate estate terrace, no generated road, and instantaneous teleport travel to active homes. Resident communication still uses its existing roads.

| Check | Observed outcome |
| --- | --- |
| Estate | Solo Leveling has a silver-and-blue domed sanctuary with four spires, columns, broad stairs, perimeter lights, letterbox, and arrival waystone. Cyberpunk has a private penthouse with gold roof trim, columns, terrace, and neon details. Both sit outside the city footprint. |
| Chief road exclusion | Starter-city generated routes contain scout, reader, analyst, writer, editor, and critic. There is no Chief route or connecting bridge. |
| Teleport | A single click put the Chief at Sage's entrance, x=0 and z=-3.15, with moving=false. Teleport home returned directly to x=-5 and z=12.7, location=Inside sanctuary. |
| Owner control during pause | The same teleport worked while paused=true. Residents remained held. The Chief is excluded from provider scheduling and staffing. |
| Player screen | Six access slots open letterbox decisions, a new goal, headquarters, visits, domains, and outcomes. The complete outline figure and slot layout fit together. |
| Resident equipment | Sage showed the Source codex and Shadow attire. Memory displayed two notes; the Memory slot opened those notes. Role equipment is a cosmetic representation of the city model. |
| Notification | Separate exclamation icon and title, thin silver inner borders, brighter blue outer rails, centered request, scoped details, and Allow once / Deny controls. Allow once changed approval to allowed and phase to review. |
| Quest info | The goal and existing deliverable checklist use the refined System window. Completion styling uses green without introducing penalties or arbitrary game stats. |
| Theme isolation | Switching to Cyberpunk retained Rajdhani and removed the System rails. Solo Leveling retained Abel. The Equipment tab falls back to Work in Cyberpunk. |
| Narrow viewport | Document width equals 390. All six Player slots fit inside the viewport, from x=77 through x=347. |
| Quiet | Zero added frames across a 1.1-second fixed-light check. Portal animation is suppressed in Quiet mode. |
| Build | TypeScript and Vite passed. The large renderer-chunk warning remains. |

Revision 5 adds original SVG figures, static border artwork, CSS, and procedural estate geometry. No runtime library, font, or image download was added. The arrival rings animate briefly; their animation does not control provider state. Browser captures and a short teleport recording support review. Native performance gates remain pending.

Revision 6 build transfer sizes were about 678 kB JavaScript before compression, 182 kB compressed, 23 kB compressed CSS, and 92 kB of bundled fonts. These are not process-memory measurements.

## Revision 4 browser observations

The Cyberpunk reference pass remains on the prototype branch and awaits design approval. The new street and room details are procedural. No source screenshot, game logo, character model, or extracted game UI asset is included.

| Check | Observed outcome |
| --- | --- |
| Menus | Red borders and inactive labels, cyan selections, notched buttons, framed OFF/ON switches, and condensed Rajdhani throughout the Cyberpunk interface. |
| Skills | Atlas has a hexagonal display of Task planning, Evidence synthesis, and Delegation, backed by the same accepted skill list. |
| City | Layered awnings, balcony rails, signboards, utility pipes, roof vents, overhead cables, lanterns, market stands, and background towers. Street geometry still comes from the existing pathfinder. |
| Lighting | Local time remains the default. The fixed night study shows cyan and red street illumination. Clock-mode and fixed-preset changes now redraw lighting immediately. |
| Interior | A furnished cutaway includes a backlit wall strip, warm wall panel, desk, seat, and utility cable. |
| Quiet | Zero additional frames across a 1.1-second fixed-light check, with motion scheduling disabled. |
| Theme isolation | Solo Leveling retained Abel and its translucent framed controls. The seven resident keys remained equal after switching back to Cyberpunk. |
| Mobile | Page width equals the 390-pixel viewport. The framed switches and menu content remain within the scrollable inspector. |
| Layouts | B retained the 838 by 678 map alongside command content. C retained its 458 by 308 compact map. |
| Label positioning | Explicit redraws now reposition world labels immediately when an inspector or viewport changes the map size. |
| Build | TypeScript and Vite passed. The existing large-chunk warning remains. No release resource gate is claimed. |

This pass adds two unshadowed point lights and static geometry. It adds no bloom pipeline, particle loop, image downloads, runtime libraries, or font files. Signs use small generated canvas textures. Native CPU, GPU, and memory budgets still need the full-build profiling gates.

Current transfer sizes are about 669 kB JavaScript before compression, 179 kB compressed, and 20 kB compressed CSS. Fonts remain about 53 kB. One night-view observation had 319 draw calls and about 136,000 triangles. These are browser renderer counters, not native process measurements.

Revision 4 captures are `cyberpunk-reference-night.png`, `cyberpunk-reference-day.png`, `cyberpunk-reference-skills.png`, `cyberpunk-reference-interior.png`, and `cyberpunk-reference-mobile.png` in `docs/evidence/`.

## Revision 3 browser observations

Checked through T3 Code's collaborative browser at 1440 by 900 and 390 by 844.

| Check | Observed outcome |
| --- | --- |
| Theme typography | Computed Rajdhani for Cyberpunk and Abel for Solo Leveling. Same seven resident keys after switching. |
| Indoor work | Sage remained at the home coordinate with `location=Inside home`. Look inside revealed the resident, desk, monitor, and shelves while hiding the outer home. |
| Chief headquarters | Separate Chief home, letterbox actions, white uniform and crest, furnished cutaway, and visits to residents. The Chief is excluded from agent staffing and consolidation. |
| Chief movement | Two samples 1.1 seconds apart showed movement on a generated road. A visit to Atlas completed, then Return home ended at the original coordinate with `location=Inside headquarters`. |
| Courier | MSG-025 traveled from scout to reader via the mayor hub, changed from In transit to Delivered, and then left the scene. Delivery is an in-memory animation fixture. |
| Pause | Character positions and courier progress were equal across snapshots after pause acknowledgement. No motion frames were scheduled. |
| Quiet | No additional frames across a 1.2-second local-time check; speech cleared. |
| Capacity and paths | Work arrivals produced nine agents plus the Chief access route. Every route had cells before and after replanning. Work settlement returned to seven agents. |
| Speech | Activity and quirk bubbles appeared over homes and traveling characters. Maximum two bubbles, optional toggle, no inference. |
| Mobile | Document width equaled 390 pixels. All four capacity and Chief controls stayed inside the viewport. System and Cyberpunk inspectors remained scrollable. |
| Layouts | B kept command content left of the 3D view. C retained its compact 458 by 308 canvas. |
| Build | TypeScript and Vite passed. The renderer chunk still triggers the 500 kB size warning. |

Saved revision 3 screenshots are in `docs/evidence/`. A browser recording also captures the indoor view, courier, and Chief visit.

The Hamlet transcripts and images establish a resident community with persistent homes, a Chief, message roads, and adaptive staffing. Durable memory, road-to-rail maturity, mergers with retained records, elections with a two-thirds threshold, the former mayor remaining a resident, and production inter-town broker links remain full-build requirements. This prototype does not claim to implement them. Theme changes and layout replanning rebuild the scene and can interrupt an in-flight demo courier. Production message delivery must be independent of scene animation.

## Revision 2 browser baseline

These earlier observations describe the previous revision, before indoor work and the System font correction.

Checks used T3 Code's collaborative browser at a 1440 by 900 CSS viewport and at 390 by 844. Saved screenshots can have a different raster size because the preview capture scales its output.

| Check | Observed outcome |
| --- | --- |
| Found Glass Harbor | Two town records. The new town had `phase=idle`, `approval=none`, and seven resting residents. |
| Town isolation | A Glass Harbor note and running reader task remained in that town. Afterlight retained zero queued tasks and did not receive the note. |
| Work arrives | Seven homes became nine. Appearance stage became 2. All eight specialist roads had generated paths. |
| Work settles | Nine homes became seven. The extra resident records, notes, and task history remained. |
| Recall Forge | Eight active homes. Forge retained its resident key and original note. |
| Replan layout | Path geometry changed. Resident keys remained unchanged and every active specialist retained a generated route. |
| Pause | After acknowledgement, positions remained equal across successive snapshots and animation scheduling stopped. |
| Continue | Positions changed and 26 frames advanced in a 1.2-second check. |
| Clock while paused | Clock advanced approximately 0.4008 simulated hours over 1.5 seconds. Agent positions remained equal. |
| Quiet mode | Frame count remained unchanged across a 1.1-second check with local time selected. |
| Theme cycles | Three repeated cycles returned to 201 geometries and 7 textures for Cyberpunk, and 238 geometries and 10 textures for Solo Leveling in that scene. Resident keys remained equal. These are renderer counters, not process-memory measurements. |
| Typography | Browser computed Rajdhani for Cyberpunk and Cormorant Garamond with Cinzel headings for Solo Leveling. |
| Layouts | City first, Command room, and Desktop companion render with the 3D scene. |
| Narrow viewport | Document width equals the 390-pixel viewport. Capacity buttons remain within the viewport. Roster scrolls horizontally. |
| Movement evidence | A browser recording captures moving agents, a closer follow view, and the theme switch. The video is a review artifact, not part of the app bundle. |

The first broad automation sequence encountered ambiguous control selectors. The checks above were rerun successfully with scoped controls. No alternate browser system was used.

The scene workload in one check was about 89,000 rendered triangles and 259 draw calls. CPU, GPU allocation, combined process memory, native focus behavior, and long-run stability were not measured. These browser counters do not establish the native budgets.

## Commands and source

Commands ran from `/Users/alvarobalkowski/Documents/agent-city/agent-city`.

| Command | Outcome |
| --- | --- |
| `PATH=/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin npm install --save-exact three` | Installed the pinned renderer. |
| `PATH=/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin npm install --save-dev --save-exact @types/three` | Installed matching development types. |
| `PATH=/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin npm install --package-lock-only` | Reconciled package naming and the lockfile. Audit reported zero vulnerabilities. |
| `PATH=/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin npm run build` | TypeScript check and static build pass. Vite reports that the uncompressed renderer chunk exceeds 500 kB. |
| `git branch -m prototype/agent-city` | Renamed the isolated prototype branch to match the product. |

The revision 3 prototype contains roughly 663 kB of JavaScript before compression, about 177 kB compressed. CSS is about 17 kB compressed, and active fonts total about 53 kB. These are transfer sizes, not application memory. Native profiling remains the deciding gate.

The renderer follows the official [WebGLRenderer](https://threejs.org/docs/pages/WebGLRenderer.html) and [InstancedMesh](https://threejs.org/docs/pages/InstancedMesh.html) contracts. These APIs do not establish this application's resource use.

## Acceptance mapping

Every full-release acceptance gate remains pending. The rows below identify prototype evidence without converting simulation into production acceptance.

| ID | Scenario | Prototype evidence or pending work |
| --- | --- | --- |
| AT-01 | Create the starter town | Seven homes and six generated mayor routes in the starter town. In-memory only. |
| AT-02 | Rename a resident and switch its model while idle | Pending full build. No release evidence in this prototype. |
| AT-03 | Assign a goal with valid scope | A task can be queued, then started through Run demo task. No durable command or provider dispatch. |
| AT-04 | Repeat the same assignment command | Pending full build. No release evidence in this prototype. |
| AT-05 | Submit a plan with a dependency cycle | Pending full build. No release evidence in this prototype. |
| AT-06 | Reach the concurrency limit | Pending full build. No release evidence in this prototype. |
| AT-07 | Pause during an active turn | Simulated pause acknowledgement holds visible agent positions. |
| AT-08 | Finish naturally while pause is pending | Pending full build. No release evidence in this prototype. |
| AT-09 | Lose interruption acknowledgement | Pending full build. No release evidence in this prototype. |
| AT-10 | Resume an interrupted task | Continue restores motion. No real provider history is attached. |
| AT-11 | Pause a parent with an active child | Pending full build. No release evidence in this prototype. |
| AT-12 | Pause the town | City pause holds simulated resident movement while the independent world clock advances. |
| AT-13 | Deliver a late completion for an old run | Pending full build. No release evidence in this prototype. |
| AT-14 | Provider turn ends successfully | The sample review and owner acceptance are separate actions. |
| AT-15 | Accept a goal twice, replay, and restart | Repeated acceptance is guarded in memory. Durable deduplication remains pending. |
| AT-16 | Reopen an accepted goal | Pending full build. No release evidence in this prototype. |
| AT-17 | Send through a valid direct road | Pathfinding produces inspectable paths for the permitted starter relationships. Broker enforcement is unimplemented. |
| AT-18 | Send without a direct road but valid mayor roads | The route inspector explains mayor relay. No durable routing implementation. |
| AT-19 | Send without any permitted route | Pending full build. No release evidence in this prototype. |
| AT-20 | Spoof a sender in tool arguments | Pending full build. No release evidence in this prototype. |
| AT-21 | Disable a road while messages are queued | Pending full build. No release evidence in this prototype. |
| AT-22 | Duplicate message delivery | Pending full build. No release evidence in this prototype. |
| AT-23 | Crash after recipient provider submission | Pending full build. No release evidence in this prototype. |
| AT-24 | Fill a mailbox | Pending full build. No release evidence in this prototype. |
| AT-25 | Create a ping-pong exchange | Pending full build. No release evidence in this prototype. |
| AT-26 | Send twelve empty acknowledgements | Pending full build. No release evidence in this prototype. |
| AT-27 | Meet the useful-collaboration rule | Pending full build. No release evidence in this prototype. |
| AT-28 | Stop or hide the renderer during delivery | Scene animation and in-memory work commands are separate. No real delivery. |
| AT-29 | Restart after a committed memory update before file replacement | Pending full build. No release evidence in this prototype. |
| AT-30 | Externally edit notes based on an old revision | Pending full build. No release evidence in this prototype. |
| AT-31 | Run a new task after a useful note was accepted | Pending full build. No release evidence in this prototype. |
| AT-32 | A model summary contradicts an owner instruction | Pending full build. No release evidence in this prototype. |
| AT-33 | A retrieved document asks for wider permissions | Pending full build. No release evidence in this prototype. |
| AT-34 | Update a skill during an active run | Pending full build. No release evidence in this prototype. |
| AT-35 | Decide a real provider approval | A simulated permission request can be allowed once. No real provider approval. |
| AT-36 | Approve a stale or altered request | Pending full build. No release evidence in this prototype. |
| AT-37 | Provider cleans up an unresolved approval | Pending full build. No release evidence in this prototype. |
| AT-38 | Deny one specialist's request | The denial branch adapts the sample result. Real provider evidence remains pending. |
| AT-39 | Crash after sending a launch before recording its response | Pending full build. No release evidence in this prototype. |
| AT-40 | Storage becomes full or read-only | Pending full build. No release evidence in this prototype. |
| AT-41 | Frontend misses a snapshot patch | Pending full build. No release evidence in this prototype. |
| AT-42 | Provider emits an unsupported event | Pending full build. No release evidence in this prototype. |
| AT-43 | Enter Ambient mode while typing elsewhere | Pending full build. No release evidence in this prototype. |
| AT-44 | Use the shortcut and menu-bar recovery | Pending full build. No release evidence in this prototype. |
| AT-45 | Remove the display containing the overlay | Pending full build. No release evidence in this prototype. |
| AT-46 | Sleep and wake during active work | Pending full build. No release evidence in this prototype. |
| AT-47 | Lose WebGL context | Pending full build. No release evidence in this prototype. |
| AT-48 | Enable reduced motion | Quiet mode stops work-animation frames. OS reduced-motion preference is read on startup. |
| AT-49 | Change the day/night preset | Local time drives light by default. A 90-second preview and explicit fixed-light inspection are available. |
| AT-50 | Change themes repeatedly | Repeated theme cycles retain resident keys and return to the same geometry and texture counts for each theme. |
| AT-51 | Export and import into a clean installation | Pending full build. No release evidence in this prototype. |
| AT-52 | Import an archive with path traversal or invalid digest | Pending full build. No release evidence in this prototype. |
| AT-53 | Start a second app instance on the same data directory | Pending full build. No release evidence in this prototype. |
| AT-54 | Switch to an observed external agent | Pending full build. No release evidence in this prototype. |
| AT-55 | Seed test secrets in provider payloads | Pending full build. No release evidence in this prototype. |
| AT-56 | Inspect a house using only keyboard and screen reader | DOM roster, labeled forms, tabs, dialog focus control, and inspectors are present. Assistive-technology audit pending. |
| AT-57 | Replace the mayor manually | Pending full build. No release evidence in this prototype. |
| AT-58 | Merge two residents later | Pending full build. No release evidence in this prototype. |
| AT-59 | Run a seven-resident election later | Pending full build. No release evidence in this prototype. |
| AT-60 | Spawn a resident during an open election | Pending full build. No release evidence in this prototype. |
| AT-61 | Send across towns later | Separate towns can be created and switched. Cross-town message policy and delivery remain unimplemented. |
| AT-62 | Leave the town idle for an hour | Idle cities have no work-animation loop. Clock updates are coarse. No model calls exist. Native profiling pending. |
| AT-63 | Complete an internal specialist task | Pending full build. No release evidence in this prototype. |
| AT-64 | Revise an accepted prerequisite | Pending full build. No release evidence in this prototype. |
| AT-65 | Queue two mutating runs against one checkout | Pending full build. No release evidence in this prototype. |
| AT-66 | Mayor delegates and waits for results | Pending full build. No release evidence in this prototype. |
| AT-67 | Switch between Cyberpunk and Solo Leveling | Models, architecture, typography, frames, and menus change coherently across both themes. |
| AT-68 | Advance and reopen qualifying accepted goals | Demand fixtures expand and consolidate homes and annexes. Retired residents retain notes, identity, and task history. No persistence. |
| AT-69 | Inspect Solo Leveling controls at compact and expanded sizes | Blue System frames and serif menu typography are present in expanded and compact layouts. Owner visual review pending. |
| AT-70 | Select the rendering approach | Actual 3D candidate available alongside archived 2.5D source. Native comparison and resource acceptance pending. |

## Review gate

The owner has not approved the revised appearance or capacity rules. Keep this implementation on `prototype/agent-city`. The first 2.5D prototype is preserved in commit `2ea2d26` and in `src/prototype/scene-2d.ts`.

Before production adoption, implement demand policy against authoritative work state, preserve retired identity and memory durably, validate road authority separately from geometry, and compare native resource use. The prototype does not settle those implementation gates.

## Visual evidence

- [Cyberpunk 3D city](evidence/cyberpunk-3d.png)
- [Solo Leveling System inspector](evidence/solo-system-3d.png)
- [Narrow Cyberpunk view](evidence/mobile-cyberpunk.png)

The movement recording is a separate local review attachment. It is not included in the runtime bundle or committed to the repository.
