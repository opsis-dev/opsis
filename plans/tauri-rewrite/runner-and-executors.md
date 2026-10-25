# Runner and executors

What the worker process does today, as tokio tasks inside the Tauri app: plan a run, key and cache results, run the 20 executors, render mix stems, store traces and report progress. Renders go to the sidecar (`sidecar.md`); capture is in `capture-cdp.md`.

Crates: `opsis-runner`, `opsis-media` and `opsis-render` (phase 4), `opsis-traces` and the temp folders in `opsis-engine` (phase 3).

## What it adds

- No worker process: no `utilityProcess`, MessagePorts, worker temp-folder bookkeeping in main, or a second copy of compile and results.
- Mixer levels without reading whole stems into memory.
- Loudness as numbers, not parsed from FFmpeg's log.

## Fit with Opsis

Already in place:

- The runner (`src/worker/engine/runner.ts`, 593 lines): `planRun` (topological order, the nodes a run needs, those that execute), `hashNodes` (cache keys), `ResultStore` (results plus a compile cached by version), `keyedAfterRun` (keys worked out again after a node with `secretParams` runs), `reportInvalid`, run events through one reducer (`applyRunEvent`, `src/shared/run-progress.ts`), `last-run.json`.
- Cache keys: `sha256(ENGINE_VERSION, type, interpolated params, upstream hashes)` plus file stamps of every project file a setting names, and for render-phase nodes the project settings, variables, mix and `PICTURE_VERSION`; nodes with `usesProjectSize` add width, height and frame rate; node and picture keys hold the app version (`AGENTS.md` **Cache keys**).
- The 20 executors (`EXECUTORS`, `src/worker/executors/index.ts`): `capture.record`, `capture.still`, `browser.signIn`, `media.import`, `media.trim`, `media.concat`, `media.speed`, `media.reverse`, `media.scalePad`, `media.convert`, `media.applyGrade`, `media.gif`, `media.extractAudio`, `media.loudnorm`, `media.thumbnail`, `media.faststart`, `media.probe`, `output.render`, `output.still`, `output.exportProject`.
- FFmpeg as separate programs (`src/worker/media/ffmpeg.ts`: `run`, `probe` through `ffprobe`), filter builders tested by string (`media/filters.ts`, `media/grade.ts`, `mix/graph.ts`), codecs (`media/codecs.ts`).
- Mix: stems into `.opsis/cache/mix/` (`src/worker/mix/render.ts`), per-frame peak and RMS read from `pcm_f32le` stems by `wav.ts` (the whole file with `readFile`, then a parsed copy), loudness from `parseEbur128` reading FFmpeg's log.
- Exports render the picture once without sound into `.opsis/cache/pictures/<pictureKey>` and mux the master stem (`muxArgs`, `src/worker/render/picture.ts`); chapters go into the finishing copy (`finishArgs`).
- Traces into `.opsis/traces` (zstd text, raw images, content-addressed; `src/traces`).
- Live frames to the editor over a dedicated MessagePort (`src/worker/worker.ts`).

Missing:

- All of it in Rust.

## Work

1. **Runner.** One tokio task per run with a `tokio_util::sync::CancellationToken` (cancel from the Run panel, `/project/run/stop` and app exit), a semaphore for the per-run concurrency, and the same plan, keys and order as today. Compile is a function call in the same process (`compile-in-rust.md`); `inputs_for` and interpolated params come from the cached compile result, recompiled when a new result arrives.
2. **Cache keys.** `sha2` 0.11.1 for every key that picks a file (`versionedHash`); the same inputs as today. Keys change format with the port; nothing has shipped, so old caches are deleted, not migrated (`AGENTS.md` **No old formats**).
3. **Run events and progress.** Rust owns run progress (the `applyRunEvent` reducer, `exportedArtifacts`) and sends each event plus the resulting progress over a `Channel`; the UI renders what it receives. `runStatus` and mid-run `resume` read Rust's state.
4. **Executors.** Port the 17 media and output executors with their argument builders and string tests (`media/*`, `executors/media.ts`, `executors/output.ts`, `executors/export-project.ts`), running `ffmpeg`/`ffprobe` with `tokio::process` and the same progress parsing; `probe` deserialises `ffprobe -print_format json` into `serde` types instead of zod. `output.render` and `output.still` send the render job to the sidecar and mux here. The three capture executors are in `capture-cdp.md`.
5. **Mix.** Port `mix/render.ts` and the graph builders. Stem levels: stream each `pcm_f32le` stem in blocks (a 10-minute 48 kHz stereo stem is 230 MB) and compute per-frame peak and RMS without holding the file. Loudness: the `ebur128` crate (0.1.10, MIT) on the same stream, replacing `parseEbur128`. Keep `MIX_VERSION` behaviour.
6. **Analysis.** Waveform levels and beat detection (`waveformLevels`, `detectBeats` in `src/worker/media/analysis.ts`) port to Rust; decoding stays FFmpeg (or `symphonia` 0.6.1, MPL-2.0, for audio-only reads where it decodes the format).
7. **Traces.** Port `src/traces` (749 lines) to Rust with `zstd` 0.14.3 for text blobs and `sha2` for addresses; the format becomes our own (`capture-cdp.md`, Work step 7), so `readTrace` and the Trace tab read the new manifest.
8. **Temp folders.** Rust gives each child program (headless shell, FFmpeg, sidecar) a temp folder under `<userData>/tmp/<pid>-<kind>-<id>` through `TMPDIR`/`TEMP`/`TMP`, deletes it when the child exits, and clears stale folders at start, as `clearStaleWorkerTemps` does today.
9. **Live frames.** Screencast frames for the editor PiP go from the capture task to the window over a `Channel`, throttled to the live fps setting.
10. **Tests.** Port the worker tests (2,259 lines) and `src/engine` tests (333); `executors.test.ts`'s rule (every `executes: true` node has an executor) becomes a Rust test over the node registry.

## Risks

- **Argument parity.** Every FFmpeg argument list is tested by string today; port the expected strings with the tests so a refactor can't silently change an encode.
- **Progress parsing.** FFmpeg runs with `-progress pipe:1 -nostats` today (`ffmpegArgs`, `src/worker/media/ffmpeg.ts`); the Rust reader must parse the same key/value lines on all three systems, tested with recorded samples.

## Sources

- `src/worker/engine/runner.ts`, `src/worker/executors/index.ts`, `src/worker/mix/render.ts`, `src/worker/render/picture.ts`, `src/traces/*` (read 2026-10-25); executor list printed from `EXECUTORS` with `tsx` (2026-10-25).
- crates.io API for `tokio-util` 0.7.20, `sha2`, `ebur128`, `symphonia`, `zstd` (2026-10-25).
