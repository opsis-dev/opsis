# App gaps

The limits and bugs that the Shipyard launch-demo build ran into and that are still open, with the work to close each one. The demo's own plan was deleted once the film was built, so this file carries what it found: what users couldn't do, what the demo did instead, where the cause is in the code, and how to fix it.

Status: the first five sections are built and were checked in the app on 2026-10-17 (results under **Checked** in each). **Lost frames and frame rates** is open and comes next; **Known, not planned** stays as it is.

## How to read this plan

- Each section is one feature or one bug. The user picks which to build and when. A large one is split into numbered Work steps, each checked before the next starts.
- Feature sections have the parts **What it adds** (for users), **Fit with Demoflow** (Already in place / Missing), **Work**, **Risks**, and **Sources** where outside docs matter. Bug sections use **What happens** and **Cause** in place of the first two.
- **Size**: S is a day or less, M a few days, L a week or more.

## Conventions

These restate rules from `AGENTS.md`; when the two disagree, `AGENTS.md` wins.

- No comments of any kind; kebab-case file names; never hand-edit generated or registry files; dependencies only through `bun add` / `bun remove`.
- User-facing copy: no library, tool or format names; sentence case; durations in seconds; browser nodes are "actions"; elements are described in words. `src/shared/nodes/card.test.ts` checks descriptions and banned words.
- Nothing in `src/shared` or `src/video` imports `node:*`, `electron` or `playwright-core`. Animation in `src/video` stays a pure function of the frame.
- Bump `ENGINE_VERSION` (`src/worker/engine/cache.ts`) when an executor's output changes, so old takes record again.

**Checks when a section is done.**

- `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`.
- The no-comments check from `AGENTS.md`.
- Try it in the app on a real project, and compare the editor Preview with an export: preview must equal render.
- Don't commit or push.

## Summary

| Section                     | What users get                                                                                                                              | Size |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- | ---- |
| Drag action                 | A Drag action: drop an element on another, move it by a distance, or draw a path, with the cursor, camera, cards and sounds following along | L    |
| A transition at each cut    | One Sequence with a different transition (and length) before any clip, instead of chains of transition nodes                                | M    |
| Higher speed limits         | Change speed and Speed up to 64×, matching what Fit length already allows                                                                   | S    |
| Run panel bugs              | Progress that survives the editor reopening mid-run, and a later run that no longer hides the earlier run's errors and exports              | M    |
| Chapters in exported files  | Chapters from Chapter actions written into MP4, MOV and MKV exports, and a YouTube timestamp list next to the export                        | S    |
| Lost frames and frame rates | Recordings that keep every page change, and frame-rate settings that each do what they say (open)                                           | M    |
| Known, not planned          | Leftover render files in the system temp folder after a cancelled or crashed render                                                         | —    |

## Drag action

### What it adds

A new browser action, **Drag**, in three ways:

- **Drop it on an element**: pick up a card and drop it in a column, drag a node from a panel onto a canvas.
- **Move it by a distance**: drag a slider, resize a box, draw a rectangle or arrow in a whiteboard after choosing the tool.
- **Draw a path**: hold the button down along a list of points, for pens and free drawing.

The drawn cursor follows the drag, stays pressed while the button is down, and the camera frames the whole drag. The card shows what is picked up and where it goes, and the drop is an action mark on the Timeline, so a Sound effect set to **On an action** plays on the drop.

The demo couldn't build an n8n workflow by dragging nodes onto its canvas (act 3 ran an existing workflow), couldn't draw shapes in Excalidraw (act 1 loaded the sketch before the page opened and only added text), and couldn't drag the Twenty deal card into Won (act 6 changed the stage in the deal panel).

### Fit with Demoflow

Already in place:

