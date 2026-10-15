# Review the Agent Town prototype

Open [the town preview](http://localhost:5173/prototype/?variant=A&theme=cyber). Use the bottom switcher or the left and right arrow keys to compare three layouts. Use the header to switch between Cyberpunk and Shadow domain.

| Layout | Preview | Main interaction |
| --- | --- | --- |
| A | [Town first](http://localhost:5173/prototype/?variant=A&theme=cyber) | Explore the settlement and open contextual inspectors. |
| B | [Command room](http://localhost:5173/prototype/?variant=B&theme=cyber) | Start with a goal and keep the town alongside the plan. |
| C | [Desktop companion](http://localhost:5173/prototype/?variant=C&theme=cyber) | Inspect a small town overlay within a sample desktop. |

All tasks, provider events, memory, approvals, and progress are simulated. Changes stay in memory and reset on reload. The compact layout previews composition in a browser. It does not implement a native overlay.

## Start the preview

From this repository, run:

```sh
./prototype
```

Open `http://localhost:5173/prototype/`. Keep the terminal running. Press Ctrl+C to stop the server.

For a fresh checkout, install the locked development dependencies first:

```sh
npm ci
./prototype
```

The launcher selects the native Homebrew Node on this Mac. This avoids the older Intel Node at `/usr/local/bin/node`. On a correctly configured Node installation, `npm run prototype` also works.

## Try the work flow

1. Select a home or a resident in the bottom roster. Inspect Work, Memory, Skills, and History.
2. Open **New goal**, enter an outcome, and select a resident. Submit the form to add a queued demo task.
3. Add a note in the resident's **Memory** tab. Switch themes and inspect the same resident to confirm that the note remains.
4. Use **Pause town**. The demo shows Pausing before acknowledging Paused. Use **Continue town** to restore the prior activities.
5. Open **Approvals**. Inspect the exact file scope and select **Allow once**.
6. Open **View result** after the sample review finishes. Select **Accept sample outcome**. The third accepted sample outcome adds archive annexes to the settlement.
7. Open **Accepted outcomes** to inspect the sample ledger and preview all three appearance stages.
8. Use **Replay demo** to reset the sample goal and replay finite message couriers. Assigned tasks and note edits remain in memory.
9. Open **Settings** to enable Quiet mode. Use the map controls to change lighting, pan the canvas, or zoom.

Arrow keys change layouts unless a form field or dialog has focus. Escape closes a dialog or inspector. The bottom roster and navigation provide DOM controls for the canvas interactions.

## Check the source

```sh
npm run build
```

This checks TypeScript and builds the static frontend. Vite and TypeScript are development dependencies. The application has no runtime library dependencies. Fonts are bundled locally and require no remote requests.

The production build hides the design switcher. Use the development preview for layout review.

Read [the prototype status](docs/PROTOTYPE-STATUS.md) for verified behavior and pending release gates. The [supplied specification](docs/reference/MASTER-SPEC.md) remains the full-build reference. Approval of this prototype is pending.
