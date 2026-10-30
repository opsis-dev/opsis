# Renderer (v3)

Opsis v1 was Electron, v2 is Rust and Tauri with Remotion drawing the film. v3 is a greenfield renderer written for Opsis: one Rust engine that drives the recording browser for takes, stills and exports, with a film page Opsis owns end to end. Nothing about v2's film code is a constraint: elements, nodes, the page runtime, the authoring model and the build can all change if the result is faster or simpler. Opsis is pre-alpha (never published, never pushed, one local deb install for checks), so no file format, setting or element needs a migration.

Status: research done (2026-10-30), plan only. Work happens on `exp/rs-renderer` in the worktree `~/projects/opsis-v3`, building into the main checkout's target with `CARGO_TARGET_DIR=/home/panda/projects/demoflow/target` (never a symlink, never a change to the tracked `.cargo/config.toml`).

## How to read this plan

- Section 1 is v2 as it stands, with numbers. Section 2 is the research on Remotion, HyperFrames, Motion Canvas, the Rust toolchain crates and other people's experiments, plus measurements made on Opsis's own pinned browser. Section 3 is the v3 design. Section 4 is the work, in the order agreed: leaf packages first (smallest first), then the page runtime, then the Rust renderer, then cleanup. Section 5 lists measurements still to take, 6 decisions for the user, 7 risks, 8 sources.
- Each phase in section 4 ends with the same checks: `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, the element stills compared against the previous run (`bun run elements:render`, 138 stills, byte comparison), and the dev-engine tests of any crate touched. Visual changes are allowed (greenfield) but each one is looked at and named.
- Rules from `AGENTS.md` hold unless this plan changes them on purpose; every such change updates `AGENTS.md` in the same commit.
- Remotion is leaving, so it informs nothing below except as a record of what v2 does today. Its licence also forbids copying or modifying its code to sell a derivative, so no Remotion code is copied into v3; everything replacing it is written for Opsis. HyperFrames (Apache-2.0) and Motion Canvas (MIT) may be borrowed from with their notices in `THIRD_PARTY_NOTICES.md`.

## 1. v2 today

### 1.1 How an export is made

1. Compile produces a `TimelineSpec`; `hosted` turns `project:` and `prepared:` refs into addresses on the render file server.
2. The render executor (`crates/render/src/executors.rs`) works out the picture key (`picture_key`, `crates/render/src/picture.rs`: the spec without its audio, mix and asset address, `PICTURE_VERSION`, `PREPARED_VERSION`, the app version) and, when `.opsis/cache/pictures/<key>` has nothing, asks the sidecar for a render.
3. The sidecar (`apps/sidecar`, Bun, one JSON message per line) calls `selectComposition`, then `renderMedia` or `renderStill` from `@remotion/renderer` (`apps/sidecar/src/render.ts`): `serveUrl` is the prebuilt bundle, `browserExecutable` the engine's headless shell, `chromiumOptions.gl` the export's `Gl` (`angle` by default; `swangle`, `egl`, `vulkan`, `swiftshader`), `concurrency: 1`, codecs `h264 | h265 | vp8 | vp9 | prores | gif`, `crf`, `x264Preset`, `pixelFormat` (`yuv444p`), ProRes `proxy | light | standard`, `everyNthFrame`, `numberOfGifLoops`, `muted`; stills `png | jpeg` with `scale` and `jpegQuality`; the licence key from Settings (`"free-license"` when empty).
4. Remotion's renderer opens a tab per worker, sets each frame, waits, takes a picture and pipes it to the FFmpeg inside Remotion's compositor.
5. Opsis adds the master stem with its own FFmpeg (`mux_args`, `crates/media/src/codecs.rs`), writes chapters in the finishing copy (`finish_args`), and saves the result.

Sound never comes from Remotion: the mix is rendered in Rust (`crates/media/src/mix/render.rs`) and muxed afterwards. `DemoTimeline` mounts one `<Audio>` for `spec.mix` only so Remotion knows the film has sound.

### 1.2 What ships because of Remotion

| Item                                                                            | Size                                   | Notes                                                                                                                               |
| ------------------------------------------------------------------------------- | -------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| `resources/binaries/opsis-render-<triple>` (the sidecar, `bun build --compile`) | 87 MB                                  | about 89 MB of the release-plan measurement is the Bun runtime                                                                      |
| `resources/compositor` (Remotion's Rust compositor with its FFmpeg)             | 27 MB                                  | GPLv2 FFmpeg: `COPYING.GPLv2`, `SOURCES.md`, `licences/compositor-sources.json`, the CI `compositor-sources` job, release section 7 |
| `remotion-bundle/`                                                              | 38 MB with `public/`                   | `@remotion/studio` is 7.17 MB (28%) of a development bundle though renders never show it; mediabunny 3.38 MB, three 2.48 MB         |
| The deb as a whole (2026-10-27)                                                 | 103.8 MB download, 218.7 MiB installed |                                                                                                                                     |

Rules and workarounds that exist only for Remotion: every `remotion`/`@remotion/*` package pinned to one exact version; TypeScript pinned to 6.0.3 (Remotion's esbuild loader reads `tsconfig.json` through `typescript.sys`); "Remotion ignores tsconfig paths" (no `@` alias in the video); `#video/` instead of `#/` (Rspack rejects `#/`); the Remotion licence key and notice in Settings and the first-run screen; the section 7 permission at the top of `LICENSE`; the engine pin read from `@remotion/renderer`'s `TESTED_VERSION` (`scripts/engine-downloads.ts`); renders at `concurrency: 1`; users at companies with more than three employees needing a Remotion company licence.

### 1.3 What the film code uses from Remotion

`packages/video/src` has 304 source files (166 of them element components); 184 import the `remotion` core package and 48 import `@remotion/*` packages.

Core (`remotion`), by import count: `interpolate` 89, `AbsoluteFill` 88, `useVideoConfig` 68, `useCurrentFrame` 61, `Easing` 47, `spring` 37, `Img` 22, `random` 18, `getRemotionEnvironment` 8, `Sequence` 7, `continueRender` 6, `delayRender` 6, `interpolateColors` 5, `staticFile` 4, `OffthreadVideo` 4, `Interactive` 3, `EffectDescriptor` 3, `useDelayRender` 2, `SpringConfig` 2, and one each of `Composition`, `Freeze`, `Loop`, `registerRoot`, `useRemotionEnvironment`, `isHtmlInCanvasSupported`, `cancelRender`, `Internals` (the test harness only).

Leaf packages, ordered by the size of their own source in Remotion's repository (`packages/<name>/src`, tests excluded), which is the order phase 1 removes them in:

| #   | Package                    | Own source | Opsis files / uses | What Opsis uses                                                                                                                                                            | Depends on                                                    |
| --- | -------------------------- | ---------- | ------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| 1   | `@remotion/tailwind-v4`    | 64 lines   | 1 / 1 (sidecar)    | `enableTailwind`                                                                                                                                                           | css-loader, style-loader, tailwindcss, `@tailwindcss/webpack` |
| 2   | `@remotion/noise`          | 95         | 2 / 4              | `noise3D`                                                                                                                                                                  | simplex-noise                                                 |
| 3   | `@remotion/light-leaks`    | 336        | 1 / 1              | `LightLeak`                                                                                                                                                                | `@remotion/effects`                                           |
| 4   | `@remotion/lottie`         | 402        | 2 / 4              | `Lottie`, `LottieAnimationData`                                                                                                                                            | lottie-web (peer)                                             |
| 5   | `@remotion/mac-cursors`    | 459        | 1 / 1              | `resolveCursor`                                                                                                                                                            |                                                               |
| 6   | `@remotion/motion-blur`    | 495        | 1 / 2              | `CameraMotionBlur`                                                                                                                                                         |                                                               |
| 7   | `@remotion/three`          | 751        | 1 / 2              | `ThreeCanvas`                                                                                                                                                              | `@react-three/fiber` (peer)                                   |
| 8   | `@remotion/rough-notation` | 1,526      | 1 / 10             | `Box`, `Circle`, `CrossedOff`, `Highlight`, `Underline`                                                                                                                    | roughjs, `@remotion/paths`                                    |
| 9   | `@remotion/paths`          | 5,737      | 3 / 19             | `evolvePath`, `getBoundingBox`, `getLength`, `getPointAtLength`, `getTangentAtLength`, `interpolatePath`, `normalizePath`, `reversePath`, `scalePath`, `translatePath`     |                                                               |
| 10  | `@remotion/transitions`    | 5,954      | 26 / 69            | `TransitionSeries`, `TransitionPresentation`, `TransitionPresentationComponentProps`, `linearTiming`, `springTiming`, `fade`, `slide`, `wipe`, `flip`, `iris`, `clockWipe` | `@remotion/shapes`, `@remotion/paths`                         |
| 11  | `@remotion/media`          | 12,241     | 3 / 3              | `Video` (recordings, with `effects`), `Audio` (the mix)                                                                                                                    | mediabunny (MPL-2.0)                                          |
| 12  | `@remotion/effects`        | 47,965     | 2 / 4              | `colorCorrection`, `noise`, `vignette` (Picture look), `regionBlur` (Blur areas)                                                                                           | core `EffectDescriptor`                                       |

The sidecar also uses `@remotion/renderer` (`renderMedia`, `renderStill`, `selectComposition`, `makeCancelSignal`) and `@remotion/bundler` (`RspackOverrideFn`); those go in phase 3.

### 1.4 What Opsis already owns (the base v3 builds on)

`crates/capture` (8,188 lines) already is most of a frame-accurate browser renderer:

- **Browser.** One engine: Chrome Headless Shell 157.0.8080.0 (`crates/engine/src/downloads.generated.json`; Remotion's build on Linux x64/arm64 and Apple Silicon, Google's Chrome for Testing elsewhere), launched with Playwright's switches merged by `effective_switches` (`crates/capture/src/browser.rs`), children remembered and ended with the app (`crates/engine/src/children.rs`), child temp folders.
- **CDP.** Our own client over chromiumoxide (`crates/capture/src/cdp.rs`): evaluate in the main world, an isolated utility world with Playwright's injected script, `call_on`, screenshots.
- **One clock per take.** `TakeClock` (`take_clock.rs`) with real and stepped modes; smooth takes (`smooth.rs`) hold the page clock (`Clock::hold_for_frames`), set every CSS and web animation's rate to 0 and step them one frame at a time (`STEP_FRAME`, and `STEP_OVERLAYS` for the injected overlays' closed shadow root), keep page time across documents (`page_time_for_new_pages`), and take each frame's picture with `Page.captureScreenshot` at an explicit clip and scale. Measured: a 30 s take of a busy 1920×1080 page at 2× took 36 s with Standard (JPEG), 62 s with Lossless (PNG), 149 s with Full detail.
- **Encoding.** `FrameEncoder` (`encoder.rs`) pipes JPEG or PNG frames into ffmpeg-static at a constant rate without ever making the browser wait.
- **Serving.** The render file server answers project files and prepared files (`crates/prepare`: typefaces, Draco/meshopt models to plain GLB, flattened SVG, downsized textures), keyed by source size, time and `PREPARED_VERSION`.
- **Media.** `opsis-media`: FFmpeg argument builders, grade filter graphs (`crates/media/src/grade.rs`), mixing, muxing; `opsis-probe` reads media in Rust.
- **Caching.** Node keys, picture keys, prepared keys, all sha256.

What it lacks is a film page of its own, a seek handshake with that page, video frames for that page, and the export plumbing the sidecar does today.

## 2. Research

Clones (shallow, `~/.cache/opsis-research`): Remotion `3b7e700` (2026-10-29, 870 MB), HyperFrames `9fd90d2` (2026-10-30, 406 MB), Motion Canvas `7b91435` (2026-07-02, 9.4 MB).

### 2.1 Remotion (source-available, Remotion License)

- **Render loop.** `renderMedia` → `renderFrames` opens a pool of pages (`Pool`, `nextFrameToRenderState`: each free page takes the next unrendered frame, failed frames go back with `MAX_RETRIES_PER_FRAME`), `cycleBrowserTabs` brings tabs to the front so Chrome doesn't throttle them. Per frame (`seek-to-frame.ts`): wait for `window.remotion_renderReady === true` (no open `delayRender` handles, polled with `waitForFunction`), call `window.remotion_setFrame(frame, composition, attempt)` (React state change plus a `delayRender` released on the next `requestAnimationFrame`), wait for ready again, then `document.fonts.ready`. Timeouts report the open `delayRender` labels.
- **Picture.** `Page.captureScreenshot` (`screenshot-task.ts`) with `optimizeForSpeed`, `captureBeyondViewport`, `fromSurface` (needed above 8192 px), the body background set black, or transparent for PNG/WebP. A fast path needs Remotion's patched Chromium and patched FFmpeg: frames go through POSIX shared memory (or file-backed pools with `-pool_dir` where `/dev/shm` is missing) straight into FFmpeg (`remotionFrameSlot`, `remotionFrameId`).
- **Encoding.** Frames piped to FFmpeg; "parallel encoding" pre-stitches while frames render; long renders split into chunks concatenated afterwards, with extra frames rendered at chunk edges so AAC can be joined seamlessly (`get-extra-frames-to-capture.ts`).
- **Compositor.** A long-running Rust process on rust-ffmpeg: opens videos (`opened_video_manager`), a frame cache with a memory budget, rotation, HDR tone mapping, audio extraction, silence detection, metadata. `OffthreadVideo` asks it for frames over the renderer's HTTP server.
- **`@remotion/media`.** 12k lines over mediabunny: decodes video and audio in the page with WebCodecs and draws onto a canvas; Opsis's recordings play through it and its `effects` prop runs `@remotion/effects` WebGL passes (48k lines) on each frame.
- **Web renderer.** `@remotion/web-renderer` renders entirely in the page: it draws the DOM into a canvas with `drawElementImage` (Chromium's HTML-in-canvas, `CanvasDrawElement`) or its own DOM composer, and encodes with WebCodecs through mediabunny, no screenshots and no FFmpeg.
- **Core sizes** (for scale, not to copy): `interpolate.ts` 1,484 lines, `interpolate-colors.ts` 729, `Sequence.tsx` 1,290, `Img.tsx` 777, `delay-render.ts` 216, `spring/` 380, `easing.ts` 168, `random.ts` 46; `TransitionSeries.tsx` 1,053, `ThreeCanvas.tsx` 258, `Lottie.tsx` 236, motion blur 489. Most of that is the Studio, the Player and premounting; Opsis needs a fraction.
- **Licence.** Free for individuals, companies up to 3 employees and non-profits; "It is not allowed to copy or modify Remotion code for the purpose of selling, renting, licensing, relicensing, or sublicensing your own derivate of Remotion."

### 2.2 HyperFrames (Apache-2.0, HeyGen)

- **Authoring.** HTML compositions with data attributes; animation through seekable timelines registered on `window.__timelines` (GSAP by default). The page exposes `window.__hf = { duration, seek }`; "the renderer never calls play()". Adapters implement `{ id, init?, getDurationFrames, seekFrame, destroy? }`. Time is quantized `Math.round(time * fps) / fps`. Rules: no `Date.now()`, no unseeded `Math.random()`, no network at render time; Google Fonts imports rewritten to local copies; the runtime bundle checked by sha256.
- **Engine** (`packages/engine`, Node and Puppeteer, `frameCapture.ts` alone is 5,634 lines). Three capture modes:
  - `beginframe`: `HeadlessExperimental.beginFrame` runs layout, paint, composite and the screenshot in one call and returns `hasDamage`. Linux with chrome-headless-shell only; flags `--deterministic-mode --enable-begin-frame-control --run-all-compositor-stages-before-draw --disable-threaded-animation --disable-threaded-scrolling --disable-checker-imaging --disable-image-animation-resync --enable-surface-synchronization`. With begin-frame control the main thread stops ticking by itself, so `document.fonts.ready` and rAF-based polling hang; they warm up with `beginFrame` every 33 ms and `noDisplayUpdates: true`, poll with evaluate plus `setTimeout`, and start capture at a frame time past the warm-up. A liveness probe falls back to screenshots when the domain is missing; on macOS and Windows Chrome "either crashes under --deterministic-mode or has its own issues", so they use screenshots there.
  - `screenshot`: `Page.captureScreenshot` with "did the frame land" heuristics (fonts, computed styles, pixel hashes); their own write-up calls it "not the most robust".
  - `drawelement`: `canvas.drawElementImage` reads DOM paint records into a `<canvas layoutsubtree>` around the composition, an in-page `OffscreenCanvas` worker encodes JPEG; "~46% faster than Page.captureScreenshot on local GPU", pixel-perfect alpha on GPU, falls back on SwiftShader with transparency.
- **Per frame** (`prepareFrameForCapture`): seek, decode dynamic CSS background images, a before-capture hook (video frame injection), wait for pending seek handlers, wait for colour-grading LUTs, then capture. Readiness checks for images, videos, fonts and Tailwind.
- **Video.** "Letting a browser play <video> at render time does not work." FFmpeg extracts each video's frames before capture (HDR to SDR tone mapping, playback-rate maps, metadata cache); during capture each `<video>` is hidden and an `<img>` with the frame is injected with its computed styles copied; frames reach the page through intercepted requests under a private URL prefix rather than CDP data URIs.
- **Static frames.** `computeStaticFrameSet` predicts from the GSAP timelines which frames can't differ from the previous one and reuses its buffer; any video, canvas, WebGL or non-GSAP animation disqualifies the whole composition; a verification pass samples some of the predicted static frames.
- **Parallelism.** N Chrome processes on frame ranges, MP4 chunks concatenated; worker count from CPU and memory (never below 6 by default); a reorder buffer feeds frames to FFmpeg in order (`streamingEncoder`, `-f image2pipe`); video-heavy compositions drop to one worker because Chrome can't seek many `<video>`s at once.
- **Other.** Sub-frame motion blur: K captures across a shutter window (angle 180°, phase −90°, 16 samples by default) averaged per output frame, which blurs rotation, scale and opacity too, not only translation. HDR capture. Chrome flags that stop background throttling (`--disable-background-timer-throttling`, `--disable-backgrounding-occluded-windows`, `--disable-renderer-backgrounding`), `--force-color-profile=srgb`, `--font-render-hinting=none`.
- **Field reports.** Issue #4584 (open): with a hardware GPU and 6 or more workers starting together, every `beginFrame` probe times out (0.4 s for one browser plus about 0.33 s per browser starting alongside, against a 2000 ms deadline) and the render silently falls back to screenshots at about twice the time (60 s clip: 10.3 s with beginFrame against 22.2 s); retrying the probe once fixed it (9 of 9 renders). Issue #294: "beginFrame wasn't found" was a non-headless-shell build; the fix was a probe and fallback, the "Chromium 147" claim was never confirmed.

### 2.3 Motion Canvas (MIT)

- Scenes are generator functions; values are signals (reactive, computed values cached by dependency); drawing is a 2D scene graph on `<canvas>`; layout measures nodes in a hidden DOM `div` (flexbox by the browser) and draws on the canvas.
- Seeking replays the current scene's generator from its first frame up to the target (`PlaybackManager.seek`), with cached scenes; there is no random access to a frame.
- Time events (`waitUntil('event')`) are stored in a metadata file and dragged in the editor's timeline (`EditableTimeEvents`), like Opsis's Timeline handles.
- No headless renderer: export runs in the editor's browser tab; the FFmpeg exporter reads `getImageData` per frame and sends it through the Vite dev server's HMR channel to an FFmpeg process. Last commit 2026-07-02.

### 2.4 Other people's experiments

- **scorsese** (Rust, issue #775, closed 2026-10-06): headless Chromium as an external program, CDP hand-written over `--remote-debugging-pipe` ("No CDP crate"), the clock owned by a JS shim (`performance.now`, `Date`, timers, rAF, every Web Animation paused and seeked) because "neither CDP virtual time nor beginFrame's own clock is deterministic"; the shim clock starts above zero (anime.js treats a rAF timestamp of 0 as not started); frames from `beginFrame` under `--deterministic-mode` (prime with one or two begin-frames, early ones have no damage), fallback `Page.captureScreenshot` plus `--disable-threaded-animation`; SwiftShader everywhere; a virtual origin served entirely through the Fetch domain, failed requests reported, never silent; fonts limited with `FONTCONFIG_FILE`; cache keyed by page bytes, every loaded file, raster, fps, duration, Chromium version and shim version. Numbers: about 3.4 min per minute of video on SwiftShader, byte-identical across runs; 2.2 min for 60 s with 4 tabs.
- **Chromium headless-dev** ("screenshot functionality is extremely slow", 2022): about one 1920×1080 screenshot a second on a shader page; the Chromium developer's answer: GPU readback latency and reading the whole viewport, "one of the larger contributing factors is PNG encoding", "JPEG is considerably faster than PNG"; PNG with no row filter and compression 1 about 2× faster.
- **vidium** records with `Page.startScreencast` (real time, drifts), not frame-exact.

### 2.5 Measured on Opsis's own browser (2026-10-30)

A Bun script speaking CDP directly to the pinned Chrome Headless Shell 157.0.8080.0 (Remotion's Linux x64 build), one 1920×1080 page with a moving gradient box with a shadow and a text line, 60 frames, SwiftShader allowed:

| Capture                                | Format   | ms per frame | fps  | base64 per frame |
| -------------------------------------- | -------- | ------------ | ---- | ---------------- |
| `Page.captureScreenshot` (after a rAF) | JPEG q80 | 77.9         | 12.8 | 36 KiB           |
| `Page.captureScreenshot` (after a rAF) | PNG      | 74.5         | 13.4 | 200 KiB          |
| `HeadlessExperimental.beginFrame`      | JPEG q80 | 30.2         | 33.2 | 36 KiB           |
| `HeadlessExperimental.beginFrame`      | PNG      | 34.0         | 29.4 | 200 KiB          |

- `beginFrame` works on our pinned build and was about 2.5× faster than `captureScreenshot` on this page.
- `hasDamage` was true for all 60 frames even when frames 1 to 29 changed nothing, when every call asks for a screenshot. Static-frame skipping therefore can't rely on Chrome reporting damage; it has to come from the spec.
- One page, one machine, no GPU flags: section 5 lists the measurements still needed (real elements, recordings, WebGL, `drawElementImage`, parallel tabs).

### 2.6 The JavaScript and CSS toolchain as Rust crates

- **Rolldown** (oxc-based bundler, the engine under Vite's future): published as the `rolldown` crate; the Node and WASM bindings are a separate `rolldown_binding` crate. The project says issues that only affect Rust-crate use "will not be worked on as a team and will be closed", but pull requests are accepted. No published Rust usage example.
- **oxc**: `oxc` with the `transformer` feature (TypeScript stripping, JSX, lowering, define/inject), `oxc_codegen` (with source maps), `oxc_resolver`, `oxc_minifier`. Not a bundler by itself; Rolldown is the bundler built on it.
- **Rspack**: `rspack`, `rspack_core` and friends are on crates.io (0.102.x on docs.rs), all pinned to one exact version, mostly undocumented, APIs change a lot between releases, no embedding guide; the Node layer is `rspack_binding_api`.
- **swc**: `swc_core` crates usable from Rust; `swc_bundler` exists but is not where the ecosystem is going.
- **Tailwind v4**: Oxide (`@tailwindcss/oxide`) is the Rust file scanner; CSS generation is still the TypeScript compiler, and the crate isn't built for direct Rust use. The official route without Node is the standalone CLI binary. A community Rust engine (ironframe) claims v4.1 compatibility, unverified. `lightningcss` (Rust) does CSS transforms and minification.
- **Native 2D**: Vello (GPU compute 2D renderer on wgpu, pre-1.0, needs compute shaders or its CPU path), Parley (text layout), Taffy (flexbox/grid). Nobody found using them for frame-by-frame video export; a Motion Canvas-like engine on Vello and Typst exists.

### 2.7 What v3 takes from each

| Idea                                                                    | From                                 | In v3                                                                                       |
| ----------------------------------------------------------------------- | ------------------------------------ | ------------------------------------------------------------------------------------------- |
| Seek, never play; the page exposes one seek function and a ready signal | Remotion, HyperFrames                | `window.opsis.seek(frame)` returning when the frame is ready                                |
| `beginFrame` capture with a probe, retry and fallback                   | HyperFrames, scorsese, measured 2.5× | Linux capture path; screenshot path elsewhere; probe retried once (HyperFrames #4584)       |
| Our own deterministic clock shim, not CDP virtual time                  | scorsese, Opsis smooth takes         | the smooth-take clock, already ours, drives renders too                                     |
| Video frames decoded outside the page                                   | HyperFrames, Remotion's compositor   | ffmpeg-static decodes in Rust, the file server serves frames; also fixes HEVC (TODO.md bug) |
| Effects on footage done before the page                                 | Opsis's grade nodes                  | Picture look and Blur areas as FFmpeg pre-passes (prepared files)                           |
| Static frames skipped from knowledge of the timeline                    | HyperFrames                          | from the compiled spec, which knows every item's animation span                             |
| Sub-frame motion blur in the renderer                                   | HyperFrames                          | a renderer option on any span, replacing per-element tricks                                 |
| Workers on frame ranges with a reorder buffer, chunks concatenated      | Remotion, HyperFrames                | after one-tab is fast; WebGL contexts and GPU probes limit it                               |
| `drawElementImage` capture and in-page encoding                         | HyperFrames, Remotion web renderer   | measured, then chosen per platform if faster                                                |
| Editable time events stored beside the scene                            | Motion Canvas                        | already Opsis's Timeline handles                                                            |
| Every request served by us, failures reported                           | scorsese                             | the render file server; nothing loads from the internet (already a rule)                    |
| Bundle the film in Rust                                                 | Rolldown, oxc                        | optional, last; see 3.5                                                                     |

## 3. v3 design

### 3.1 Shape

```
compile (Rust) ──► FilmSpec ──► film page (static, served by the render file server)
                                   ▲ seek(frame) / ready                 │ <img> frames of videos and takes
Rust FrameDriver (opsis-capture) ──┘                                     │ prepared files (models, fonts, footage after grading and blur)
   │ beginFrame (Linux) or captureScreenshot / drawElementImage           ▼
   ▼                                                              Rust frame server (ffmpeg-static decode, cache)
FrameEncoder ──► ffmpeg-static ──► picture file ──► mux master stem, chapters ──► export
```

One Rust process does takes, stills, exports and (later) a preview; no Node or Bun at run time; one FFmpeg (ffmpeg-static), one browser.

### 3.2 The film page and its runtime

- **One seek protocol.** `window.opsis = { seek(frame, subFrame?) → Promise<void>, duration, errors }`. `seek` resolves when: the view has committed the frame, every held load is released (images decoded with `img.decode()`, fonts loaded, models and textures loaded, video frames present), and `document.fonts.ready`. Holds carry labels; a timeout reports the open ones, like v2's `delayRender` labels. Under begin-frame control nothing polls with rAF (2.2); the driver polls with evaluate.
- **One clock.** Elements read the frame from the runtime. Because the driver owns the page clock (the smooth-take machinery: page clock held, every CSS and web animation stepped), elements may also use CSS keyframes, CSS transitions and the Web Animations API, which v2 bans; they render deterministically because the driver steps them. This changes the rule "every animation is a pure function of `useCurrentFrame()`" to "every animation follows the film clock"; the oxlint rules banning `motion`/`animate-*` in the film are reconsidered with it.
- **Authoring model** (decision 6.1). React compatibility is not a constraint. The candidates, to be decided by the measurements in 5.1:
  - A. React with Opsis's own small runtime (frame and config contexts, spans, holds, maths). Keeps the 166 components close to what they are.
  - B. A signal-based DOM runtime (no virtual DOM diff per frame): each element is a function that builds its DOM once and binds properties to the frame; per-frame cost is only the bound properties.
  - C. Native Rust drawing (Vello, Parley, Taffy) for elements, the browser only for takes and HTML-heavy elements. No browser cost for most films, but text, CSS, SVG, WebGL and three.js fidelity have to be rebuilt.
  - Recommended starting point: keep the browser (takes need it anyway, and CSS, text, SVG, WebGL and three.js come for free), measure where a frame's time goes with A, and move to B only if the view layer is a real share of the frame time. C stays an experiment for a fast draft preview.
- **Spans, not sequences.** The compiled spec already places every item in time; the runtime mounts an item only inside its span (plus a premount margin for loads), so a film's DOM holds only what's on screen.
- **Maths.** Interpolation with clamping and easing curves, damped springs, colour interpolation in OKLCH, seeded random: written for Opsis in `common/` (plain TypeScript, also used by the editor), with property tests.
- **Assets.** `app:`, `project:` and `prepared:` refs resolved by Rust as today (`hosted`); the page gets plain addresses.

### 3.3 Driving and capture (Rust)

- **FrameDriver** in `opsis-capture`, shared with smooth takes: launch, warm-up, per-frame seek, readiness, capture, retries.
- **Capture** (decision 6.3): `beginFrame` on Linux (warm-up with `noDisplayUpdates`, probe once and retry before falling back, never fall back silently: the run log says which path ran); `Page.captureScreenshot` with JPEG and `optimizeForSpeed` on macOS and Windows; `drawElementImage` into a canvas with in-page JPEG encoding where 5.1 shows it faster. PNG only for alpha exports.
- **Transport.** CDP base64 today; measure binary transport (the page POSTs frames to the render file server) against it.
- **Static frames.** Compile marks, per frame, whether anything can change: an element past its animation end and before its exit, no camera move, no video, no recording, no shader time. Frames proven static reuse the previous picture without a seek or capture.
- **Workers.** One tab first; then N tabs or browsers on frame ranges with an ordered hand-over to the encoder; video-heavy films and WebGL-heavy films measured before raising N (v2 measured one tab fastest for shader films).
- **Stills, thumbnails, preview.** The same driver for one frame; a warm tab can answer frame requests for the editor (the deferred "see the film before exporting").
- **Long renders.** Chunks in fresh browsers if memory grows (TODO.md "Unverified": Chrome's `angle` leak on long renders).

### 3.4 Media in the film

- **Video and takes.** Rust decodes frames with ffmpeg-static (seek, scale to the drawn size, cache by file stamp and time), served by the render file server; the page shows them as images. This works for HEVC, which the headless shell can't decode (TODO.md bug), and for HDR (tone mapping in FFmpeg).
- **Picture look and Blur areas.** Applied to the footage before the page sees it, as prepared files: Picture look through the grade filter builders (`crates/media/src/grade.rs`), Blur areas as timed `boxblur` regions in the take's own coordinates, so they stay on the page while the camera moves. No WebGL effects pipeline in the page.
- **Sound.** Unchanged: Rust mix, master stem, mux.

### 3.5 Build

- The film page becomes a plain static app: an `index.html`, one module bundle, `public/`. No Remotion Studio (7.17 MB gone), no Remotion bundler.
- Tailwind leaves the film (11 files use `className`); elements style themselves with style objects. The editor keeps Tailwind through its Vite plugin.
- Bundler: Vite (already used for the editor) first. Later, optionally, a Rust build step with Rolldown and oxc (2.6), which would remove Rspack and `@remotion/bundler` from the toolchain; it's dev-time only (the bundle ships prebuilt), so it's a tooling simplification, not a run-time gain.

### 3.6 What replaces each leaf package (phase 1)

| #   | Package                    | Replacement                                                                                                                                                                                                                                      |
| --- | -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1   | `@remotion/tailwind-v4`    | The film stops using Tailwind: the 11 files that use `className` move to style objects, `video.css` loses `@import "tailwindcss"`, the sidecar's bundler override drops `enableTailwind`                                                         |
| 2   | `@remotion/noise`          | `simplex-noise` directly (MIT), seeded per element                                                                                                                                                                                               |
| 3   | `@remotion/light-leaks`    | Our own light-leak shader element with colour settings (the audit's missing "light-leak colours" become possible)                                                                                                                                |
| 4   | `@remotion/lottie`         | `lottie-web` directly, `goToAndStop(frame, true)` per frame, held until loaded                                                                                                                                                                   |
| 5   | `@remotion/mac-cursors`    | Our own cursor SVGs drawn from Lucide's ISC shapes, keyed by CSS cursor keyword                                                                                                                                                                  |
| 6   | `@remotion/motion-blur`    | Our own sub-frame trail component now; renderer-level sub-frame blur in phase 3                                                                                                                                                                  |
| 7   | `@remotion/three`          | `@react-three/fiber` directly with `frameloop="never"` and an explicit render per frame, held until assets load                                                                                                                                  |
| 8   | `@remotion/rough-notation` | `roughjs` directly with our own path drawing and seeds                                                                                                                                                                                           |
| 9   | `@remotion/paths`          | Our own SVG path helpers in `lib/paths` (length, point and tangent at length, bounding box, normalise, reverse, scale, translate, interpolate, evolve), property-tested                                                                          |
| 10  | `@remotion/transitions`    | Our own transition series and presentation interface (the 24 presentations are ours already); `fade`, `slide`, `wipe`, `flip`, `iris`, `clockWipe` rewritten with the settings the audit couldn't offer (direction, start angle, centre, colour) |
| 11  | `@remotion/media`          | Recording and video frames from Rust (3.4); the `<Audio>` for the mix disappears with Remotion                                                                                                                                                   |
| 12  | `@remotion/effects`        | Picture look and Blur areas as FFmpeg pre-passes (3.4)                                                                                                                                                                                           |

Until phase 2, replacements still read the frame through Remotion's hooks; phase 2 swaps those for the runtime's.

## 4. Work, in order

### Phase 1: leaf packages, smallest first

One package at a time, in the order of 1.3 and 3.6. Each step: replace, remove the package with `bun remove`, update `THIRD_PARTY_NOTICES.md` and `AGENTS.md`, run the checks, compare the element stills, and name every still that changed and why. The user stages each step; it's committed before the next starts.

### Phase 2: the page runtime

1. Measure first (5.1) with the current elements under a stub runtime.
2. Decide the authoring model (6.1).
3. Write the runtime: seek protocol, holds with labels, spans, maths in `common/`, config.
4. Move every element and part to it (184 files import Remotion's core today); delete `registerRoot`, `Composition`, `Freeze`, `Loop`, `Interactive`, `getRemotionEnvironment` uses.
5. A test harness that renders an element at a frame without any browser test runner hacks (replaces `core/test-frame.tsx`).

### Phase 3: the Rust renderer

1. FrameDriver in `opsis-capture` with the capture paths of 3.3, the probe and the fallback.
2. The frame server for video and takes, and the footage pre-passes (3.4).
3. Exports: picture through FrameEncoder, codecs (h264, h265, vp8, vp9, ProRes, GIF through `palettegen`/`paletteuse`), quality, every Nth frame, scale; then mux as today.
4. Stills.
5. Static-frame skipping from the spec.
6. Workers.
7. Remove the sidecar, `@remotion/renderer`, `@remotion/bundler`, the compositor staging, `scripts/video-bundle.ts`'s Remotion path.

### Phase 4: cleanup

- `LICENSE`: drop the section 7 Remotion permission.
- Settings and first-run: drop the Remotion licence key and notice.
- `plans/release.md`: section 7 (the compositor's FFmpeg source offer) goes; size numbers measured again.
- `scripts/engine-downloads.ts`: pin Chrome for Testing's headless shell ourselves (decision 6.4).
- `package.json`: lift the TypeScript 6.0.3 pin and the exact-version pins.
- `AGENTS.md`: the runtime boundary, animation and "one engine" sections rewritten for v3.
- `THIRD_PARTY_NOTICES.md`: remove Remotion entries, add HyperFrames if any of its code is adapted.

## 5. Measurements still to take

1. Where a 1920×1080 frame's time goes with real films: seek and commit, readiness, paint, capture, transport, encode; for a text element, a chart, a shader background, a recording with the camera, a 3D model.
2. `beginFrame` against `captureScreenshot` against `drawElementImage` with in-page JPEG, with `gl` set to `angle`, `swangle` and `egl`, at 1080p and 4K.
3. CDP base64 against binary frames posted to the file server.
4. Rust-decoded video frames as images against `<video>` seeking and against mediabunny in the page.
5. Static-frame savings on the example projects.
6. Tabs and browsers in parallel, with and without WebGL, with recordings.
7. Install size and memory against v2.

## 6. Decisions for the user

1. Authoring model: A, B or C (3.2). Recommended: A to start, B if measurements say the view layer matters.
2. CSS and web animations allowed in the film because the driver owns the clock (3.2).
3. Capture path per platform: `beginFrame` on Linux, screenshots on macOS and Windows, `drawElementImage` if faster (3.3).
4. Browser build: keep Remotion's Chromium builds (they have Remotion's shared-memory patch, which we won't use) or switch every platform to Google's Chrome for Testing headless shell.
5. Whether the preview comes back in v3 (3.3).
6. Whether to replace Vite with a Rust build step (3.5).

## 7. Risks

- **`HeadlessExperimental` is experimental.** It could change or go; the screenshot path must stay working and tested.
- **`drawElementImage` is experimental** (the `CanvasDrawElement` feature); Opsis already depends on it for canvas transitions.
- **No shared-memory frames.** Remotion's fastest path needs its patched Chromium and FFmpeg; v3 gives that up and wins time back elsewhere (beginFrame, static frames, footage pre-passes).
- **Determinism across machines.** Fonts (limit fallback), GPU differences (prefer SwiftShader for reference renders), the clock shim starting above zero.
- **Scope.** Phases 2 and 3 touch every element; the element stills and the dev-engine capture tests are the safety net.
- **Rolldown's Rust API is unsupported** by its maintainers; keep Vite as the default.

## 8. Sources

- Remotion `3b7e700`: `packages/renderer/src/{render-media,render-frames,seek-to-frame,take-frame,screenshot-task,next-frame-to-render,get-extra-frames-to-capture}.ts`, `packages/core/src/TimelineContext.tsx`, `packages/compositor/rust/`, `packages/media/`, `packages/web-renderer/src/{html-in-canvas,take-screenshot,render-media-on-web}.ts`, `LICENSE.md`.
- HyperFrames `9fd90d2`: `packages/engine/README.md`, `packages/engine/src/services/{frameCapture,screenshotService,browserManager,drawElementService,videoFrameExtractor,videoFrameInjector,videoFrameTransport,parallelCoordinator,streamingEncoder,motionBlur}.ts`, `packages/core/src/adapters/types.ts`, `packages/core/src/runtime/window.d.ts`, `docs/guides/performance.mdx`, the "vs Remotion" page.
- Motion Canvas `7b91435`: `packages/core/src/app/{Renderer,PlaybackManager}.ts`, `packages/core/src/scenes/timeEvents/EditableTimeEvents.ts`, `packages/core/src/signals/`, `packages/ffmpeg/`, `packages/2d/src/lib/components/Layout.ts`.
- HeyGen, "HTML to Video Was Not Easy: Here's How We Solved It" (2026-06-22), https://www.heygen.com/research/html-to-video
- HyperFrames issues #294 and #4584, https://github.com/heygen-com/hyperframes/issues/294, https://github.com/heygen-com/hyperframes/issues/4584
- scorsese issue #775, https://github.com/MatthewLacerda2/scorsese/issues/775
- Chromium headless-dev, "Chrome's screenshot functionality is extremely slow", https://groups.google.com/a/chromium.org/g/headless-dev/c/aVZw00nzxTU
- Rolldown Rust crates, https://rolldown.rs/apis/rust-crates; oxc transformer, https://oxc.rs/docs/guide/usage/transformer; Rspack project architecture, https://rspack.rs/contribute/development/project; Tailwind Oxide discussion #15254, https://github.com/tailwindlabs/tailwindcss/discussions/15254; Vello, https://docs.rs/vello
- Cargo build cache, https://doc.rust-lang.org/cargo/reference/build-cache.html
- The capture benchmark script of 2.5 (scratchpad, not committed).
