# Review the Agent City prototype

Open [Agent City](http://localhost:5173/prototype/?variant=A&theme=cyber). This is the revised design prototype. The production build awaits approval.

Use the header to switch between Cyberpunk and Solo Leveling. Drag the scene to orbit the real 3D geometry, scroll to zoom, or select a resident and choose **Follow in 3D**.

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

## Watch agents and time

1. Watch Nyx and Sage walk their generated routes, work at a destination, and return. Other residents have role-specific equipment and poses.
2. Select a resident and choose **Follow in 3D** for a closer view. Use the camera reset control to return to the city view.
3. Select **Pause city**. Agents hold their visible positions after the simulated interruption acknowledgement. **Continue city** resumes movement.
4. Open **Settings**. Local time is the default. **90s preview** runs a full day in 90 seconds, including gradual dawn and dusk. The clock continues while work is paused.
5. Choose **Fixed light** only for manual art inspection. Return to **Local time** for normal use.
6. Enable **Quiet mode** to stop walking and pose animation. The scene still redraws for user changes and coarse clock updates.

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