- Every pointer action follows one path. The node is defined in `src/shared/nodes/browser.ts` (`hoverNode`, `clickNode`, listed in `browserNodes`), using `target()`, `beatFields` and `pointerFields` from `src/shared/nodes/params.ts`. `runTake` (`src/worker/capture/driver.ts`) handles it in `runStep` (`case "browser.hover"`), resolves elements with `resolveTarget` (which also accepts `"x, y"` page points), and calls the `FlowContext` built by `buildContext` (`src/worker/capture/recorder.ts`).
- In `buildContext`, `glideTo` moves the mouse every 8 ms along an eased curve (`cameraEase`), calling `mark()` for each sample, so the synthetic cursor track (`CursorSample`) already records any path the mouse takes. `logEvent` pushes a `ClickEvent` (`src/video/camera/lib/click-log.ts`); `clickToAction` in the driver turns it into an `Action` (`src/shared/schema/actions.ts`).
- On the video side, `toClickLog` (`src/video/camera/action-log.ts`) turns actions whose kind is in `POINTER_KINDS` (`src/shared/compile/recording.ts`) into `ClickEvent`s. `buildCameraTrack` (`src/video/camera/lib/zoom.ts`) zooms in from `tDepartMs`, frames `rect` with `frameFor` and holds until `tMs` plus the hold; `cursorAt` (`src/video/camera/lib/cursor.ts`) squashes the arrow and draws the ripple at `tDownMs`; `resolveAnnotations` puts highlight boxes and spotlights on `rect`.
- `flattenSpec` (`src/shared/timeline/flatten.ts`) makes an action mark at each action's `t`, named by `actionLabel`. The Sound effect's action list comes from `ACTION_TYPES` in `src/renderer/editor/inspector.tsx`.
- The element picker (`pickInWindow`, `PICKER_SCRIPT` in `src/main/browser-windows.ts`) fills any `target` widget field through its own Pick button, so a second element field on one node works as it is.
- Playwright 1.64.1 is installed: `page.mouse.down()` / `move(x, y, { steps })` / `up()` and `locator.dragTo(target, { sourcePosition, targetPosition, steps })`. In Chromium, Playwright turns a mouse drag over a `draggable` element into native drag and drop, so pages built on pointer events and pages built on drag-and-drop events both work with the same mouse calls.

Missing:

