# Nodes pass resources forward

A node is something a user does. It takes resources in and gives resources out: a browser session, a recording, a clip, a file, a sound. Most nodes already work this way. This plan fixes the ones that don't:

- The browser session carries no data, so capture nodes walk the graph backwards to find their steps.
- Sign in once has no ports and is linked to Open browser by a typed name.
- 14 nodes claim to take any clip but only act on recordings.
- Port types leak into inputs they shouldn't reach, and the conversions between types are spread over several helpers.

It adds two resources, both built the same way: a list collected forward, one entry per node, run by the node at the end. **`browser-session`** holds the browser's steps, and Record take or Screenshot plays them. **`recording-style`** holds cursor, annotation and camera rules, and Auto zoom (or Apply style) applies them.

The workarounds that exist only because of the old shape are deleted, not kept as fallbacks: the backward walks, the hidden sign-in scheduling and retry, the guessed start of the video, Export's undeclared output, and the scattered input conversions.

Status: built on 2026-10-21.

## How to read this plan

- One change, built in the order of **Work**. Each step type-checks and passes the tests before the next one starts.
- Sections: **The problem**, **Fit with Opsis** (Already in place / Missing; every file, function and line was checked in the repo on 2026-10-21), **Design**, **Work**, **Risks**, **Later**, **Sources**.

## Conventions

These restate rules from `AGENTS.md`; when the two disagree, `AGENTS.md` wins.

