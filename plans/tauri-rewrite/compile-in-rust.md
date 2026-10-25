# Compile in Rust

Node definitions, compile, cards, the timeline flattener, the catalogue and validation move to Rust. The editor sends its workflow and gets compiled values back; the runner and the Local API call the same code in the same process.

Crate: `opsis-compile` (phase 2; steps 7 and 8, the IPC and the UI, land with the switch in phase 6).

## What it adds

- One compile per change instead of one per window: today the editor and every pop-out panel window each compile on every keystroke.
- Runs and `/validate` compile in-process, with no copy in a worker and no call to the UI or the sidecar.
- The webview carries no node definitions, no zod schemas for settings and no compile code.

## Fit with Opsis

### What compile does today

`compileWorkflow` (`src/shared/compile/compile.ts`) takes the workflow and any run results and walks the nodes in dependency order (`topoSort`). For each node:

1. **Settings:** `{{variables}}` are filled in, then the node's zod schema parses its params, adding defaults and checking ranges (`interpolatedParams`, `parseParams`).
2. **Inputs:** values on the incoming edges, converted to the type each port takes (`gatherInputs`, `convertInput`, the `CONVERT` table in `src/shared/nodes/ports.ts`); a `many()` port merges its values.
3. **The node's `compile()`:** browser actions append a step to the `browser-session` list; style and grade nodes append to theirs; element nodes turn settings into component props (seconds to frames, `mapLeaves`, sounds into Mixer items); Sequence and Layers place clips and transitions; Auto zoom applies `STYLE_TABLE`; Add sound places sounds on action marks through `flattenSpec`; nodes that execute take their outputs from the run results. A disabled node passes its input through.
4. **Specs:** each output node's clip becomes a `TimelineSpec` (`clipToSpec`); errors are collected per node.

It returns `{ values, errors, specs, order }`. There are 221 node types; 219 have a `compile()`: the 138 elements and transitions share one generic compile (`src/shared/nodes/elements.ts`), the other 81 are written by hand. 220 have a `card()`.

### Where it runs today, and how often

| Where                      | When                                                                                                                                                                                                                                       | Count                                                                                               |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------- |
| Editor (`use-compiled.ts`) | `compileWorkflow` + `validateWorkflow`, memoised on `[workflow, results]`: every edit (fields write on every keystroke) and every run result that arrives                                                                                  | 1 per change in the editor (`editor.tsx:67`), plus 1 per open pop-out window (`panel.$name.tsx:20`) |
| Editor API bridge          | `workflowProblems` (`api-bridge.ts:35`, `:156`)                                                                                                                                                                                            | per API request that asks                                                                           |
| Worker runner              | `workflowProblems` at the start (`runner.ts:482`); `ResultStore.compile()` again whenever a new result arrived, before the next node's inputs (`runner.ts:229`); params parsed for cache keys (`hashNodes`, `runner.ts:133`) twice per run | about 1 + (nodes that execute) per run                                                              |
| Main, `/validate`          | `checkWorkflow` (`api-routes.ts:108`): zod parse, `normalizeWorkflow`, `compileWorkflow`, `validateWorkflow`                                                                                                                               | 1 per request                                                                                       |

### Cost, measured on Node (2026-10-25)

| Example        | Nodes | Compile + validate (mean of 200) | Project file | Compile result (JSON, no run results) |
| -------------- | ----- | -------------------------------- | ------------ | ------------------------------------- |
| feature-launch | 11    | 0.65 ms                          | 5.9 KB       | 7.2 KB                                |
| todomvc        | 18    | 0.89 ms                          | 9.5 KB       | 15.4 KB                               |
| onboarding     | 14    | 0.47 ms                          | 7.2 KB       | 9.4 KB                                |
| changelog      | —     | —                                | 6.1 KB       | 13.1 KB                               |
| social-teaser  | —     | —                                | 5.5 KB       | 6.3 KB                                |
| before-after   | —     | —                                | 8.3 KB       | 12.9 KB                               |

Missing:

- Everything on the Rust side; a JSON Schema reader for the Inspector.

## Work

