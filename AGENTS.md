# Working on Demoflow

Read `README.md` for what the app does and `PLAN.md` for the spec it implements. This file is about changing it.

## Hard rules

**No comments. Any kind. Anywhere.** No `//`, no `/* */`, no JSDoc, no file headers, no CSS comments, in source, tests, scripts and config. If something needs explaining, the name is wrong or the function is too big; constraints that span the codebase go in this file. The only allowed comment-shaped lines are functional directives: `oxlint-disable…`, `/// <reference …>`, `@ts-…`. Check:

```bash
rg -n '^\s*//|/\*|\*/' --glob '*.{ts,tsx,mts,css}' src scripts *.config.ts -g '!src/video/elements/**' -g '!src/video/lib/**' -g '!src/renderer/components/**' -g '!src/renderer/routeTree.gen.ts' | rg -v 'oxlint-|/// <reference|@ts-|\*\*/|/\*\*"'
```

**Never hand-edit installed or generated code.**

| Path                                                                                      | Owner                      | Regenerate with                                                       |
| ----------------------------------------------------------------------------------------- | -------------------------- | --------------------------------------------------------------------- |
| `src/video/elements/**`, `src/video/lib/**`                                               | Remocn / snapcn registries | `bun run registry:ingest` (list in `scripts/registry-ingest.list.ts`) |
| `src/shared/elements/elements.generated.ts`, `src/video/registry/components.generated.ts` | registry ingest            | `bun run registry:ingest -- --no-add`                                 |
| `src/renderer/components/ui/**`                                                           | shadcn (Base UI, nova)     | `bunx --bun shadcn@latest add <name>`                                 |
| `src/renderer/components/*.tsx`                                                           | React Flow UI              | `bunx --bun shadcn@latest add https://ui.reactflow.dev/<name>`        |
| `src/renderer/routeTree.gen.ts`                                                           | TanStack Router            | `bun run routes`                                                      |

**Dependencies go through the package manager.** `bun add`, `bun add -d`, `bun remove` only; never edit dependency blocks by hand. Always the latest release, except:

- every `remotion` / `@remotion/*` package is pinned to the same exact version (`bun add --exact`); mismatches break silently;
- `typescript` is pinned to `6.0.3` until TypeScript 7.1. Remotion's esbuild loader reads `tsconfig.json` through `typescript.sys`, which TS 7.0 does not expose, and the Remotion bundle then compiles with the classic JSX runtime. Do not patch around this; upgrade when 7.1 ships.

**Tooling:** bun is the package manager, Node/Electron the runtime. Every tool config is a `*.config.ts`. Dev scripts run on Node (`node scripts/x.ts`; `tsx` only when a script imports extensionless TypeScript).

**Verify with** `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`. The `:fix` variants fix and report the rest; do not run the non-fix variants first.

**Do not commit or push.** Leave changes in the working tree unless asked.

**Large files go to `~/.cache/demoflow-*`** (browser binaries, clones, renders); `/tmp` is for small scripts and screenshots only. Long-running processes start with `run_in_background`.

## Layout

```
src/main/      Electron main: window, menus, IPC, protocols, projects, secrets, worker host
src/preload/   contextBridge `window.demoflow`
src/worker/    utilityProcess: engine/, executors/, capture/ (Playwright), media/ (FFmpeg), render/ (Remotion)
src/engine/    Node-only: engine paths and installer (headless shell + FFmpeg), shared by main, worker, forge
src/traces/    Node-only: compressed, deduplicated Playwright trace store
src/shared/    runtime-neutral: schema/, nodes/, elements/, compile/, templates/
src/video/     Remotion root: DemoTimeline, registry/, core/, camera/ (programatic-demo), elements/ + lib/ (registries)
src/renderer/  React UI: routes/, app/, editor/, store/, components/
scripts/       registry ingest, examples, start wrapper, oxlint plugin
```

## Invariants that must not regress

**Runtime boundaries.** `src/shared` and `src/video` are imported by the Vite renderer, the worker and the Remotion webpack bundle: no `node:*`, `electron`, `playwright-core`, `@remotion/renderer` or `@remotion/bundler` there (oxlint enforces it). Node-only code lives in `src/worker` and `src/main`. `src/shared/elements` metadata never imports a component; implementations are keyed separately in `src/video/registry`.

**Every animation in `src/video` is a pure function of `useCurrentFrame()`.** No Framer Motion, `motion`, GSAP, CSS transitions or Tailwind `animate-*` (oxlint: `no-restricted-imports`, `demoflow/no-animate-class`). Randomness uses `random(seed)`; async work uses `delayRender()`.

