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

1. Open **Chief** beneath the city heading or use the **C** button on the left rail. Your home has a letterbox, a travel beacon, and a distinct white-coated character.
2. Open requests from the letterbox, give the mayor a goal, or manage towns and capacity.
3. Choose **Look inside headquarters**. The outer building hides to reveal your character and desk. **Close cutaway** restores the building.
4. Choose a resident under **Your home & visits**, or open a resident and select **Visit as Chief**. Your character follows the generated access roads. Pause holds a visit in place. Return home sends the Chief back inside.
5. Residents normally work or rest indoors. Select Sage and choose **Look inside Sage's home** to see the reading workspace. Nyx makes one sample trip to the mayor hub and returns, rather than pacing indefinitely.
6. From the Chief panel, choose **Send a demo courier**. A small robot carries a source index from Nyx to Sage through the mayor hub. **Activity** records its progress and delivery. Replanning or switching themes interrupts a visible delivery in this throwaway scene.
7. Occasional speech bubbles describe an activity or a character quirk. These are scripted, require no model calls, and show at most two lines at once. Turn **Resident chatter** off in Settings, or use Quiet mode.

Solo Leveling uses Abel, a locally bundled narrow sans-serif font, and translucent System windows based on the supplied menu references. Its illuminated rails, inset labels, geometric linework, framed actions, status tabs, and notifications differ from the Cyberpunk red frames, cyan selections, angular controls, and condensed Rajdhani lettering. Cyberpunk Skills also shows a hexagonal diagram of the resident's actual accepted procedures. The font and artwork are original choices for this prototype, not extracted anime assets.

## Review the Cyberpunk reference pass

Open [the Cyberpunk city](http://localhost:5173/prototype/?variant=A&theme=cyber). The revised city has layered street fronts, shop awnings, neon signs, service pipes, rooftop equipment, overhead cables, lanterns, and a denser background skyline. Indoor cutaways have backlit wall panels and workstation details.

The menus use red outlines and labels, cyan selected controls, notched buttons, and framed OFF/ON toggles based on the supplied game menu references. Select Atlas, then **Skills**, to inspect the hexagonal procedure display. It shows accepted skills without inventing game levels or additional permissions.

For the night lighting study, open **Settings** and choose **Fixed light** with the night preset. Return to **Local time** afterward. The default follows your computer clock. Two unshadowed colored lights provide the street illumination. Signs use small canvas textures, and the rest is procedural geometry. No new runtime library or downloaded image asset was added for this pass.

## Watch time and pause

1. Select **Pause city**. Residents, the Chief, and couriers hold their current positions. **Continue city** resumes movement.
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

This checks TypeScript and builds the static frontend. The 3D revision adds one runtime library, Three.js. Geometry and characters are procedural, and repeated static primitives are instanced. Fonts are bundled locally. No model, image, font-service, or analytics requests run in the application.

The production build hides the design switcher. Use the development preview for layout review.

Read [the prototype status](docs/PROTOTYPE-STATUS.md) for observed checks and pending release gates. The [supplied specification](docs/reference/MASTER-SPEC.md) remains the full-build reference, subject to the owner's Agent City naming and adaptation corrections.
