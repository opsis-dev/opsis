# Code split

How many lines move to Rust, what stays exactly as it is, what changes, and what ends up on both sides. Counts are source lines without tests unless a row says tests, measured on the tree at `df01ff0` on 2026-10-25.

## How it was measured

- A script walked every import (runtime and type-only) from the entry points: `src/renderer/main.tsx`, `src/video/index.ts`, every file of `src/main`, `src/engine`, `src/traces` and `src/worker`.
- For the UI, it cut every import into node definitions, compile, graph, API and templates (the UI gets those from Rust) and into `src/video` (no Player in the webview).
- Each `src/shared` file then got one destination (`workspace.md`), and the per-destination totals add up to all 18,484 lines.

## 1. Converted to Rust: 23,012 lines

| From                                                                                                                                                                                                                                                          | Lines |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----- |
| `src/main` logic: fonts, online fonts, projects, Local API routes and server, API editor bridge, socket address, licences, asset import, loaded-files watch, database, key sealing, settings, secrets, view state, engine setup, model import, analysis queue | 1,846 |
| `src/main` Electron plumbing rewritten on Tauri: window registry, bounds, project window, panel windows, app windows, settings window, window roles, placement, raise, `main.ts`, menu, protocol, dialogs, canvas colour, exit, IPC handlers                  | 944   |
| `src/engine` (paths, installer, licences, hashing, JSON reading)                                                                                                                                                                                              | 703   |
| `src/traces` (our own trace format replaces Playwright's)                                                                                                                                                                                                     | 749   |
| `src/worker`: engine (runner, cache), executors, capture, media, mix, `worker.ts`; excludes the 203-line page script                                                                                                                                          | 5,815 |
| `src/worker/render`: the mux (`picture.ts`) and the asset server                                                                                                                                                                                              | 123   |
| `src/shared` → Rust                                                                                                                                                                                                                                           | 9,988 |
| `src/shared` types → Rust (TS generated from them)                                                                                                                                                                                                            | 1,710 |
| `src/shared` kept on both sides (Rust copy)                                                                                                                                                                                                                   | 1,134 |

**Tests to write in Rust: about 9,200 lines** — `src/shared` 5,180 (Rust-bound 4,437, types 130, both-sides 613), `src/main` 1,002, `src/engine` 333, `src/traces` 437, `src/worker` 2,259.

**New Rust code these counts don't cover:**

- what Playwright does for capture today (locators, auto-waiting, screencast, tracing; `capture-cdp.md`);
- the streamed views (`streamed-views.md`);
- the Local API server on a socket and a named pipe (`local-api.md`);
- the sidecar link (`sidecar.md`);
- asset preparation and image nodes (`asset-preparation.md`, `image-nodes.md`).

## 2. Deleted or turned into data

| What                                                                                                                                                        | Lines |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------- | ----- |
| Worker hosting in main: `lazy-worker`, `workers`, `worker-temp`, `run-host`, `run-reporter` (the runner runs in-process)                                    | 438   |
| Electron picker and sign-in windows: `browser-windows`, `login-state` (streamed views replace them)                                                         | 201   |
| `src/preload` (Tauri `invoke` replaces it)                                                                                                                  | 86    |
| `worker-protocol.ts`, `pending-replies.ts`                                                                                                                  | 101   |
| **Data instead of code:** `elements.generated.ts` 2,418 (the ingest writes it for Rust), templates 488 (JSON files), `sounds.ts` + `backdrops.ts` 94 (JSON) | 3,000 |

## 3. Unchanged

| What                                                                                           | Lines         |
| ---------------------------------------------------------------------------------------------- | ------------- |
| `src/video` vendored registry code (`elements/`, `lib/`, `hooks/`)                             | 45,692        |
| `src/video` own code (51 files only change import paths)                                       | 7,040         |
| Renderer: installed shadcn and React Flow components, generated route tree                     | 9,021         |
| Renderer: own code with no change points                                                       | 10,778        |
| Page scripts injected as they are: picker 219, capture 203                                     | 422           |
| `src/shared` → `apps/ui`                                                                       | 1,773         |
| `src/shared` → `packages/video`                                                                | 567           |
| `src/shared` kept on both sides (TS copy)                                                      | 1,134         |
| Sidecar: `render.ts`, bundle, bundler override, source layout, compositor lookup (small edits) | 271           |
| drizzle schema (`src/main/db/schema.ts`; `drizzle-kit export` still writes the SQL Rust runs)  | 47            |
| `scripts/` (minus `start.ts` and the AppImage launcher)                                        | most of 4,909 |

## 4. TS that changes

- **Rewritten: 901 lines.** The preview becomes a streamed view (`preview-composition`, `preview-controls`, `preview-dock`, `preview-toolbar`, `use-player-bridge`: 600) and `sync.ts` moves to Tauri events (301).
- **78 renderer files (9,331 lines) hold change points; the changes are small:**
  - 40 files import node definitions, compile, graph, API or templates and switch to the catalogue and compile results Rust sends;
  - 49 files make 83 `window.opsis` calls; one `window.opsis` facade over `invoke` keeps every call site;
  - window controls (`app-bar.tsx`), live frames (`live-view.tsx`), local fonts (`fonts/library.ts`);
  - the Inspector forms (`zod-form.tsx` 464, plus `introspect.ts` 163 rewritten) read JSON Schema.
- **`apps/ui` files from `shared` that imported node definitions:** `timeline/edits.ts` (680), `workflow-edits.ts` (186), `named-files.ts`, `workflow-diff.ts` read node defaults from the catalogue instead.
- **`packages/video`:** import paths in 51 files. The no-stem ducking fallback (`AudioLayers` in `demo-timeline.tsx`, `mix/duck.ts`) is no longer reached once every film plays its stem and can go.

## 5. Left on both sides

**Types: none kept by hand.** The 1,710 lines of schemas and plumbing types become Rust `serde` types; the TS types are generated (`types-and-contracts.md`).

**Logic: 1,134 lines in 21 files**, kept in TS for the UI or the video bundle and ported to Rust for compile and the executors:

| Group                                                                                                                                | Lines |
| ------------------------------------------------------------------------------------------------------------------------------------ | ----- |
| Small helpers: `asset-ref`, `color`, `format`, `limits`, `mime`, `project-dirs`, `project-folder`, `safe-name`, `text-input`, `vars` | 529   |
| Timeline maths the video draws with: `time`, `envelope`, `camera-path`                                                               | 184   |
| `mix/levels`                                                                                                                         | 166   |
| Element constants: `colors`, `screen-3d`, `seconds`, `theme`, `three`, `types`                                                       | 157   |
| `nodes/icon-name` (the Rust enum and the UI's icon map)                                                                              | 98    |

They are kept in step by fixture tests: the 613 lines of tests for these files become JSON fixtures that both the TS and the Rust tests read.

## Sources

- Line counts with `wc -l` and the import-graph scripts in this session's scratchpad (`split2.mjs`, `dest.mjs`), 2026-10-25.