- No comments anywhere. File names in kebab-case.
- `src/shared` stays runtime-neutral: zod schemas only, no Node imports.
- **No old formats.** Nothing has been released, so there is no migration for project files with the old ports or node types. Rewrite `examples/` with `scripts/write-examples.ts` and delete local test projects.
- **Replace, don't route around.** When the new shape makes a helper, flag or special case pointless, delete it in the same step. No compatibility paths, no "today's rule when the new one doesn't apply".
- No version bumps before release (`ENGINE_VERSION`, `PICTURE_VERSION`, `MIX_VERSION`). Clear the test projects' caches instead.
- Before writing a helper, search for one that already does the job; after deleting one, search for its remaining users.
- User-facing copy: browser nodes are "actions", sentence case, no library names.
- **Checks.** `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, the no-comments check from `AGENTS.md`, and `bun run glyphs:bundle` after touching `NODE_LOOKS`. App checks open Opsis windows on the user's screen, so ask before running them. Don't commit.

## The problem

### The browser session is an empty token

`sessionValueSchema` (`src/shared/schema/values.ts:71`) is `{ type: "session" }` with no fields, and nothing produces it. None of the 19 `browser.*` nodes has a `compile()`, so every browser edge carries `undefined` (`runCompile`, `src/shared/compile/compile.ts:87`). Every user of the chain walks backwards instead (`walkSessionChain`, `src/shared/graph.ts:179`):

| Where                                                                                                                       | Walks back for                                                                                                                                                                                                        |
| --------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `retryAfterSignIn` → `sessionChain` (`src/worker/executors/capture.ts:93`)                                                  | The steps a take replays                                                                                                                                                                                              |
| `loginDependencies` (`graph.ts:210`), used by `neededNodes` (`runner.ts:128`) and `RunState.dependencies` (`runner.ts:367`) | Whether a take needs a sign-in                                                                                                                                                                                        |
| `captureChain` (`src/renderer/editor/use-form-context.ts:50`)                                                               | The element picker's window size and start address. It searches _forward_ for a capture node, then walks back from it. With a fork it takes the first capture it finds; with none connected, the picker gets no size. |
| `runsBeforeFirstPage` (`graph.ts:414`)                                                                                      | The "reload before any page" warning on Run page script                                                                                                                                                               |

Because the chain is read as raw workflow nodes, the take also has to:

- Drop Open browser and disabled steps itself (`chain.slice(1).filter((n) => !n.disabled)`, `src/worker/capture/driver.ts:89`).
- Interpolate and parse every step's params again with its own copy of the variables (`step`, `take-steps.ts:67`; `TakeRun.vars`, `take-run.ts:84`).

### Sign in once is wired by name and handled by special cases

- `capture.loginOnce` has `inputs: {}` and `outputs: {}`. Its `secretName` is matched against the `storageState` text on Open browser.
- The runner adds it as a hidden dependency (`loginDependencies`), `runTargets` special-cases it (`graph.ts:164`), and it's the only `volatile` node (`NodeDef.volatile`, `runner.ts:392`, `:401`).
- The cache key reads a param called `storageState` on any node (`hashNodes`, `runner.ts:156`).
- **Expiry is handled across nodes.** Go to page throws `SignInExpiredError` when it lands on the sign-in page (`take-steps.ts:125`). The capture executor catches it through `TracedError.signInExpired` (`src/worker/engine/context.ts:58`) and `retryAfterSignIn` (`capture.ts:98`), finds the Sign in once node through `sessionAccess`, opens its window and records again.
- It opens its own window instead of using the session it belongs to, and its category is Record, though it records nothing.

### When recording starts is guessed

`setupLength` (`take-steps.ts:347`, called at `driver.ts:121`) counts as setup everything up to the first Go to page, plus the steps flagged `offCamera` and the non-timed Waits after it. Whether a node shows in the video depends on where it happens to sit. `offCamera` exists only for this guess.

### Fourteen nodes claim `clip → clip` but only act on recordings

They change `core/recording` items through `patchRecordings` (`src/shared/compile/clip.ts:539`) and do nothing to any other clip:

- `camera.manualZoom`, `camera.pan`, `camera.followCursor`, `camera.idleSpeedup`, `camera.speedRamp`, `camera.backdrop`, `camera.motionBlur`
- `cursor.style`, `cursor.clickRipple`, `cursor.keystrokeChips`, `cursor.spotlight`, `cursor.highlightBox`
- `text.tooltip`, `text.calloutArrow`

`camera.deviceFrame` mixes the two: Laptop and Phone wrap any clip, while Browser window only patches recordings.

### Port types and conversions

Port types are `PORT_TYPES` (`src/shared/schema/workflow.ts:5`); which outputs fit which inputs is `ACCEPTS` (`src/shared/nodes/ports.ts:6`).

- **`ACCEPTS` promises conversions that nothing does in one place.** An input that accepts another type receives the other type's value, and each node converts it, or doesn't:
  - `mediaInput` (`src/shared/nodes/media.ts:68`) turns `image` and `file` values into `media`. It's used by Music, Sound effect, Use as clip, the media nodes' `passMedia`, and the worker's `requireMedia` (`src/worker/executors/util.ts:34`).
  - `toTimeline` (`src/shared/nodes/output.ts:129`) turns a clip into a `timeline`.
  - `audio` inputs accept `media`, but Add sound reads only `audio` values, so a connected file is dropped silently. `gatherInputs` merges several edges only when all are `audio`.
- **Export has an undeclared output.** Export video, Export images and Export as code return a `timeline` value that no port declares (`passTimeline`). `compileWorkflow` picks it up by name to fill `specs` (`compile.ts:141`).
- **Unused or wrong types:**
  - `scene` is accepted by `clip`, but no port declares it.
  - `timeline` is only Export's input type, labelled "Clip".
  - `file` is the type of File info's details and Export as code's folder, and `media` inputs accept it, so a folder can be wired into Trim file.
  - `recording` can't reach the File tools, though a take is a video file.
  - The `transition` value (`values.ts:69`) is never produced.

## Fit with Opsis

Already in place:

- **Executors already receive compiled values.** `ResultStore.inputsFor` (`src/worker/engine/runner.ts:262`) falls back to compiled values for nodes that never run. Once browser nodes have a `compile()`, Record take and Screenshot receive the session value with no runner change.
- **Disabled nodes already pass their input through.** `bypass` (`compile.ts:74`) gives a disabled node's outputs the input of the same type. A disabled step therefore never enters the list, with no special handling.
- **Hashes already cover upstream.** `hashNodes` (`runner.ts:137`) hashes every node, virtual ones included, with its upstream hashes, so a take's key follows its steps. `signInStamp` (`src/worker/engine/file-stamps.ts:22`) already turns a saved sign-in into key material.
- **The edits a style makes already exist.** Each recording-only node's compile is one `patchRecordings` or `remapClip` call on `RecordingProps` (`src/shared/compile/recording.ts`): `cursor`, `cursorScale`, `cursorLook`, `ripple`, `keystrokes`, `annotations`, `extraBeats`, `backdrop`/`backdropChosen`, `motionBlur`, `chrome`/`windowStyle`/`frameTheme`/`addressBar`.
- **Templates build chains in one place.** `GraphBuilder.chain` and `recordTake` (`src/shared/templates/build.ts:83`) link steps and connect Record take to Auto zoom. Templates use `cursor.keystrokeChips`, `cursor.spotlight`, `cursor.highlightBox` and `text.tooltip`, and link Export's `timeline` port (`src/shared/templates/index.ts:23`, `src/shared/test-fixtures.ts:68`).

Missing:

- Data on the session edge, and a `compile()` on browser nodes.
- A Sign in node in the chain that runs like any other node, and a step that loads saved cookies and page storage mid-take.
- An explicit start for the video.
- A `recording-style` resource, the recording's own style entries, and Auto zoom applying them.
- One place that converts an output into the input type it's connected to.

## Design

### Resources

| Type                              | Port id            | Label     | Value                                | Accepts                       |
| --------------------------------- | ------------------ | --------- | ------------------------------------ | ----------------------------- |
| `browser-session` (was `session`) | `session`          | Browser   | `{ browser, steps }`                 | `browser-session`             |
| `recording`                       | `recording`        | Recording | as today, plus `style: StyleEntry[]` | `recording`                   |
| `recording-style` (new)           | `style`            | Style     | `{ entries: StyleEntry[] }`          | `recording-style`             |
| `clip`                            | `clip`             | Clip      | as today                             | `clip`                        |
| `media`                           | `media`            | File      | as today                             | `media`, `image`, `recording` |
| `image`                           | `image`            | Image     | as today                             | `image`                       |
| `audio`                           | `sounds` / `audio` | Sound     | as today                             | `audio`                       |
| `captions`                        | `captions`         | Captions  | as today                             | `captions`                    |
| `file-info` (was `file`)          | `info`             | Info      | `{ src }`                            | `file-info`                   |
| `folder` (was `file`)             | `folder`           | Folder    | `{ src }`                            | `folder`                      |

- A port's id is its resource's short name, as `clip`, `recording` and `media` already are. Export's input becomes `clip`.
- `scene` and `timeline` leave `PORT_TYPES`. `timeline`, `transition` and `session` leave `CompileValue`.
- `PORT_WORDS` gains "browser session" and "style"; mismatch messages read from it.
- `portTypeLabel` drops the "Previous"/"Next" special case: ports show the resource's name.
- `audio` stops accepting `media`. A file becomes a sound through Music or Sound effect, which own its volume and timing.

### Conversions in one place

`ACCEPTS` becomes a table of conversions next to it in `ports.ts`:

```ts
const CONVERT: {
  [T in PortType]?: Partial<Record<PortType, (v: CompileValue) => CompileValue | undefined>>;
} = {
  media: { image: imageAsMedia, recording: recordingAsMedia },
};
```

- `canConnect` reads the table. `convertInput(value, spec)` returns the value as the port's own type.
- `gatherInputs` and `ResultStore.inputsFor` both call `convertInput`, so every node receives exactly the type its port declares.
- `gatherInputs` merges every value on a `many` port. Today only Add sound's Sounds is `many`.
- Only lossless, settings-free conversions go in the table: an image is a media file, and a recording's video is one. Anything that needs choices (fit, length, volume, start) is a node.
- `mediaInput` is deleted. Music, Sound effect, Use as clip, `passMedia` and `requireMedia` read `asValue(input, "media")`.

### Specs for exports

`compileWorkflow` fills `specs[id]` for output nodes from their `clip` input (`clipToSpec`). Export nodes' `compile()` returns only their own results. `toTimeline` and `passTimeline` are deleted, and `specFor` (`src/worker/executors/film-spec.ts:11`) is unchanged.

### The browser session

```ts
const takeStepSchema = z.object({
  id: z.string(),
  type: z.string(),
  label: z.string().optional(),
  params: z.record(z.string(), z.unknown()),
});