**Nodes never generate Remotion code.** A node definition (`src/shared/nodes`) has ports, zod params and an optional pure `compile()`; executors (`src/worker/executors`) are keyed by node type. Every node with `executes: true` has an executor (tested). Compile produces a `TimelineSpec`; the single `DemoTimeline` composition renders it in both the editor `<Player>` and the worker's `renderMedia()`, so preview equals render.

**Browser nodes are virtual.** They form a `session` chain; the capture node that closes it replays the whole chain in one take.

**Params carry their own inspector UI** through zod `.meta({ description, help, widget, step, unit, labels, zeroLabel, presets, showIf, advanced, hidden, pauses })` (type `FieldMeta` in `src/shared/elements/introspect.ts`; add to existing meta with `more()`). `unit: "ms"` and `unit: "frames"` are stored as-is and shown in seconds. Widgets: `textarea`, `code`, `asset`, `target`, `secret`, `timezone`, `keys`, `datetime`, `variable`, `backdrop`, `offsets`, `spot`, `quality`, `loudness`, `transition`, `multiselect`, `color`. Plain text fields render variables as chips (`VarText`); never show `{{…}}` to a user.

**Node cards.** Every visible node has a `description` and a `card()` returning up to four `row(icon, text)` lines (`src/shared/nodes/card.ts`), computed from interpolated params, run results, saved sign-ins and connected inputs. Icons are lucide names and must exist in `src/renderer/editor/icons.ts`. Nodes that produce a file show a poster frame (made by `decorateArtifacts` in the worker, stored in the node's cache folder); nodes that produce a clip show a live `<Thumbnail>`, limited to six at a time and only when the card is on screen and the canvas zoom is at least 0.55.

**User-facing copy.** No library, tool or format names (Playwright, Remotion, Chromium, FFmpeg, CRF, DPR, page.evaluate…) in node titles, descriptions, field labels, badges, toasts or error messages; technical settings go under `advanced`. Sentence case. Durations in seconds. Browser nodes are "actions", never "steps". Elements are described in words (`elementName`). The node's name (`node.label`) is also its camera, chapter and caption label; `normalizeNode` migrates the old `params.label`. Categories keep their internal ids; display names come from `CATEGORY_LABELS`. Registry element descriptions come from the registry item JSON at ingest time, with overrides in `DESCRIPTIONS` (`scripts/registry-ingest.list.ts`). `src/shared/nodes/card.test.ts` enforces descriptions and the banned words.

**Editor panels.** The node list, Inspector and Preview/Run panels collapse (Ctrl/Cmd+B, Ctrl/Cmd+Alt+B, Ctrl/Cmd+J, also in the View menu and the top bar); `src/renderer/editor/panels.ts` owns their handles and open state, and layouts persist through `useDefaultLayout`. A collapsed node list becomes a 44 px category rail with popovers.

**One engine, two editions.** Capture (Playwright) and render (Remotion) launch the same Chrome Headless Shell: whatever `ensureBrowser()` picks for the platform at Remotion's pinned `TESTED_VERSION` (Remotion's own build on Linux x64/arm64 and Apple Silicon, Google's Chrome for Testing elsewhere). FFmpeg and ffprobe come from the ffmpeg-static GitHub release pinned in `src/engine/paths.ts`. The engine lives in `resources/engine` (full edition, staged by `forge.config.ts`) or `<userData>/engine` (lite edition and dev, installed from Settings). `DEMOFLOW_EDITION=lite bun run make` builds without it. Never fall back to Playwright's own browsers, system Chrome or Electron's Chromium for capture or render.

**Capture timing comes from frames, not markers.** Screencast `onFrame` JPEGs go into `FrameEncoder`, which writes a constant-rate stream from the first frame after the setup steps; action timestamps share that origin. Live frames for the editor PiP are the same frames, throttled to the live fps setting, sent over a dedicated MessagePort straight from the worker to the renderer.

**Interactive browsing happens in Electron windows**, never in headed Playwright: the element picker and Login once open sandboxed `BrowserWindow`s with their own session partition (`src/main/browser-windows.ts`); login state is converted to Playwright storage-state JSON and encrypted with `safeStorage`. A Login once node has no edges: the runner (`loginDependencies` in `src/shared/graph.ts`) schedules it before any capture whose Session's `storageState` names its secret, and only when that secret is not saved yet. The login window finishes on URL change or selector even in single-page apps (`did-navigate-in-page`, selector polling).

**Traces** are recorded per capture unless retention is `off`, ingested into `.demoflow/traces` (zstd for text, raw for images, content-addressed blobs shared across runs), pruned by the retention setting. Blob files keep the entry's extension (`<sha>.jpeg`, `<sha>.zst`) so screenshots load straight through `demoflow-file://`. The Trace tab in Replay (`TracePanel`) is our own shadcn UI over `readTrace()` (`src/traces/model.ts` parses `trace.trace`/`trace.network`); never embed Playwright's Trace Viewer or open a local server for it. The driver wraps each step in `tracing.group(label)` and the manifest's `groups` maps those groups back to node ids in order.

**Registry elements load lazily.** `components.generated.ts` holds `import()` loaders; `DemoTimeline` waits on `delayRender` until an element's module is loaded, so unused elements never run their side effects (font loads).

**Asset refs are host-independent.** Specs hold `project:<relative path>` or `app:<public path>`. The editor resolves `project:` through `demoflow-file://project/`, renders through the worker's 127.0.0.1 asset server, `app:` always through `staticFile()`.

**Cache keys.** A node hash is `sha256(ENGINE_VERSION, type, interpolated params, upstream hashes)`. Bump `ENGINE_VERSION` in `src/worker/engine/cache.ts` whenever an executor's output format changes.

**Packaging.**

- Forge's Vite plugin ships only `.vite/`; anything the worker loads at runtime and Vite leaves external must be staged in `forge.config.ts` `packageAfterCopy` (playwright-core, the compositor package).
- The Remotion bundle is prebuilt at package time; a packaged app never calls `bundle()`. `@remotion/bundler` and `@remotion/tailwind-v4` are imported lazily for that reason.
- `binariesDirectory` points at the compositor in `app.asar.unpacked`; all renders pass `licenseKey: "free-license"`.
- Remotion ignores tsconfig `paths`; the `@` alias is mirrored in `src/worker/render/webpack-override.ts`.
- Build entries need unique basenames (`main.ts`, `preload.ts`, `worker.ts`): Forge names outputs `<basename>.cjs`.

**Preview quality.** The editor preview defaults to Fast: `TimelineSpec.quality = "fast"` makes `RecordingClip` play the 960 px `recording.preview.mp4` (made by the Record executor, keyframe every half second) and skip camera motion blur, whose 8 samples each decode the full recording. Exact matches the export; renders never set `quality`, so they are always exact. Card thumbnails always use Fast.

**Animation names are global.** React Flow UI's `node-status-indicator.tsx` injects `@keyframes spin` (with a −50% translate) while a node runs, so Tailwind's `animate-spin` is remapped to `icon-spin` in `src/renderer/index.css`. Never name a keyframe `spin`.

**One title bar.** The window has no system title bar. macOS uses `titleBarStyle: "hiddenInset"` (traffic lights inside our bar, native menu in the system menu bar); Windows and Linux use `titleBarStyle: "hidden"` + `titleBarOverlay` (Electron draws the window buttons, height 48 = the bar) with no native menu. Every screen renders `AppBar` (`src/renderer/app/AppBar.tsx`): the `app-drag` utility makes it the window's drag handle and every control inside it `no-drag`; it reserves room for the buttons with `env(titlebar-area-*)`, and on Windows/Linux shows the in-app menu (full Menubar from 1280 px, one Menu button below). Menus come from one list, `appMenu()` in `src/shared/menu.ts`, used by the native macOS menu and the in-app menu; `useMenuShortcuts` handles its accelerators in the page on Windows/Linux (leaving cut/copy/paste/select-all/undo/redo to text fields while one is focused), roles go to `runWindowRole` in main, and navigation commands are handled once in the root route. Overlay colours are read from the theme tokens at start-up.

**Renderer hosting.** Hash history only. The packaged UI is served from `demoflow-app://app/`; its CSP is a response header set by the protocol handler, never a `<meta>`, because Vite's dev React-refresh preamble is inline.

**Linux dev sandbox.** `bun start` runs `scripts/start.ts`, which adds `--no-sandbox` on Linux in development only (AppArmor blocks unprivileged user namespaces and `chrome-sandbox` is not root-owned in `node_modules`). Packaged builds keep the sandbox.

## Licences

Demoflow is Apache-2.0 (`LICENSE`). Vendored code keeps its notices: `THIRD_PARTY_NOTICES.md`, `src/video/camera/LICENSE.programatic-demo`, `src/worker/capture/LICENSE.programatic-demo`. FFmpeg ships as separate GPLv3 executables with its licence.
