# TODO

Everything in the codebase that doesn't read as if Opsis had been written for Tauri and Rust from the start, in the order to fix it. Each item is removed when it is done. Release work (size, builds, signing, updates) is in `plans/release.md`.

## 1. Docs that describe a migration

`AGENTS.md` still tells the history of a port instead of describing the app:

- the generated-code section calls `fixtures/**` "the output of the TS code from before the switch… frozen: what wrote them is gone";
- the Layout lists `fixtures/` as "frozen output of the old TS code, read by the parity tests";
- **Workspace** says "The rewrite from TS was greenfield: Rust matches the old output where output matters…";
- **Prepared files** says the typefaces match "fontkit's output";
- the drizzle pin says "they only write `crates/store/db/schema.generated.sql` now (Rust runs the SQL)".

Rewrite those passages as plain descriptions of how things work, and check `README.md` and `THIRD_PARTY_NOTICES.md` for the same.

## 2. Feature plans written against the old tree

`plans/ai-assist.md`, `plans/motion-library.md` and `plans/voice-and-captions.md` describe their work in terms of the Electron app (`src/main`, `src/worker`, `src/shared`, zod `.meta()`, `BrowserWindow`, `utilityProcess`, Forge), with a "Rust" appendix explaining how it would change. Rewrite each as a plan for this app: crates, commands, events, JSON Schema settings, streamed views, with the appendix folded in.

## 3. Fixtures and parity tests

`fixtures/` (11 MB) holds outputs recorded from the old TS code: compile results, node and param cases, catalogue, element metadata, workflows (`fixtures/compile`), FFmpeg arguments, filter graphs, mix graphs, analysis and probe results (`fixtures/media`), capture tables (`fixtures/capture`). Tests compare against it as "parity": `crates/compile/tests/parity_{catalog,nodes,timeline,workflows}.rs`, `crates/media/tests/parity/`, `crates/capture/tests/parity.rs`, with helpers named for the port (`ts_defaults`, `added_after_the_switch`, `prepared_files_replace_parsing_in_the_page`). Turn them into tests owned by the Rust code: golden files the tests can write again on request, named for the behaviour they check, with no "parity", "TS" or "switch" left, and no accepted-difference helpers.

## 4. Prepared-file fixtures framed as fontkit parity

`fixtures/prepare/typefaces.json` is "fontkit's output", checked by `crates/prepare/tests/typefaces.rs` with helpers named for the differences from fontkit (`same_advance_with_fixed_point_coordinates`, `without_bounds_three_never_reads`); `fixtures/prepare/stills` are described as "before the Rust preparers". Make them golden files of the preparers' own output.

## 5. The `window.opsis` facade

`apps/ui/src/app/opsis.ts` (327 lines, `OpsisApi`, `createOpsis`) wraps every generated command and event in a preload-style object so call sites written for Electron keep working; 110 call sites use it. Call the generated `commands` and `events` from `@/generated/bindings` directly, with small typed helpers only where a channel or a frame decoder is involved.

## 6. Project state and undo in the editor

The editor's store owns the workflow, undo and autosave; Rust receives the whole workflow on every edit through `compile`. Local API edits are sent to the editor window (`apps/ui/src/editor/api-bridge.ts`), so the API depends on that window being awake. Move the document, undo, autosave and edits into Rust; windows render it and send edit commands.

## 7. Pop-out window sync

`apps/ui/src/app/sync.ts` (286 lines) and `relay` mirror the editor store into panel windows with `replace` and gesture messages, the design of a `BroadcastChannel`. With state in Rust (item 6), every window follows Rust's state events instead.

## 8. Logic written twice, in TS and in Rust

- the run-progress reducer: `apps/ui/src/core/run-progress.ts` and `crates/runner/src/progress.rs`;
- graph walks: `apps/ui/src/core/graph.ts` and `crates/compile/src/graph.rs`;
- bar clocks: `apps/ui/src/core/clock.ts` and `BarClock` in `crates/compile/src/timeline/flatten.rs`;
- ducking spans: `apps/ui/src/core/duck.ts` and the mix plan;
- variables: `apps/ui/src/core/vars.ts` and `crates/compile/src/vars.rs`;
- asset refs and font faces: `packages/video/src/common/asset-ref.ts` and `fonts.ts`, and `crates/compile/src/asset_ref.rs`, `fonts`, `elements/prepared.rs`;
- mixer ranges and defaults: `apps/ui/src/core/mix.ts`;
- settings defaults: `schemaDefaults` in `apps/ui/src/core/schema.ts` and `NodeDef::defaults`.