1. **Node registry in Rust.** A `NodeDef` trait (type, title, description, category, inputs and outputs with port types and `many`, `executes`, `phase`, `secret_params`, a params type with `JsonSchema`, `compile`, `card`) and one module per today's file (`browser`, `camera`, `timeline`, `media`, `elements`, `audio`, `grade`, `cursor`, `output`, `capture`, `effects`, `text`, `transitions`, `utility`). The style and grade modifiers (`styleModifier`, `recordingModifier`, `gradeModifier`) become generic helpers.
2. **Settings.** Each node's params struct carries defaults (`#[serde(default = …)]`), ranges and the field metadata as `x-` keywords (`types-and-contracts.md`). Interpolation of `{{variables}}` (`vars`) runs before parsing, as today; range checks produce the same messages.
3. **Element nodes.** The registry ingest writes element metadata in a form Rust reads (JSON, embedded) instead of `elements.generated.ts`; the generic element compile, `mapLeaves`, `framesFromSeconds` and `elementSounds` move to Rust with `ELEMENT_TIMINGS` and `animationLength` (`elements/lengths.ts`).
4. **Compile.** `compile_workflow(workflow, results) -> CompileResult` with the same four steps, `gather_inputs` and the conversion table; `clip_to_spec`, `sequence_clips`, transitions, `STYLE_TABLE`, the timeline flattener (`flatten`, lanes, action labels, audio bars, thumbnails, chapters) and the mix plan.
5. **Cards.** Every `card()` returns its rows (icon name from the `NodeIconName` enum, text) inside the compile result, so the UI draws rows it is given.
6. **Validation and catalogue.** `validate_workflow` (node, edge, cycle, reload, anchor, export-clash, sign-in-clash and skipped-source issues) and `check_workflow`; the catalogue (titles, descriptions, categories, ports, settings JSON Schema) for the UI's node list and Inspector and for `/nodes`.
7. **IPC.** The editor calls `compile(workflow)` through `invoke` on every change (the store already has every change; saving stays debounced as today). Rust compiles once, keeps the latest result per project, and sends it with the flattened timeline over a `tauri::ipc::Channel` that each window opens, so pop-out windows get the same result without compiling. Results arriving during a run trigger the same compile and send.
8. **UI changes.** `useCompiled()` becomes a subscription to that channel; cards, the Inspector, the Timeline and problems read it. The Inspector builds its forms from the catalogue's JSON Schema and `x-` keywords: `zod-form.tsx` and `introspect.ts` change from zod introspection to JSON Schema, keeping every widget component. Timeline and workflow edits (`timeline/edits.ts`, `workflow-edits.ts`) stay TS in the UI and read node defaults from the catalogue when they insert "Delay from timeline", "Trim from timeline", "Speed from timeline" or split parts.
9. **Tests.** Port the 4,437 lines of Rust-bound `src/shared` tests (node registry, cards, compile, timeline edits' compile side, mix plan and graphs, flatten, chapters). Add a parity check before the TS code is deleted: compile every example and every template with both implementations and compare the JSON.

## Phase 2 as built

Phase 2 (2026-10-25, branch `feat/tauri`) built steps 1 to 6 and the parity half of step 9; steps 7 and 8 (IPC, the UI on the compile channel and the JSON Schema Inspector) belong to phase 6, when the editor runs in Tauri.

- **Crates.** `crates/types` (`opsis-types`): the boundary types with `serde`, `specta` and `schemars`. `crates/compile` (`opsis-compile`): node definitions (`node.rs`, one module per today's node file in `nodes/`), settings (`params.rs`), compile (`compile.rs`), clip helpers, remap, envelopes, the style table as a registry lookup (`style.rs`), validation and `check_workflow` (`graph.rs`, `check.rs`), cards and their context (`cards.rs`), the flattener, chapters and camera paths (`timeline/`), the mix plan and levels (`mix/`), the catalogue (`catalog.rs`), elements and their lengths (`elements/`).
- **Settings.** Each node's settings are a Rust struct with `serde` defaults (`smart-default`) and `schemars` attributes: the field metadata as `x-` keywords, our own checks as an `x-check` keyword (`not-blank`, `colour`, `colour-or-empty`, `locale-or-empty`, `time-zone-or-empty`, `wall-time`, `in-project`, `unique`) with their messages. `jsonschema` validates with that keyword; `params.rs` fills defaults (in rows too), drops unknown keys, words the first problem in field order as today (`is missing`, `must be at least 320 px`, `takes a number. Variables work only in text settings`…) and the struct's `Params::check` covers cross-field rules. Shared field groups (target, pause and zoom, pointer) are `#[serde(flatten)]` structs.
- **Elements.** The 29 core elements are Rust structs (`elements/core.rs`). The registry ingest now also writes `crates/compile/src/elements/registry.generated.json`: each element's zod schema through `z.toJSONSchema`, with our meta as `x-` keys and `zColor` fields marked `x-check: colour` (`scripts/lib/element-json.ts`). `fonts:bundle` also writes `crates/compile/src/fonts/bundled.generated.json`; the built-in sounds are `crates/compile/src/data/sounds.json`.
- **Crates used instead of hand-written code.** `jsonschema`, `schemars`, `smart-default`, `jiff` (dates, time zones), `language-tags`, `heck`, `num-format`, `slug` (file and folder names), `url`, `percent-encoding`, `csscolorparser`, `icu_segmenter` (words in Chinese, Japanese and Thai scripts, as `Intl.Segmenter` did), `serde_path_to_error`, `xxhash-rust` (the in-memory mix plan key).
- **TS types.** `bun run types:export` writes `apps/ui/src/generated/bindings.ts` from both crates (also cards, compile results, problems, preview, flat timeline, mix plan, chapters, catalogue and check results); `bindings.test.ts` checks them against what the editor reads.
- **Parity.** `bun run fixtures` writes `fixtures/compile` (and, since phase 4, `fixtures/media` and `fixtures/run`) from the TS code; the Rust tests compare: every node's metadata and catalogue entry, 442 node cases, 14,199 settings cases, and the six examples, seven templates and a sound workflow (compile with and without run results, problems, cards, preview, flattened timelines, chapters, mix plans), plus 16 broken workflows for validation, `check_workflow`, run targets and connection problems. A vitest test fails when the fixtures or the generated JSON are stale.
- **Accepted differences, each a named function in the tests.** An empty Element field says “Pick an element”; colour problems say “Pick a colour” rather than Remotion's “Invalid color”; an incomplete nested setting says it “isn't valid”; optional fields take `null`; a setting's default is checked too (zod skipped refinements on defaults); sign-in and export file names come from `slug`; the mix plan key is xxh3 of canonical JSON.
- **Unit tests (ported during phase 4's audit).** `crates/compile/tests/unit` holds the Rust-bound `packages/core` tests, 153 cases: helpers, formatting, schemas, the catalogue, checks, graph, compile, clips, remaps, flattening, chapters, camera paths, mix levels and plan, and every node file. They use a small test graph builder (`helpers.rs`) like `GraphBuilder`; the mix filter-graph cases live in `opsis-media` with `mix/graph`. Tests of TS-only code stay in TS: zod field builders and `firstIssueText`, `safeName` (replaced by `slug`), `joinFolder`, `percent`, `formatTimeAgo`, `formatTime`, `playRange`, the colour picker's `formatColor`/`formatOf`, `menu.ts` (phase 6).
- **Fixed by that audit.** A saved `src`, `log` (recording), `ripple` or `file-info` value is required but may be `null`, as in TS (`deserialize_with = "Option::deserialize"`), instead of missing being read as none. `signed_text`, `format_bytes` and the speed and transition card rows used Rust's `{:.N}`, which rounds exact binary ties to even; they now use `text::to_fixed`, which rounds them as `toFixed` does, and `round_to` and `number` follow `Math.round` and `String(n)` (`ryu-js`), so one rounding rule runs through compile and the FFmpeg arguments. A whole-number float (`1000.0`) is now read by an integer setting, which the JSON Schema already accepted. Added what Rust-bound code needed: `size::estimate_card_height` and the layout constants, `mix::plan::channel_identity` with `strip_settings`, `text::wall_time_in`, `css_hex` and `hue_and_saturation`, `NodeDef::all_rows` (a card's rows before the four-row cut, for the card-length test). Removed duplicates: one `typed_values` for settings, compile's `project_folder_name` and `slug_name` in the project and font crates.
- **Left in TS on purpose.** Editor-only constants and helpers stay with the editor until phase 6 reads the catalogue: labels and choice lists (`CATEGORY_LABELS` and `NODE_CATEGORIES` are in the catalogue, `ENVELOPE_LABELS`, `PREVIEW_QUALITIES`, `EMPTY_SPEC`, the settings labels, the mixer's `STRIP_PRESETS`, `EQ_BAND_BOUNDS`, `LOUDNESS_CHOICES`, `BUS_SETTING_KEYS`, `BUILT_IN_BUSES`, `CHANNEL_KINDS`), the mixer's `resolveMix` and `audibility` (the plan has its own), `versionedUrl`, the form type guards. Capture-only helpers (`rectFromBounds`, `rectCenter`, `lerpPoint`, the storage state schema) come with phase 5.
- **Still thin.** The fixtures barely reach chapters (one chapter in one take); the unit tests add the chapter cases.

## Decisions for the user

- **Debounce.** Compile under 1 ms means no debounce is needed; if large projects prove slower, debounce the compile request, never the store.

## Risks

- **A frame of lag.** Cards and the Timeline update after a round trip instead of in the same render. Measure on a large project; if it shows, keep the last result on screen until the new one arrives (it already looks this way during typing).
- **Run results in the compile result.** The measured sizes have no run results; recordings add their action logs. Measure the result with a long take; the channel carries it either way.
- **Parity.** 81 hand-written compiles and the generic element compile are the largest single port; the JSON comparison in step 9 is what proves it.

## Sources

- `src/shared/compile/compile.ts`, `src/shared/graph.ts`, `src/shared/api/check.ts`, `src/renderer/editor/use-compiled.ts`, `src/worker/engine/runner.ts` (read 2026-10-25).
- Node counts from `NODE_DEFS` and timings from scripts run with `tsx` on Node 26.12 (2026-10-25).
- [Tauri: calling the frontend from Rust (events and channels)](https://v2.tauri.app/develop/calling-frontend/) (2026-10-25).
