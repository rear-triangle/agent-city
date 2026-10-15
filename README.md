# Review the Agent City prototype

Open [Agent City](http://localhost:5173/prototype/?variant=A&theme=cyber). This is the revised design prototype. The production build awaits approval.

Use the header to switch between Cyberpunk and Solo Leveling. Drag the scene to orbit the real 3D geometry, scroll to zoom, or select a resident and choose **Look inside** for a furnished cutaway.

| Layout | Preview | Main interaction |
| --- | --- | --- |
| A | [City first](http://localhost:5173/prototype/?variant=A&theme=cyber) | Explore the city and open contextual inspectors. |
| B | [Command room](http://localhost:5173/prototype/?variant=B&theme=cyber) | Start with a goal and keep the city alongside the plan. |
| C | [Desktop companion](http://localhost:5173/prototype/?variant=C&theme=cyber) | Inspect a compact crop within a sample desktop. |

Work and provider events are simulated. Town creation, layouts, notes, tasks, and retirement are interactive in-memory state. They reset on reload. The compact layout previews a desktop composition without implementing a native overlay.

## Start the preview

From this repository, run:

```sh
./prototype
```

Open `http://localhost:5173/prototype/`. Press Ctrl+C in the terminal to stop the server.

For a fresh checkout, install the locked dependencies first:

```sh
npm ci
./prototype
```

The launcher selects this Mac's native Homebrew Node. It avoids the older Intel installation at `/usr/local/bin/node`. With a correctly configured Node installation, `npm run prototype` also works.

## Explore an adapting city

1. Open **Towns** under the city heading or select the current town in the header.
2. Enter a name and purpose under **Found a town**. Submit the form. The new town starts with its own resting residents and empty task queue.
3. Switch between the town cards. Notes, tasks, approvals, population, and appearance stage stay with their town.
4. Under **Workload-driven capacity**, select **Work arrives**. The demo creates two specialist homes and expands the annexes. The added tasks remain queued.
5. Select **Work settles**. The extra homes retire. Their notes and task history remain inspectable.
6. Select **Recall** to restore a retired resident with the same identity and memory.
7. Use **Expand** and **Consolidate** to inspect individual capacity changes. Consolidation first removes annexes, then retires an eligible idle home. It excludes the mayor and residents with unfinished work.
8. Select **Replan layout & generate paths**. Houses move to seeded plots and the pathfinder generates new routes around their footprints.

These controls preview responses to demand. They do not implement an autonomous provider scheduler or grant new execution permissions. Inactivity causes no decay penalty.

## Meet the Chief and residents

1. The Chief starts outside the sanctuary and stays visible. Click any open part of the world to teleport there. Click a house to appear at its entrance and open its small menu. Dragging rotates the view.
2. House menus show the resident's name, role, status, and current task. Choose **Look inside** to reveal the room and agent. **More details** opens memory, skills, history, equipment, and work controls. **Less detail** returns to the small menu.
3. Open **Chief** beneath the city heading or use the **C** button. The small owner menu opens decisions, the sanctuary cutaway, and teleport home. **More controls** or **Open Player menu** expands the owner controls.
4. The larger sanctuary stands on a separate terrace with a letterbox and arrival waystone. It has no city road connection. **Teleport home** returns the Chief to its terrace. Owner travel works while city work is paused or Quiet mode is active.
5. The Chief has a white coat and crown, a persistent ground ring, and a clickable location marker. If the Chief is outside the current camera view, the marker stays at the edge. Clicking it focuses the character. Full mobile inspectors keep the marker visible in their header.
6. Residents normally work or rest indoors. Select Sage in Cyberpunk or Tusk in Solo Leveling and choose **Look inside**. Nyx or Kaisel makes one sample trip to the mayor hub and returns.
7. In the expanded Chief menu, choose **Send a demo courier**. A robot carries a source index from the scout to the reader through the mayor hub. **Activity** records its progress and delivery. Replanning or switching themes interrupts a visible delivery in this throwaway scene.
8. Occasional speech bubbles describe an activity or a character quirk. These are scripted and require no model calls. Turn **Resident chatter** off in Settings, or use Quiet mode.

Solo Leveling uses Igris as mayor, Kaisel as scout, Tusk as reader, Iron as analyst, Shadow Knight 01 as writer, Tank as editor, and Beru as critic. New specialists use distinct numbered Shadow Knight, Mage, and Assassin names. These are theme aliases; IDs, tasks, notes, and town history are unchanged when switching to Cyberpunk.

Solo Leveling uses the owner's installed Friend or Foe Tall BB (with bundled Abel as a fallback), and translucent System windows based on the supplied menu references. Its illuminated rails, inset labels, geometric linework, framed actions, status tabs, and notifications differ from the Cyberpunk red frames, cyan selections, angular controls, and condensed Rajdhani lettering. Orbitron is reserved for secondary metadata. Cyberpunk Skills also shows a hexagonal diagram of the resident's actual accepted procedures. The artwork is original; no extracted anime assets ship in the prototype. Friend or Foe Tall BB is resolved through the local font installation and its binary is not included.

## Review the Player menu and private sanctuary

Open [Solo Leveling](http://localhost:5173/prototype/?variant=A&theme=shadow) and select **Chief**, then **Open Player menu**. The Player window has a white outline figure and six working access slots for the letterbox, new goals, headquarters, visits, domains, and accepted outcomes. Counts come from the current prototype state. **View your sanctuary** focuses the grander residence. Cyberpunk has a gold-trimmed private penthouse; Solo Leveling has a silver sanctuary with a dome and four spires.

The Chief's terrace is physically separated from the city. No road is generated for it. Teleporting is the owner's way to visit any active home, including while city work is paused. The brief arrival rings respect Quiet mode.

Select a resident, choose **More details**, and open **Equipment** in Solo Leveling. The diagram matches the role's visual tool and attire. Memory, Skills, Home, and Connection slots open their existing views. Equipment is cosmetic and does not grant permissions. The Cyberpunk theme retains its own resident tabs and hexagonal skills menu.

Notifications now separate the icon and title inside silver frames, with brighter blue outer rails. Permission requests keep their exact scope and **Allow once** / **Deny** controls. Quest info uses the same window treatment. Work and decisions remain simulated.

## Review the Cyberpunk reference pass

Open [the Cyberpunk city](http://localhost:5173/prototype/?variant=A&theme=cyber). The revised city has layered street fronts, shop awnings, neon signs, service pipes, rooftop equipment, overhead cables, lanterns, and a denser background skyline. Indoor cutaways have backlit wall panels and workstation details.

The menus use red outlines and labels, cyan selected controls, notched buttons, and framed OFF/ON toggles based on the supplied game menu references. Select Atlas, choose **More details**, then **Skills**, to inspect the hexagonal procedure display. It shows accepted skills without inventing game levels or additional permissions.

For the night lighting study, open **Settings** and choose **Fixed light** with the night preset. Return to **Local time** afterward. The default follows your computer clock. Two unshadowed colored lights provide the street illumination. Signs use small canvas textures, and the rest is procedural geometry. No new runtime library or downloaded image asset was added for this pass.

## Watch time and pause

1. Select **Pause city**. Residents and couriers hold their current positions. **Continue city** resumes movement. The Chief can still teleport while work is paused.
2. Open **Settings**. Local time is the default. **90s preview** runs a full day in 90 seconds, including gradual dawn and dusk. The clock continues while work is paused.
3. Choose **Fixed light** for manual art inspection. Return to **Local time** for normal use.
4. Enable **Quiet mode** to stop walking, pose animation, and chatter. User changes and coarse clock updates still redraw. Indoor work without a cutaway uses coarse updates for optional chatter rather than a continuous animation loop.

## Try an assignment

1. Select a resident and open **Memory**. Save a note, then switch themes to confirm that the same resident retains it.
2. Open **New goal**, enter an outcome, and select a resident. Submit the form to add a queued demo task.
3. Open the resident's **Work** tab. Select **Run demo task** to activate that resident's simulated work animation.
4. In the starter town, open **Approvals** and inspect the exact sample scope. Select **Allow once** or **Deny**.
5. Open the result after the sample review finishes. Owner acceptance remains separate from the simulated task completion.

Arrow keys change layouts unless a form field or dialog has focus. Escape closes a dialog or inspector. The roster provides DOM controls for canvas selections.

## Check the source

```sh
npm run build
```

This checks TypeScript and builds the static frontend. The 3D revision adds one runtime library, Three.js. Geometry and characters are procedural, and repeated static primitives are instanced. Open fonts are bundled locally; Friend or Foe Tall BB uses the local system installation. No model, image, font-service, or analytics requests run in the application.

The production build hides the design switcher. Use the development preview for layout review.

Read [the prototype status](docs/PROTOTYPE-STATUS.md) for observed checks and pending release gates. The [supplied specification](docs/reference/MASTER-SPEC.md) remains the full-build reference, subject to the owner's Agent City naming and adaptation corrections.
