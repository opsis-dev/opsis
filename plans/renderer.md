# Renderer

The film is drawn by Opsis's own renderer: Rust drives the recording browser over the film page (`apps/film`, on `@opsis/runtime`), captures each frame and encodes it with FFmpeg (`AGENTS.md`, The film page and the frame driver). This plan is what's left to make it faster and to decide its open questions.

Status: the renderer is on `exp/rs-renderer` in the worktree `~/projects/opsis-v3`, not merged into `main`. The worktree builds into its own `target` only (no `CARGO_TARGET_DIR`), and its `target` is deleted once it's merged. Performance is measured on release builds only, and only after the merge.

## How to read this plan

- Section 1 lists the measurements to take after the merge, section 2 the work they decide, section 3 decisions for the user, section 4 risks, section 5 sources.
- Each piece of work ends with the usual checks: `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, the element stills against the previous run (`bun run elements:render`, 138 stills, byte comparison) and the dev-engine tests of any crate touched (`crates/render/tests/frames.rs`, `three_stills.rs`, `run.rs`).
- Rules from `AGENTS.md` hold unless this plan changes them on purpose; every such change updates `AGENTS.md` in the same commit.

## 1. Measurements, after the merge

1. Where a 1920×1080 frame's time goes with real films: seek and commit, readiness, paint, capture, transport, encode; for a text element, a chart, a shader background, a recording with the camera, a 3D model.
2. `beginFrame` against `captureScreenshot` against `drawElementImage` with in-page JPEG, with `gl` set to `angle`, `swangle` and `egl`, at 1080p and 4K.
3. CDP base64 against binary frames posted to the file server.
4. Footage frames from the Rust decoder: time per frame for 1080p and 4K sources, decoder memory, and the restart cost of a seek.
5. Static-frame savings on the example projects.
6. Tabs and browsers in parallel, with and without WebGL, with recordings.

## 2. Work the measurements decide

- **Capture.** `drawElementImage` into a canvas with in-page JPEG encoding where measurement 2 shows it faster than `beginFrame`. Begin-frame screenshots ignore an emulated scale, so a scaled render takes a screenshot per frame; a scale set at launch (`--force-device-scale-factor`) would let scaled renders use `beginFrame` too.
- **Transport.** Binary frames posted by the page to the render file server, if measurement 3 shows CDP's base64 costs enough.
- **Static frames.** Compile marks, per frame, whether anything can change: an element past its animation end and before its exit, no camera move, no video, no recording, no shader time. Frames proven static reuse the previous picture without a seek or capture.
- **Workers.** N tabs or browsers on frame ranges with an ordered hand-over to the encoder; video-heavy and WebGL-heavy films measured before raising N (Remotion's renderer measured one tab fastest for shader films).
- **Long renders.** Chunks in fresh browsers if memory grows (TODO.md "Unverified": Chrome's `angle` leak on long renders).
- **More footage operations.** The footage chain can run more of the image operations (crop, rotate and flip, adjust, blur and sharpen, spotlight, magnify); each needs its own node design, since a crop changes the footage's size, which the camera and window frame depend on.

## 3. Decisions for the user

1. CSS and web animations in the film. The driver owns the page clock (the smooth-take machinery: page clock held, every CSS and web animation stepped), so elements could use CSS animations and the Web Animations API, which `AGENTS.md` bans today; the rule holds until this is decided.
2. Browser build: keep Remotion's Chromium builds (`crates/engine/src/downloads.generated.json`, pinned by `pin-downloads` for the version it's given) or pin Google's Chrome for Testing headless shell on every platform.
3. Whether a preview comes back: a warm tab of the frame driver answering frame requests for the editor (TODO.md, "A way to see the film before exporting").
4. Whether to replace Vite with a Rust build step (Rolldown and oxc). It's dev-time only, since the film page ships prebuilt, so it's a tooling simplification, not a run-time gain.

## 4. Risks

- **`HeadlessExperimental` is experimental.** It could change or go; the screenshot path must stay working and tested (`crates/render/tests/frames.rs` draws a still both ways and compares the bytes).
- **`drawElementImage` is experimental** (the `CanvasDrawElement` feature); the canvas transitions and filters depend on it.
- **Determinism across machines.** Fonts (limit fallback), GPU differences (prefer SwiftShader for reference renders).
- **Rolldown's Rust API is unsupported** by its maintainers; keep Vite as the default.

## 5. Sources

- HyperFrames `9fd90d2`: `packages/engine/src/services/{frameCapture,screenshotService,browserManager,drawElementService,parallelCoordinator,streamingEncoder}.ts`, `docs/guides/performance.mdx`.
- Remotion `3b7e700`: `packages/renderer/src/{render-media,render-frames,take-frame,screenshot-task}.ts`, for the parallel-tab and capture measurements it made.
- Chromium headless-dev, "Chrome's screenshot functionality is extremely slow", https://groups.google.com/a/chromium.org/g/headless-dev/c/aVZw00nzxTU
- Rolldown Rust crates, https://rolldown.rs/apis/rust-crates; oxc transformer, https://oxc.rs/docs/guide/usage/transformer