const browserSessionValueSchema = z.object({
  type: z.literal("browser-session"),
  browser: z.object({ nodeId: z.string(), params: z.record(z.string(), z.unknown()) }),
  steps: z.array(takeStepSchema),
});
```

- **Open browser** (`browser.open`, was `browser.session`). `compile()` returns `{ browser: { nodeId, params }, steps: [] }` with its parsed params. It keeps size, sharpness, device, identity, language, time zone, theme and recording sharpness, and loses `storageState`.
- **Every other browser node** gets `compile: appendStep` from one helper in `browser.ts`: `{ ...input, steps: [...input.steps, { id, type, label, params }] }`, where `params` are the node's parsed, interpolated params from its compile context. With no input it returns nothing, so the node shows the usual missing-input warning.
- **Record take and Screenshot** read `inputs.session`.
  - `openSession` takes `browser.params`.
  - The take runs `steps` as given: no `slice(1)`, no disabled filter.
  - Each step parses `entry.params` with its node's schema, without interpolating again.
  - `TakeOptions.chain`, `TakeOptions.vars` and `TakeRun.vars` go when nothing else reads them.
- **Deleted:** `sessionChain`, `walkSessionChain`, `captureChain`, and the "doesn't start from Open browser" graph issue (a missing input says it).
- **The picker** reads the compiled value at the node's own input: `browser.params` for the window size, the last Go to page in `steps` for the address. It works with no capture connected and with forks.
- **`runsBeforeFirstPage`** reads the input's `steps`.

### Sign in

`browser.signIn` (was `capture.loginOnce`), category Browser, `browser-session → browser-session`. It's an ordinary executing node in the chain.

- **Params** don't change: `url`, `secretName` ("Save as"), `doneWhen`, `selector`, `timeoutSec`. `signInClashes` stays and checks the new type.
- **`compile()`** appends its step, like every browser node.
- **Executor:** when the sign-in named `secretName` is saved, it returns at once. Otherwise it opens the window with the session's size (`LoginRequest` gains the viewport; `src/main/browser-windows.ts` uses it) and saves the result.
- **Caching:** the runner schedules it as the upstream of the take, like any node. `NodeDef` gains `secretParams?: string[]` (here `["secretName"]`). `hashNodes` adds `signInStamp` for each param named there, in place of reading `storageState`. A new sign-in changes the Sign in node's hash, and through upstream hashes every take after it.
- **In the take:** a new step in `STEPS` (`take-steps.ts`) reads the saved state:
  - `context.addCookies(state.cookies)`.
  - For each origin, an init script that writes its `localStorage` when a page of that origin loads, plus a direct write if the current page already is that origin.
- **Expiry:** Go to page still notices when it lands on the Sign in step's address. It fails the take with "Signed out during the take. Run “<Sign in node name>” again." The Run panel already links failed nodes. There's no automatic retry across nodes.
- **Deleted:**
  - `loginDependencies`, `loginOnceExecutor`, the `runTargets` special case and `NodeDef.volatile` (Sign in once is its only user).
  - `sessionAccess`, `retryAfterSignIn`, `TracedError.signInExpired` and `SignInExpiredError`'s catch path.
  - `TakeOptions.storageState`, `TakeOptions.signIn`, the `storageState` option of `openSession`, and the `storageState` check in `hashNodes`.

### Start recording

`browser.startRecording`, `browser-session → browser-session`, no params. Card: "Video starts here".

- The video starts at the first Start recording step. Steps before it run off camera: sign-ins, page setup, the first page load.
- With no Start recording, the video starts with the first step. Record take's card says so in a muted row ("Starts with the first action").
- `setupLength` and the `offCamera` flag are deleted.

### Recording style

```ts
const styleEntrySchema = z.object({
  nodeId: z.string(),
  type: z.string(),
  params: z.record(z.string(), z.unknown()),
});
```

Each node that edits a recording exports two pure functions next to its definition, built from its current compile body:

- `restyle(props, log, params)` returns `RecordingProps`.
- An optional `retime(clip, params, ctx)` returns `ClipValue`, for Skip quiet parts and Speed ramp.

`STYLE_STEPS` maps node type to these functions, the same pattern as `STEPS` in `take-steps.ts`.

**Style nodes: `recording-style? → recording-style`.** The input is optional, so the first node starts the style.

- `cursor.style`, `cursor.clickRipple`, `cursor.keystrokeChips`, `cursor.spotlight`, `cursor.highlightBox`
- `text.tooltip`, `text.calloutArrow`
- `camera.followCursor`, `camera.idleSpeedup`, `camera.backdrop`, `camera.motionBlur`
- The new `camera.windowFrame`

Their `compile()` appends `{ nodeId, type, params }` to `entries`. A `styleModifier` base sits next to `modifierBase` in `define.ts`.

**Per-take nodes: `recording → recording`.** These need one take's frames and times, and the spot picker shows a frame of the recording upstream (`spotBackground`, `use-form-context.ts:38`):

- `camera.manualZoom`, `camera.pan`, `camera.speedRamp`

Their `compile()` appends to the recording value's own `style`. A `recordingModifier` base: `recording` in and out.

**Auto zoom: `recording + recording-style? → clip`.**

1. Builds `RecordingProps` from its own fields, as today.
2. Applies the connected style's entries in order, then the recording's own entries.
3. Builds the clip, then runs each entry's `retime` in the same order.

Settings are applied in that order, so the most specific one wins: Auto zoom's fields < the style < the take's own nodes. Within one list, later beats earlier. `camera.backdrop` keeps setting `backdropChosen`, which makes a chosen backdrop beat the camera style's own ground (`src/video/camera/demo-clip.tsx:227`).

**Apply style: `clip + recording-style → clip`** (`camera.applyStyle`). It runs `restyle` on every recording in the clip through `patchRecordings`, then `retime`. This is how a style reaches several takes after Sequence or Layers. Card: "No recording in this clip" (warn) when there's none.

**Window frame** (`camera.windowFrame`, style node): style, theme and address bar, with the same fields and `hostDefaults` as Device frame's Browser window. Device frame keeps Laptop and Phone only.

**Cards.** `card-rows.tsx` builds `inputTakes` from `recording` values as well as clips, so per-take nodes keep `takeRow`. Style nodes have no take, so Auto zoom's card warns when its style holds Follow cursor and the take has no cursor track.

### Other port fixes

- **File info:** output `info: file-info`.
- **Export as code:** output `folder: folder`.
- **Export video and Export images:** input `clip: clip`.

### Node types and looks

New or renamed types: `browser.open`, `browser.signIn`, `browser.startRecording`, `camera.windowFrame`, `camera.applyStyle`.

- Each needs a `NODE_LOOKS` entry: Sign in keeps `lucide:key` with `twist`; the others get a stroke-only glyph and no motion. Then `bun run glyphs:bundle`.
- Each needs a `description` and a `card()`.
- `CATEGORY_LABELS` don't change.

## Work

1. **Port types and conversions.**
   - `PORT_TYPES` and `PORT_WORDS` (`workflow.ts`); `ACCEPTS` → `CONVERT`, `canConnect`, `convertInput` and `portTypeLabel` (`ports.ts`).
   - `gatherInputs` and `ResultStore.inputsFor` convert and merge.
   - `CompileValue` (`values.ts`): add `browserSessionValueSchema`, `recordingStyleValueSchema` and `style` on `recordingValueSchema`; remove `session`, `transition` and `timeline`.
   - `compileWorkflow` fills `specs` from output nodes' `clip` input.
   - The Export, File info and Export as code ports.
   - Delete `mediaInput`, `toTimeline` and `passTimeline`.
   - Tests:
     - `ports.test.ts`: the new table, and that `audio` refuses `media`.
     - A compile test where a Screenshot feeds Trim file and a Record take feeds Convert.
     - Add sound with two Sound effects connected.
     - An Export's spec from its `clip` input.
2. **Session value.**
   - `browser.open`, and the `appendStep` helper on every `browser.*` node.
   - Record take and Screenshot read their input; `openSession` takes `browser.params`; steps parse without interpolating.
   - Replace `captureChain` and `runsBeforeFirstPage`.
   - Delete `sessionChain`, `walkSessionChain`, the driver's slice and filter, and the take's `vars` if unused.
   - Tests:
     - Compile a chain and check `steps` order.
     - A disabled step is missing from `steps`.
     - A fork gives two independent lists.
     - `capture.test.ts` builds takes from the value.
3. **Sign in.**
   - Move the node to `browser.ts` as `browser.signIn`, with its executor; add the cookie and storage step.
   - `secretParams` and the `hashNodes` stamp; size in `LoginRequest`; the expiry error naming the node.
   - Delete everything listed under **Sign in → Deleted**.
   - Tests:
     - Runner: a take after an unsaved Sign in runs Sign in first.
     - A saved one opens no window.
     - A new sign-in changes the take's hash.
     - Landing on the sign-in page fails the take with the node's name.
4. **Start recording.**
   - Node and card; the driver starts the encoder at the first Start recording step, or at the first step.
   - Delete `setupLength` and `offCamera`.
   - Test: steps before Start recording are outside the take's video.
5. **Recording style.**
   - Extract `restyle`/`retime` from the 14 compile bodies into `STYLE_STEPS`.
   - `styleModifier` and `recordingModifier` bases; Auto zoom's style input and apply order; `camera.applyStyle`; `camera.windowFrame`; Device frame without Browser window.
   - `inputTakes` for recordings.
   - Tests:
     - Each style node gives the same `RecordingProps` through Auto zoom as it did after Auto zoom before this step.
     - Precedence: Auto zoom backdrop < style backdrop < take's own.
     - Apply style on a Sequence of two takes styles both.
     - A Skip quiet parts plus Speed ramp case compared with today's output.
6. **Templates and examples.**
   - `GraphBuilder.recordTake` takes an optional style and inserts Start recording after the first Go to page and its waits.
   - Templates move Keystroke chips, Spotlight, Highlight box and Tooltip in front of Auto zoom as a style chain, and link Export's `clip` port.
   - Update `test-fixtures.ts`.
   - `bun run examples`; delete local test projects.
7. **Looks, copy, docs.**
   - `NODE_LOOKS` and `glyphs:bundle`; `card.test.ts` (descriptions, banned words); `icon-name.test.ts`; `catalog.test.ts`.
   - `README.md` (Record section, node count).
   - `AGENTS.md`: "Browser nodes are virtual", "Interactive browsing" (Login once paragraph), "Node cards", "Ports that take several connections"; add a line for the conversion table.
8. **Checks.** Everything in **Conventions**, plus searches for the deleted names (`sessionChain`, `loginDependencies`, `mediaInput`, `setupLength`, `offCamera`, `volatile`, `storageState`, `signInExpired`, `"timeline"` as a port). With the user's go-ahead: run the to-do sample with a Sign in step, record, export, and use the element picker on a node with no Record take connected.

## Risks

- **No automatic re-sign-in.** A sign-in that expires mid-run now fails the take with the Sign in node's name, instead of opening the window and recording again. That's one extra click, in exchange for no capture code that knows about Sign in.
- **Recordings start earlier without Start recording.** Projects and templates without it record the first page load. Templates add it; the Record take card says where the video starts.
- **Mid-take sign-in.** Cookies can be added at any time, but `localStorage` only reaches pages loaded after the init script, or the current page if it is that origin. `sessionStorage` and IndexedDB aren't carried, as today. The step's card says it applies from the next page load.
- **Order of two remaps.** Skip quiet parts (style) now runs before Speed ramp (per take), whatever their order on the canvas used to be. `remapClip` maps recording time through `clipFrameOfRecording`, which reads the recording item's own speed but not an outer remap. Compare today's output in step 5 before relying on the combination.
- **Card stills and pictures.** Clips are built from the same props, so stills and exported pictures should match. Any difference shows in the step 5 equivalence tests.
- **Style without a take.** Style nodes can't show take-specific warnings (no cursor track, a time past the end). Auto zoom and Apply style carry those warnings instead.

## Later

- Give Record take and Screenshot a `browser-session` output, so a chain can continue after a capture. Each capture still launches its own browser and replays its list; this only changes which list it receives.
- A picker of actual actions on Tooltip, Callout arrow and the cursor annotations, next to name matching. A picked action is a node id, so in a shared style it only fires in the take that contains it.
- Other sources (a terminal, an app window) as their own resource types, each with its own start node and capture.

## Sources

- `src/shared/schema/workflow.ts` (`PORT_TYPES`, `PORT_WORDS`), `src/shared/schema/values.ts` (`CompileValue`, `recordingValueSchema`, `sessionValueSchema`, `transitionValueSchema`, `timelineValueSchema`).
- `src/shared/nodes/ports.ts` (`ACCEPTS`, `canConnect`, `portTypeLabel`, `SESSION_PORT`), `src/shared/nodes/define.ts` (`modifierBase`, `NodeDef.volatile`, `virtual`).
- `src/shared/nodes/browser.ts`, `src/shared/nodes/capture.ts`, `src/shared/nodes/camera.ts`, `src/shared/nodes/cursor.ts` (`annotationNode`), `src/shared/nodes/text.ts`, `src/shared/nodes/audio.ts`, `src/shared/nodes/media.ts` (`mediaInput`, `passMedia`, `probeNode`), `src/shared/nodes/output.ts` (`toTimeline`, `passTimeline`), `src/shared/nodes/looks.ts`, `src/shared/nodes/card.ts` (`InputTake`).
- `src/shared/compile/compile.ts` (`gatherInputs`, `bypass`, `runCompile`, `compileWorkflow`), `src/shared/compile/clip.ts` (`patchRecordings`, `wrapClip`, `clipToSpec`), `src/shared/compile/recording.ts` (`recordingPropsSchema`, `defaultRecordingProps`).
- `src/shared/graph.ts` (`walkSessionChain`, `sessionChain`, `loginDependencies`, `runTargets`, `signInClashes`, `runsBeforeFirstPage`).
- `src/worker/engine/runner.ts` (`neededNodes`, `hashNodes`, `ResultStore.inputsFor`, `RunState.dependencies`), `src/worker/engine/file-stamps.ts` (`signInStamp`), `src/worker/engine/context.ts` (`TracedError`).
- `src/worker/executors/capture.ts` (`sessionAccess`, `retryAfterSignIn`, `loginOnceExecutor`), `src/worker/executors/util.ts` (`requireMedia`), `src/worker/executors/film-spec.ts` (`specFor`).
- `src/worker/capture/driver.ts`, `src/worker/capture/take-steps.ts` (`STEPS`, `step`, `setupLength`, `SignInExpiredError`), `src/worker/capture/take-run.ts` (`TakeOptions`, `TakeRun.vars`), `src/worker/capture/session.ts` (`openSession`).
- `src/main/browser-windows.ts`, `src/main/login-state.ts`, `src/renderer/editor/use-form-context.ts` (`captureChain`, `viewportOf`, `pageUrlBefore`, `spotBackground`), `src/renderer/editor/card-rows.tsx`.
- `src/shared/templates/build.ts` (`chain`, `recordTake`), `src/shared/templates/index.ts`, `src/shared/test-fixtures.ts`, `scripts/write-examples.ts`, `src/shared/api/catalog.test.ts`.
- `src/video/camera/demo-clip.tsx` (`backdropChosen`).