Keep one implementation in Rust and send the UI what it needs.

## 9. Editing logic in TS

Timeline edits (`apps/ui/src/timeline/edits.ts`, 696 lines), `apps/ui/src/workflow-edits.ts`, `apps/ui/src/mix/edits.ts` and `apps/ui/src/workflow-diff.ts` edit the workflow JSON in the editor. They belong in Rust commands next to the document (item 6), tested in Rust.

## 10. Folder names from the old packages

`apps/ui/src/core` echoes the removed `packages/core`. Name folders for what they hold once items 5–9 have emptied most of it.

## 11. JS number helpers in Rust

`js_number` in `crates/media/src/ffmpeg.rs` copies JavaScript's `Number()` (empty text is 0, anything unreadable is NaN) for ffprobe fields, and `js_number` in `crates/prepare/src/typeface.rs` copies how JavaScript prints numbers. Parse ffprobe fields into `Option` values and write the typeface numbers as Rust formats them.

## 12. `f64` for whole numbers

Frames, milliseconds, pixel sizes and counts are `f64` throughout `crates/types` and `crates/compile` (for example `duration_in_frames`, `duration_ms`, `width`, `height`), with about 245 `as` casts. Use integer types where a value is whole, and convert only where the timeline spec needs a number for the video code.

## 13. Element props as JSON maps

Element props are `serde_json` maps read by camelCase key (`prepare_props` reads `"fontFamily"`, `"fontWeight"`, `"src"`); `Values` is passed around in `crates/compile/src/nodes/elements.rs`, `node.rs` and `params.rs`. Use the typed settings structs of the core elements (`CoreElementProps`) wherever an element is known, keeping maps only for the registry elements, whose settings come from data.

## 14. `String` errors

`Result<_, String>` in `crates/compile/src/node.rs`, `params.rs`, `nodes/browser.rs`, `style.rs` and `crates/desktop/src/state.rs`. Give each module a typed error.

## 15. Errors turned into defaults

97 `unwrap_or_default()` calls; some hide real failures, such as `rel_dir: to_posix_rel(&self.project_dir, &dir).unwrap_or_default()` in `crates/runner/src/runner.rs`, which makes an empty ref instead of failing. Keep defaults only where a missing value really means the default, and return errors elsewhere.

## 16. Node kinds as strings

`crates/compile/src/graph.rs` compares node kinds to string literals (`n.kind == "browser.signIn"`, `"browser.setState"`). Use the node definitions' own kind constants, or an enum for the built-in kinds.

## 17. The database schema written in drizzle

`crates/store/db/schema.ts` is a drizzle schema whose only use is `drizzle-kit export` into `schema.generated.sql`, which Rust embeds; `drizzle-kit` and `drizzle-orm` are pinned to a release candidate for it. Keep the schema as a plain SQL file owned by `opsis-store` and drop drizzle.

## 18. Engine logic in TS scripts

`scripts/engine-downloads.ts`, `scripts/lib/engine/*` (download, atomic writes, paths) and `scripts/lib/app-data.ts` repeat what `opsis-engine` and the desktop paths already do. Move them into Rust binaries (as `install-engine` already is), keeping TS only where a TS package is the input (`@remotion/renderer`'s `TESTED_VERSION`).

## 19. Element metadata authored in zod

`scripts/lib/element-meta.ts` writes registry element settings as zod schemas with `.meta()` and converts them to JSON Schema for Rust. The ingest reads the elements' TSX, so it stays in TS, but the metadata format could be JSON Schema with `x-` keywords from the start, as the Rust nodes use.

## 20. Old build output in the checkout

`.vite/` (Forge's output), `resources/` (files staged for the Electron package) and an Electron-era `node_modules/` are still in the main checkout. Delete them and run `bun install`.
