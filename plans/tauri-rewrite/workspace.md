# Workspace

The repository becomes a Bun-workspace turborepo with three TS packages and one Rust crate. `src/shared` stops existing: every file goes to the one package that owns it, and what two sides need is either generated from Rust or kept small and tested on both sides.

## What it adds

- Each part builds with its own tool and ships only what it imports: the UI with Vite, the video bundle with Remotion's bundler, the sidecar with `bun build`, the app with `cargo` through the Tauri CLI.
- Package boundaries replace today's lint boundaries: `video` can't import Node or the UI because it doesn't depend on them.

## Fit with Opsis

Already in place:

- `src/shared` is runtime-neutral by rule (oxlint forbids `node:*`, `electron`, `playwright-core`, `@remotion/renderer`, `@remotion/bundler` there), so its files can move without untangling Node code.
- Bun is already the package manager.
- `src/video` and `src/renderer` already only meet through `src/shared`, apart from two imports: `registerFaces`/`unregisterFaces` (`src/renderer/fonts/library.ts`) and `designScaleFor` (`src/renderer/editor/live-view.tsx`).

Missing:

- No workspace, no turbo, no Rust.

## The packages

| Package            | Contains                                                                                                                                                                    | Built by                                    | Ships as                                   |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------- | ------------------------------------------ |
| `packages/ui`      | `src/renderer`, plus the `shared` files only the editor uses (Timeline and workflow edits, snapping, gradient editing, node looks, font plan, small UI helpers)             | Vite                                        | the Tauri frontend (`frontendDist`)        |
| `packages/video`   | `src/video`, plus what the spec needs to be drawn: generated spec types, `walkItems`, `groupOf`, `remapFrame`, time, envelope and camera-path helpers, fonts, small helpers | `@remotion/bundler` (webpack) at build time | a resource folder the headless shell loads |
| `packages/sidecar` | `src/worker/render` minus what moves to Rust (the mux in `picture.ts`, the asset server)                                                                                    | `bun build`                                 | a Tauri sidecar (`bundle.externalBin`)     |
| `app` (Rust crate) | Everything else: today's `src/main`, `src/engine`, `src/traces`, `src/worker` (except render), and node definitions, compile, cards, flattener, catalogue, validation       | `cargo` via `tauri build`                   | the app binary                             |
| `scripts/`         | Registry ingest, examples, fonts and glyph bundling, the oxlint plugin                                                                                                      | `tsx`                                       | not shipped                                |

`packages/video` exposes two entry points through its `package.json` `exports`: `video/spec` (types and the helpers above) for the UI, and the Remotion root for the bundler. The UI may import only `video/spec`; the exports map makes anything else unreachable.

## Where each `src/shared` file goes

Totals from `code-split.md`: 18,484 source lines.