- No `"drag"` in `ACTION_KINDS`, no drag fields on `Action` or `ClickEvent`, no drag on `FlowContext`.
- `cursorAt` only knows a short press (`CLICK_SQUASH_S`); nothing holds the arrow pressed.
- The baked cursor in `CURSOR_INIT_SCRIPT` moves on `mousemove` only. During a native drag the page gets `dragover` instead, so the baked arrow would freeze.
- The picker returns elements only, never a point, and its window isn't the size of the take's browser (`PickRequest.viewport` exists but the inspector doesn't pass it, and `sandboxedWindow` sets the outer size).
- No `grab` or `spline` icon in `src/renderer/editor/icons.ts` (`move` is there). Both exist in `lucide-react` (`grab.mjs`, `spline.mjs`).

### Work

1. **Action log.** In `src/shared/schema/actions.ts`, add `"drag"` to `ACTION_KINDS` and two optional fields to `actionSchema`: `path` (array of `{ x, y }` page points, thinned to at most 64) and `drop` (`rectSchema`, the drop element or the path's bounds). For a drag, `tDepart` is when the cursor sets off toward the element, `tDown` the press, `t` the release (a click's `t` is also taken after the button comes up), `point` the release point and `rect` the element picked up. Add `"drag"` to `POINTER_KINDS`. Old logs parse unchanged.
2. **Click events.** Add `tUpMs`, `path` and `drop` (all optional) to `ClickEvent`. `clickToAction` copies them; `toClickLog` reads them back for drags:
   - If the union of `rect` and `drop` is at most half the window in each direction, make one event: `rect` = the union, `x`/`y` = its centre, `tDownMs`, `tUpMs` = `t`. The camera then zooms in before the pick-up and holds until after the drop, with both ends in frame.
   - Otherwise make two events in one zoom group (`cluster` = the action's group, or `drag:<action id>`): the pick-up (`tMs` = `tDown`, `rect`) and the drop (`tDepartMs` = `tDown`, `tMs` = `t`, `rect` = `drop`, `tUpMs` = `t`). `buildCameraTrack` already pans between them, or zooms out and back in when they are further apart than `MAX_PAN_PX`.
   - Name the half-window rule as a constant in `action-log.ts` and test both cases in `src/video/camera/action-log.test.ts`.
3. **Cursor.** In `cursorAt`, an event with `tUpMs` keeps `squash` at `CLICK_SQUASH` from `tDownMs` to `tUpMs` and springs back over `CLICK_SQUASH_S` after it. The ripple stays at the press only. Extend `src/video/camera/lib/cursor.test.ts`. In `CURSOR_INIT_SCRIPT`, also move the baked arrow on `dragover` (it carries `clientX` / `clientY`) and release it on `mouseup`, `drop` and `dragend`.
4. **Path walking.** Move the sampling loop out of `glideTo` into a helper that walks a list of points at an eased pace by distance along the line, calling `mark()` and `page.mouse.move()` every 8 ms. Put the pure part (points for the glide curve, the offset, a path) in a new `src/worker/capture/drag-path.ts` with its own test. Glides must move exactly as they do today; if any sample changes, bump `ENGINE_VERSION`.
5. **Drag in the recorder.** Add `drag(from, to, label, opts)` to `FlowContext` (`src/worker/capture/flow-types.ts`), with `to` as `{ target: Target } | { by: { x: number; y: number } } | { path: Array<{ x: number; y: number }> }`. In `buildContext`:
   - glide to the element (`boxOf`, `hoverMs` settle), note `tDownMs`, `page.mouse.down()`, wait `holdMs`;
   - move 6 px toward the drop first: many pages only start a drag past a small threshold;
   - measure the drop element again with `boxOf` after the press, because lists and columns move when an item is lifted;
   - walk to the drop centre (with the slight bulge glides use), by the offset, or along the path, over `dragMs` (or from the distance as `glideTo` does with `GLIDE_PX_PER_MS`, clamped to 0.6–2.5 s);
   - wait 120 ms over the drop so the page shows its drop highlight, `page.mouse.up()`, then `logEvent` with the drag fields.
   - Quick method: `asLocator(...)` on both ends and `locator.dragTo(target, { steps: 20 })`, timed around the call, with the cursor track written as a straight line from start to drop.
6. **Node.** `dragNode` in `src/shared/nodes/browser.ts`, type `browser.drag`, title "Drag", icon `grab`, listed after `hoverNode`. Description: "Drags an element onto another one, by a distance, or along a path, like moving a card or drawing a shape." Params:
   - `target`: `target("Element to drag", "Pick it on the page, type its visible text, or give a spot as across, down.")`
   - `mode`: `choice(["element", "by", "path"], "element", "Drag", { element: "Onto an element", by: "By a distance", path: "Along a path" })`
   - `to`: `optionalTarget("Drop on")`, shown for `element`
   - `dx`, `dy`: integers in px, "Across" and "Down", shown for `by`
   - `path`: textarea, "Path", help "One point per line, as across, down in pixels from where the drag starts.", with `presets` for Circle, Tick, Zigzag and Square, shown for `path`
   - `holdMs`: `ms(150, "Hold before moving", 5000)`; `dragMs`: `ms(0, "Drag time", 10_000)` with `zeroLabel: "Auto"`
   - `method` under `advanced`: "Smooth, with the cursor" (default) or "Quick, for pages that ignore the smooth drag"
   - `...beatFields`, `...pointerFields`
   - Card: `elementRow("grab", p.target, raw)`; then `elementRow("move", p.to, raw, "to", "Pick where to drop it")`, or `row("move", "By 300, 0 px")`, or `row("spline", "Along 12 points")`; `pause(p.after)`; `zoomRow(p)`.
7. **Driver.** `case "browser.drag"` in `runStep`, modelled on `browser.click` (resolve both targets with `resolveTarget`, pass `frame`, `zoomScale`, `travelMs`, `hoverMs`). Map `browser.drag` to `"drag"` in `clickToAction`. Parse `path` into points and fail with a plain message on a bad line ("Line 3 of the path isn't two numbers.").
8. **Icons, Timeline, sounds.** Add `grab` and `spline` to `ICONS` in `src/renderer/editor/icons.ts`. Add `case "drag"` to `actionLabel` ("Drag …"). Add `"browser.drag"` to `ACTION_TYPES` in `inspector.tsx`. The mark sits at `t`, the drop.
9. **Picker.** In `PICKER_SCRIPT`, Shift-click returns the spot as `"x, y"` (`clientX`, `clientY`) instead of an element, with the tag showing the numbers while Shift is held. `pickTarget` passes the take's Open browser size as `viewport`, and `pickInWindow` uses `useContentSize: true` so picked spots match the take. The toast becomes "A browser window opened. Click the element, or Shift-click a spot."
10. **Docs.** README: "19 browser actions", with drag in the list.
11. **Tests.** `card.test.ts` rows for the three modes; `drag-path.ts` (offset, path, glide unchanged); `action-log.test.ts` (one or two events); `zoom.test.ts` (a drag that fits gets one framing that contains both ends); `cursor.test.ts` (held squash).
12. **Check in the app** against local copies of the three demo apps: n8n (drag a node from the side panel onto the canvas, a native drag-and-drop page), Excalidraw (Press keys R, then Drag by 300, 200; pen tool with the Circle path) and Twenty (drag a deal card into Won). Each with the synthetic and the baked cursor, Auto zoom on, in 16:9 and 9:16, Preview against export.

### Risks

- **Pages that need a pause or a threshold.** Sortable lists often wait for a short hold or a few pixels of movement before a drag starts. `holdMs` and the 6 px nudge cover the common cases; the Quick method is the fallback.
- **The drop moves away.** Columns reflow and canvases pan while dragging. Measuring the drop after the press handles reflow; a page that scrolls itself near the edge can still miss. When the drop element isn't on screen after the press, fail with "The place to drop it isn't on screen. Scroll to it first, or use a bigger window."
- **Big drags.** A drag across the whole window can't be framed closely; the two-event rule pans instead. Check that the camera doesn't zoom out and in twice on a short, wide drag.
- **Quick method timing.** `dragTo` runs at its own pace, so the drawn cursor's straight line is an approximation. Keep it the non-default choice.
- **Canvas apps and iframes.** Points are page coordinates. A whiteboard inside an iframe, or one that pans its canvas between takes, needs an element start instead of a fixed spot.

### Sources

- [Playwright: `locator.dragTo`](https://playwright.dev/docs/api/class-locator#locator-drag-to) (`sourcePosition`, `targetPosition`, `steps`)
- [Playwright: Mouse](https://playwright.dev/docs/api/class-mouse) (`mouse.down`, `mouse.move` with `steps`, `mouse.up`)
- [Playwright: Drag and drop](https://playwright.dev/docs/input#drag-and-drop) (manual drags with `mouse`, drag-and-drop pages)
- Types of the installed `playwright-core@1.64.1` (`dragTo` options including `steps`; checked 2026-10-16)

### Checked

On local test pages instead of n8n, Excalidraw and Twenty (their images were no longer on this computer): a pointer-event kanban, a native drag-and-drop palette onto a canvas, and a drawing canvas with a rectangle tool, a pen and a range slider. With the synthetic and the baked cursor, Auto zoom on, 16:9 and 9:16: the webhook lands where it was dropped, the box is 360 × 220, the circle path is drawn, the stroke slider goes from 3 to 20, the baked cursor follows native drag-and-drop, and Preview matches the export. The kanban card drops on the page but the recording misses it: see **Lost frames and frame rates**.

## A transition at each cut

### What it adds

One Sequence node can use a different transition, with its own length and **Keep both clips whole**, before any of its clips. The Sequence's own transition stays the default for every other cut. The demo used a different transition between every act, which took a chain of transition nodes and nested Sequences; this makes it one node with a list.

### Fit with Demoflow

Already in place:

- `sequenceClips(nodeId, clips, transitions, opts)` (`src/shared/compile/clip.ts`) already takes one `TransitionRef | undefined` per clip, with `hold` per entry. Only `sequenceNode.compile` (`src/shared/nodes/timeline.ts`) fills every entry with the same `params.transition`.
- Incoming items carry `slot: { nodeId, port }`, so the Timeline knows which port (`in1`…`in6`) each clip came from.
- `flattenSpec` turns each item's `enter` into a `TransitionMark` with `kind` and `covers`; `timeline-view.tsx` draws them, warning when `covers > 0.4`.
- Object lists already have a form: `ObjectListField` in `src/renderer/editor/zod-form.tsx` renders `z.array(z.object(...))`, as Speed ramp's `segments` do (`src/shared/nodes/camera.ts`). The `transition` widget is an enum shown with `transitionTitle`.

Missing:

- No per-cut setting on the Sequence.
- `splitBar` (`src/shared/timeline/edits.ts`) only splits in place when the sequence's `transition` is `"none"`; otherwise it nests a new Sequence.
- `TransitionMark` doesn't say which node and port own it, and marks are `pointer-events-none`.

### Work

1. **Param.** Add `cuts` to `sequenceNode`: `z.array(z.object({ before, transition, transitionDuration, keepWhole })).default([])`, labelled "Different transitions", help "Use another transition before some clips. The others use the transition above."
   - `before`: `choice(["in2", "in3", "in4", "in5", "in6"], "in2", "Before", { in2: "Clip 2", … })`, keyed by port so connecting or disconnecting other clips doesn't move it
   - `transition`: `z.enum(TRANSITION_KINDS)` with `widget: "transition"`, including `none` for a plain cut
   - `transitionDuration`: `frames(15, "Length")`; `keepWhole`: `bool(false, "Keep both clips whole")`
2. **Compile.** For clip `i > 0`, use the last entry of `cuts` whose `before` is that clip's port, else the Sequence's own transition. An entry for a port with no clip is ignored and kept, so reconnecting the clip brings it back.
3. **Card.** Keep the default row. Add `row("arrow-right-left", "2 cuts use their own transition")` when `cuts` applies to connected clips. Run `coverRow` per cut with the two neighbouring clips from `inputFrames` (keyed by port) and show the worst.
4. **Timeline.** Add `slot` to `TransitionMark`, copied from the incoming item. In `timeline-view.tsx`, clicking a mark's label selects its node (Sequence or transition node); the hatched area stays inert so drags pass through. The tooltip names the clip it leads into.
5. **Split in place.** Let `splitBar` split inside any Sequence: shift the `before` of later cuts with the same `k + 1` mapping it applies to edges, and add `{ before: <new port>, transition: "none" }` so the two halves join with a plain cut. Keep the six-port limit.
6. **Saved projects.** No migration: a missing `cuts` parses to `[]` and compiles exactly as today. Projects that chain transition nodes keep working as they are.
7. **Docs.** `AGENTS.md` → **Transitions**: one sentence on per-cut transitions.
8. **Tests.**
   - `compile.test.ts`: per-cut kinds and lengths land on the right items' `enter`/`exit`; `hold` per cut; an entry for an empty port is ignored.
   - `edits.test.ts`: split in place in a sequence with a fade default keeps every other cut on its clip and adds a plain cut; rolling a cut with its own transition keeps it; `flattenSpec` reports each mark's `kind` and `slot`.
   - `card.test.ts`: the cut row and per-cut cover warning.

### Risks

- **Ports, not positions.** If a later change ever compacts ports, `before` must follow, the way `splitBar` shifts edges.
- **Transition settings.** A cut uses the transition's default look, as the Sequence does today. Transitions with settings of their own (Light leak cut's colour shift) still need their node; say so in the help text if users ask.

### Checked

In the app: a Sequence of four title clips with a fade by default, a held wipe before clip 3 and a plain cut before clip 4 exports at 8.5 s with the marks where planned; Preview and export match at the fade, the wipe and the cut. Clicking a mark's label selects its Sequence (the tooltip names the clip it leads into). Splitting clip 1 from the Timeline stays inside the Sequence, shifts the later cuts with their clips, joins the halves with a plain cut, and undoes in one step.

## Higher speed limits

### What it adds

**Change speed** (a file) goes past 8× and the Timeline **Speed** node past 16×, up to 64×. The demo's rewind gag wanted a faster run-through than Change speed's 8× and was squeezed to 4 s on the Timeline instead.

### Fit with Demoflow

Where the caps come from:

- **Only from the param schemas.** `media.speed` (`src/shared/nodes/media.ts`): `rate` `.max(8)`. `timeline.speed` (`src/shared/nodes/timeline.ts`): `.max(16)`. Also `.max(16)` on Skip quiet parts (`camera.idleSpeedup`) and Speed ramp segments (`camera.speedRamp`) in `src/shared/nodes/camera.ts`, and `Math.min(16, …)` twice in `speedBar` (`src/shared/timeline/edits.ts`) for speed drags on the Timeline.
- **Not from FFmpeg.** The Change speed executor (`speedExecutor`, `src/worker/executors/media.ts`) uses `setpts=PTS/<rate>`, which takes any factor, and `atempoChain`, which chains `atempo=2` stages for any rate. The pinned build (`FFMPEG_RELEASE = "b6.1.1"`, `src/engine/paths.ts`) is FFmpeg 6.1, whose `atempo` takes 0.5–100 per stage anyway.
- **Not from playback rate.** Speed doesn't use a media element's `playbackRate`: `timeline.speed` compiles to a `remap` on a `core/group`, which `Group` (`src/video/demo-timeline.tsx`) draws through `<Freeze frame={remapFrame(...)}>`. Skip quiet parts and Speed ramp use the same remap. Fit length's speed mode (`durationFitNode`) uses it with no cap at all, so rates above 16× already render today.

Cost at high rates:

- Preview: every frame of a remapped clip is a seek. Fast preview plays `recording.preview.mp4`, made with a keyframe every half second, so one seek decodes at most half a second of video however high the rate.
- Export: each frame decodes forward from the previous one, so total work is about the source length, the same as playing it at 1×.

### Work

1. **One limit.** Add `MAX_SPEED = 64` to `src/shared/nodes/params.ts` and use it in `media.speed`, `timeline.speed`, `camera.idleSpeedup`, `camera.speedRamp`, and both clamps in `speedBar`.
2. **Sound.** Export `atempoChain` and test it: 64 gives six 2× stages, 0.25 two 0.5× stages, 3 gives 2× then 1.5×.
3. **Measure.** A two-minute take through Speed at 16, 32 and 64: Preview in Fast and Exact, export time, and the output frame rate of Change speed (read with File info). Compare with Fit length at the same rate.
4. **Warn only if needed.** If Fast preview drops frames above some rate, add a muted card row on Speed: "Very fast: the Preview may skip frames. The export is smooth."
5. **Tests.** `edits.test.ts`: a speed drag can reach 64× and no further; `card.test.ts` rows unchanged.

### Risks

- **Sound under Speed.** A clip's sound passes through `wrapClip` unchanged, so it plays at normal speed under a sped-up picture. That's true at any rate today and isn't changed here.
- **Rates that do use playback rate.** Auto zoom's **Playback speed** (`speed`, max 4) and the Media file element's speed (`playbackRate`, max 4 in `src/shared/elements/core.ts`) feed a video's playback rate, which browsers only support in a limited range. Keep those at 4.

### Sources

- [FFmpeg filters: `setpts`](https://ffmpeg.org/ffmpeg-filters.html#setpts_002c-asetpts) and [`atempo`](https://ffmpeg.org/ffmpeg-filters.html#atempo)
- [Remotion: `<Freeze>`](https://www.remotion.dev/docs/freeze)

### Checked

A 117 s take through Speed at 16, 32 and 64× exports 220, 110 and 55 frames, the same lengths as Fit length; export time follows the output length (17.8, 8.7 and 5.6 s). Preview playback, measured in the app: real time at 1× and 2×; from 4× it advances about 1 s per 4 s and redraws 1 to 7 times a second, so it plays slowly rather than skipping frames. That was already true up to 16×. The Speed card says "Very fast: the Preview plays slowly. The export is smooth." from 4×. Change speed writes 25 fps files: see **Lost frames and frame rates**.

## Run panel bugs

Two bugs seen during the demo's full export runs (session `bdfad9b0`). Both are in how the editor keeps run state, not in the run itself: the worker finished every run correctly.

### Progress empties mid-run

**What happens.** 36 s into a full run the Run panel's status list went empty while the run kept going: the store read `statuses: {}` with `running: true`, where 21 s in it had read 65 cached, 2 done, 2 running and 6 queued. From then on only nodes that changed afterwards came back; at the end it showed 7 done and none of the 65 cached ones. The app log shows a dev reload of `src/renderer/routes/editor.tsx` just before.

**Cause** (found in the code). When the editor route opens a project, `EditorRoute` (`src/renderer/routes/editor.tsx`) calls `useRunStore.getState().seed(p.lastRun)`, which sets `statuses: {}`, then asks `window.demoflow.runStatus()` and, if a run is going, calls `start(runId)`, which sets `statuses: {}`, `progress: {}`, `errors: {}` and `running: true`. That is exactly the state seen. `runStatus` (`IPC.runStatus` in `src/main/main.ts`) returns only `runId` and `projectDir`, so there is nothing to restore. In the demo a dev reload remounted the route. In a packaged app the same happens whenever the editor opens the project during a run, for example after going to the start screen and opening the same project again (not yet tried).

**Work.**

1. **Shared reducer.** Move the event handling in the store's `apply` into a pure `applyRunEvent(state, event)` over a `RunProgress` type (`runId`, `statuses`, `progress`, `cached`, `errors`, `traces`) in a new `src/shared/run-progress.ts`. The store uses it.
2. **Main keeps a copy.** `emitRun` in `main.ts` applies each event to a `RunProgress` for the active run, started on `run:start` and dropped after `run:done`. `runStatus` returns it as `progress`; update its type in `src/shared/ipc.ts`.
3. **Resume, not restart.** Add `resume(progress)` to the store, and have `EditorRoute` call it in place of `start(runId)`.
4. **Tests.** `src/shared/run-progress.test.ts`: a sequence of events gives the expected statuses, progress and errors. `src/renderer/store/run.test.ts`: `seed` then `resume` keeps the statuses of nodes that finished before the editor reopened.
5. **Check.** In the app: start a long export, open the start screen, open the project again: finished nodes still show Done or Saved, the running one shows its progress.

### A later run hides the earlier run's results

**What happens.** A full run ended with "1 node(s) failed". Next, **Run this** on the Export as code node ran for 7 s and finished cleanly. After it the Run panel listed no error, its exports list held only the code folder, and the failed node couldn't be found anymore: a cached run of everything afterwards was clean.

**Cause** (found in the code; which node failed can't be recovered).

- `start` in `src/renderer/store/run.ts` clears `statuses`, `errors`, `progress` and `outputs` for every node at the start of any run, including a **Run this** run of one node. `run:done` then sets `outputs` to that run's own exports only (`collectOutputs` in `src/worker/engine/runner.ts` collects output nodes in the run), so the film, stills, GIF and ProRes from the full run disappear from the panel's list.
- `runWorkflow` keeps `failures` in `last-run.json` only for nodes that haven't succeeded since: `merged` deletes a node's failure as soon as it has a result. Export as code renders the sound mix first (`mixed`, the run logged "Sound: -14.4 LUFS"), so it reruns part of what's upstream of it; most likely the failed node was among them and succeeded the second time. That is correct, but nothing showed the earlier error before it was replaced.
- The message counts `failed`, which also holds nodes skipped because something before them failed, and doesn't name any node.

To reproduce: make a node fail in a full run (for example a Check text action that looks for missing text), then **Run this** on an export that doesn't depend on it. The failure disappears from the panel and the canvas, and the exports list shows only the last export's files.

**Work.**

1. **Clear only what runs.** `start` no longer clears per-node state. On `run:start`, clear `statuses`, `progress` and `errors` only for `e.nodeIds`. Nodes outside the run keep their status and error.
2. **Exports from all results.** Build the panel's exports list from `results` of every `output.*` node in the workflow, newest `finishedAt` first, instead of `outputs`, so a partial run adds or replaces only its own files. `outputs` stays for the finished-run notification.
3. **Name the failures.** Add `failedNodeIds` to the `run:done` event (`src/shared/schema/events.ts`), from `failures` in `runWorkflow`, and count failures and skips apart. The panel shows "Export film failed" or "2 nodes failed: Export film, GIF", each name selecting its node.
4. **Rows.** The panel's rows include any node with an error, so an error from an earlier run stays visible until that node runs again.
5. **Tests.** `run.test.ts`: after a full run where `b` fails, a run of `c` alone leaves `errors.b` and `statuses.b` as they were. A test of the exports list: a code-only run keeps the film's files listed.

### Checked

In the app: during a three-shape export, going to the start screen and opening the project again kept every finished node as Done and showed the export at 29%. A run where a Check text action failed ended with "**Record take** failed, 1 skipped" (the name selects the node); a **Run this** on the title card export afterwards kept that failure and the skip listed, and the outputs list showed the title card first, then the film's files.

## Chapters in exported files

### What it adds

Chapter actions (`browser.chapter`) mark sections of a take and can show a card in the video, but nothing reaches the exported file. With this, an export carries real chapters that players list (QuickTime, VLC and most desktop players), and a `.youtube-chapters.txt` with `0:00 Title` lines to paste into a YouTube description (YouTube reads chapters from the description, not the file).

### Fit with Demoflow

Already in place:

- The capture driver logs each Chapter action as an action-log entry with `kind: "chapter"` (`src/worker/capture/driver.ts`, `case "browser.chapter"`), and `flattenSpec` (`src/shared/timeline/flatten.ts`) already maps those entries to film frames for the Timeline's chapter marks.
- Exports finish through FFmpeg: the picture render plus `muxArgs` (`src/worker/render/picture.ts`).

Missing:

- Chapters in the export's metadata, and the timestamp file.

### Work

1. **Chapter times.** A pure `filmChapters(spec)` in `src/shared/timeline/` from the same mapping `flattenSpec` uses: chapter titles with film start times, sorted, the first forced to 0:00 when a film starts before its first chapter (YouTube requires that), and chapters under 10 seconds merged into the one before (YouTube's minimum). Unit-tested, including takes moved, trimmed and sped up on the Timeline.
2. **Metadata.** For MP4, MOV and MKV/WebM outputs, write an FFmpeg metadata file (`;FFMETADATA1` with `[CHAPTER]`, `TIMEBASE=1/1000`, `START`, `END`, `title`) and add it in the mux step (`-i chapters.txt -map_metadata 1 -map_chapters 1`), so a chapter change re-muxes without rendering the picture again. GIF and image exports get none.
3. **Timestamp file.** Next to each video export, `<name>.youtube-chapters.txt`, listed with the export's artifacts.
4. **Output setting.** On Export video: "Chapters in the file" (on by default when the film has chapters) and "YouTube chapter list".
5. **Check.** Export a film with three Chapter actions; `ffprobe -show_chapters` lists them at the right times in each shape; VLC shows them.

### Checked

A 44.3 s take with three Chapter actions exported as MP4 (16:9 and 9:16), WebM and GIF: the video files list "Plan the launch" at 0:00 (recorded at 0.706 s, moved to the start), "Build the page" at 14.437 s and "Ship it" at 28.611 s, exactly the recorded times; the GIF has none; each video has a `.youtube-chapters.txt` with `0:00`, `0:14`, `0:28`. Titles with `=`, `;` and `#` come back intact. Not checked: a player's chapter menu (VLC).

## Lost frames and frame rates

Open. Found while checking the Drag action and the speed limits.

### What happens

- **A recording can miss a page change.** In the kanban check the card was dropped (the trace's page snapshot after the release reads "pointerdown mousedown pointerup acme->won mouseup"), but the recording shows it lifted until the next page opened 0.8 s later, and 3 s later when the pause after the drag was raised.
- **Change speed writes 25 fps.** From a 30 fps source, at any rate including 1× (101 frames for 4 s on a test file).
- **Frame-rate settings nobody has checked:** projects at 60 fps; Record take at 25, 30 and 60 fps; Join forces 30 fps when the files differ, even in a 60 fps project.

### Cause

- **Lost frames:** the screencast's `onFrame` (`runTake`, `src/worker/capture/driver.ts`) awaits `encoder.push`, and Chrome sends the next frame only after the previous one is acknowledged. In the app the encoder stalled for about 1.1 s (frames painted at 4.38 to 4.45 s were handled at 5.47 s); the drop painted at 5.14 s, inside the stall, and nothing changed on the page afterwards, so no frame of it was ever sent. Outside the app (less load) the same take kept the frame.
- **Change speed:** `setpts` leaves the stream without a known frame rate, so FFmpeg falls back to 25 fps; `-r 30` keeps 61 frames for a 2× speed-up of the same file.

### Work

1. Take each frame at once and let the encoder catch up from its queue (`push` already chains), surfacing encoder errors at `finish`; re-run the kanban drag in the app.
2. Change speed keeps the source frame rate.
3. Check every frame-rate setting (60 fps project, 25/30/60 fps takes, Join in a 60 fps project) and keep only those that pass.

## Known, not planned

**Render leftovers in the system temp folder.** When a render is cancelled or the worker dies, the renderer's temporary folders (frames and downloaded assets that `renderTimeline` in `src/worker/render/render.ts` would normally clean up) stay in the system temp folder, because the library skips its cleanup on those paths. The user decided to ignore this. If it ever matters, the fix is to give each render its own folder under `~/.cache/demoflow-*` or the project's `.demoflow/cache/` and delete it on the next start.