| Destination                          | Lines | Files                                                                                                                                                                                                                                                                                                                                                                           |
| ------------------------------------ | ----- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Rust**                             | 9,988 | `nodes/*` except `looks.ts` and `icon-name.ts`; `compile/*`; `graph.ts`; `api/*` (including `openapi.ts`); `mix/graph.ts`, `tempo.ts`, `plan.ts`; `timeline/flatten.ts`, `lanes.ts`, `action-labels.ts`, `audio-bars.ts`, `thumbnails.ts`, `chapters.ts`; `elements/index.ts`, `core.ts`, `lengths.ts`; `menu.ts`, `run-progress.ts`, `locale.ts`, `wall-time.ts`, `secrets.ts` |
| **Rust types, TS generated**         | 1,710 | `schema/*`, `mix/types.ts`, `mix/result.ts`, `ipc.ts`, `settings.ts`, `trace.ts`, `live.ts`, `analysis.ts`, `interactive.ts`, `engine.ts`, `events`                                                                                                                                                                                                                             |
| **Both: ported to Rust, kept in TS** | 1,134 | `asset-ref`, `color`, `format`, `limits`, `mime`, `project-dirs`, `project-folder`, `safe-name`, `text-input`, `vars`; `elements/colors`, `screen-3d`, `seconds`, `theme`, `three`, `types`; `mix/levels`; `nodes/icon-name`; `timeline/time`, `envelope`, `camera-path`                                                                                                        |
| **Data (JSON)**                      | 3,000 | `elements/elements.generated.ts` (the ingest writes it for Rust), `templates/*` (become JSON files; the six examples are already written from them), `sounds.ts`, `backdrops.ts`                                                                                                                                                                                                |
| **ui**                               | 1,773 | `timeline/edits.ts`, `snap.ts`, `handles.ts`, `workflow-edits.ts`, `workflow-diff.ts`, `mix/edits.ts`, `nodes/looks.ts`, `elements/introspect.ts` (rewritten for JSON Schema), `fonts/plan.ts`, `gradient.ts`, `errors.ts`, `ids.ts`, `named-files.ts`, `listener-set.ts`, `timing.ts`                                                                                          |
| **video**                            | 567   | `fonts/index.ts`, `types.ts`, `bundled.generated.ts`, `math`, `guards`, `memo`, `lists`, `hash`, `chord`, `objects`, `window-styles`, `audio`, `mix/duck.ts`                                                                                                                                                                                                                    |
| **deleted**                          | 101   | `worker-protocol.ts`, `pending-replies.ts`                                                                                                                                                                                                                                                                                                                                      |
| **scripts**                          | 8     | `fonts/google-font-imports.ts`                                                                                                                                                                                                                                                                                                                                                  |
| **tests**                            | 203   | `test-fixtures.ts` (its users move to Rust tests)                                                                                                                                                                                                                                                                                                                               |

## Work

1. **Workspace.** Root `package.json` with `workspaces: ["packages/*"]`, `turbo.json` with `build`, `typecheck`, `lint`, `test` per package and `app#build` depending on `ui#build`, `video#build` and `sidecar#build`. Add turbo with `bun add -d turbo`.
2. **Move without changing behaviour (phase 1, still Electron).** Move `src/renderer` → `packages/ui/src`, `src/video` → `packages/video/src`, `src/worker/render` → `packages/sidecar/src`; split `src/shared` by consumer as in the table, with the Rust-bound files staying in a temporary `packages/core` until phase 3 removes it. Use the TypeScript rename edits so imports follow (`AGENTS.md`), then search configs, scripts and docs for old paths. Check: the app runs and every test passes.
3. **The two UI→video imports.** `registerFaces`/`unregisterFaces` and `designScaleFor` move into `video/spec`.
4. **Tool configs per package.** Each package gets its own `tsconfig.json`, `vitest.config.ts` and oxlint config; `oxlint.config.ts` keeps the video rules (`no-restricted-imports` of animation libraries, `opsis/no-animate-class`, `opsis/no-three-clock`) in `packages/video`.
5. **Rust crate.** `cargo` workspace at the root with the `app` crate, created by the Tauri CLI; `tauri.conf.json` points `frontendDist` at `packages/ui/dist` and `externalBin` at the sidecar build.
6. **Retire `packages/core`** in phase 3, when its last file has moved to Rust or to generated types.

## Phase 1 as built

Steps 1 to 4 are done (2026-10-25, branch `feat/tauri`); the app still runs on Electron from the root.

- **Names.** The folders are `packages/core`, `packages/video`, `packages/ui`, `packages/sidecar`; the package names are scoped (`@opsis/core`, …) so no import can resolve to an unrelated npm package. The Electron app stays at the root (`src/main`, `src/preload`, `src/worker`, `src/engine`, `src/traces`) and depends on all four.
- **Files that wait in `core`.** A `src/shared` file moved to its final package only when nothing that stays in `core` (or in the Electron app) imports it, or the packages would import each other in a circle. Moved now: to `ui` `timeline/edits`, `timeline/snap`, `workflow-edits`, `workflow-diff`, `mix/edits`, `nodes/looks`; to `video` `mix/duck` and `fonts/google-font-imports` (the table says `scripts`, but `video/fonts/google-font.ts` imports it). Waiting in `core` until phase 3 or 4 frees them: `elements/introspect`, `fonts/plan`, `gradient`, `errors`, `ids`, `named-files`, `listener-set`, `timing`, `timeline/handles` (ui); `fonts/index`, `fonts/types`, `fonts/bundled.generated`, `math`, `guards`, `memo`, `lists`, `hash`, `chord`, `objects`, `window-styles`, `audio` (video). `nodes/icon-name.test.ts` moved to `packages/ui/src/editor`, because it scans node definitions and the editor's icon map.
- **Exports.** `video` exports `spec` (`designScaleFor`, `registerFaces`, `unregisterFaces`, `duckGain`, `duckingSpans`), `root`, and three entries that go away with the code that uses them: `player` (`DemoTimeline`, until the preview is streamed in phase 5), `capture` (what the worker's capture reads, until phase 5), plus `composition-id`, `fonts/google-font-imports` and `registry/framing` for the sidecar, the Vite config and the registry scripts. `core` and `sidecar` export every file; `ui` exports `nodes/looks` for `glyphs:bundle`. Every export key is a file path under `src`.
- **Imports.** `core`, `video` and `sidecar` import their own files by relative path, since other packages compile them; only `ui` keeps the `@/` alias. The registry ingest writes relative imports into installed elements, and the registry patches were rewritten to match. It now runs shadcn inside `packages/video` (its own `components.json`).
- **Dependencies.** Each package declares what it imports, at the versions the root had (`bun add` per package, `bun remove` at the root); the lockfile resolves the same versions. Installs are hoisted (`bunfig.toml`): with bun's isolated layout, Vite's pre-bundled dependencies couldn't resolve their own imports (`@floating-ui/utils` from `@base-ui/react`) from the root. Revisit when the root no longer runs the editor's Vite build.
- **Vite.** The renderer config aliases `@opsis/core` and `@opsis/video` to their `src` folders: through `node_modules` its dependency optimizer took them for prebuilt packages and parsed their TypeScript as JavaScript, and excluding them left their CommonJS dependencies (`lottie-web`) unconverted.
- **Sidecar.** `render.ts` no longer reads the engine folder or the codec table: callers pass `browserExecutable` and the `crf` (or none). `free-tcp-port.ts` moved with it. `picture.ts` and the asset server stay in `src/worker/render` until phase 4. Export as code copies `video`, `core` and the webpack override (`EXPORTED_SOURCES`, `source-layout.ts`) and reads dependency versions from every package's manifest.
- **Turbo.** `typecheck`, `lint`, `lint:fix` and `test` run per package, with root tasks (`typecheck:app`, `lint:app`, `test:app`) for the Electron app; `cacheDir` is set so a git worktree keeps its own cache. `build` is declared for phase 2; no package has a build script yet.
- **Checked.** Type check, lint and the 1,280 tests (the same 172 files as before) pass; the generators (`routes`, `glyphs:bundle`, `fonts:bundle`, `db:schema`, `examples`) write the same files; the dev app and a packaged lite build run. The changelog example exports the same 1:1 file byte for byte as before the split; its 16:9 file differs in the last 75 frames, which the code before the split also does between two runs, so a byte-for-byte check needs that nondeterminism found first. The to-do example records the same take key, and its render fails the same way before and after the split (`Error loading image with src: …/glaze`).

## Decisions for the user

- **Package names.** `ui`, `video`, `sidecar`, and the crate as `app` are suggestions.

## Risks

- **Two bundlers for video code.** Remotion's bundler builds `packages/video`; the UI imports `video/spec` through Vite. Keep `video/spec` free of anything Remotion-only so both resolve it.
- **Path aliases.** `@/` is mirrored in `src/worker/render/webpack-override.ts` today because Remotion ignores tsconfig `paths`; inside `packages/video`, use relative imports or the package name instead.

## Sources

- Import-graph walk of `src/` at `df01ff0` (scripts in this session's scratchpad), 2026-10-25; per-file counts in `code-split.md`.
