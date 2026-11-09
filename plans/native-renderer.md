# Native renderer

How Opsis stops drawing films in the recording browser: every element, the camera, the terminal and the transitions become Rust that returns scene nodes, Skia draws them on the CPU, wgpu runs the shader effects and 3D, and finished frames go straight into the bundled FFmpeg. Node packs draw through the same scene contract ([node-packs.md](node-packs.md)).

Status: plan only. Nothing here is built yet. Facts were checked on 5 November 2026, in the libraries' own repositories and the Opsis code; everything this plan relies on is in it or linked from it.

## How to read this plan

- **Scene** comes first: the types every element returns, the clock and the caches. Everything else draws or produces them.
- **Skia**, **Text and layout** and **Effects and 3D** come next, then **Footage and terminal** and **Output**.
- **Elements** is the port of every element, the camera and the terminal view onto the scene contract.
- **Goldens and tests**, then **Cut-over**, close it. The browser film page goes in one step, after everything above passes.
- Each part has the same sections: **What it changes**, **Fit with Opsis** (In place / Missing), **Work**, **Decisions**, **Risks**, **Sources**.

## Conventions

These restate rules from `AGENTS.md` and `agent-docs/`; when they disagree with this plan, they win. When a part ships, its rules move into the docs that own the domain (`film.md`, `elements.md`, `terminal.md`, `footage.md`, `engine.md`, `runs.md`), and a new `agent-docs/scene.md` holds the scene contract, the clock and the caches.

- **Every frame is a pure function of the frame number.** An element is `fn(&Settings, &Clock, &Stage) -> Node`; no element reads a wall clock, a random source, a file or the network. Randomness is Opsis's `random(seed)`, ported exactly.
- **Rust owns the open project and every file, as today.** The renderer is Rust in the app's process tree; nothing draws a film in a page.
- **No comments**, in Rust, WGSL and every new file.
- **No number is converted with `as`**, and floats are compared exactly only through `exact_eq`, in renderer code as everywhere else.
- **Nothing uses the internet unless the user asks.** Fonts, images, models and footage reach the renderer from the project, the data folder and `public/`, as `app:` and project refs do today.
- **Checks when a part is done:** `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, plus the part's golden pictures.

## Decided

- **Skia draws every 2D frame, on the CPU.** It is the only library that covers every 2D feature the film uses on the CPU: all blend modes, layers with backdrop blur, every CSS filter function, turbulence, displacement and colour-matrix filters, `SkM44` perspective, OKLCH gradient interpolation (`SkGradient.h`), dithering, F16 surfaces. How Opsis calls Skia is open (see **Skia**).
- **wgpu runs every shader effect and the 3D**, from WGSL, on a GPU when there is one and otherwise on a software driver: the SwiftShader inside the pinned Chrome Headless Shell, the system's lavapipe on Linux, WARP on Windows.
- **Taffy lays out boxes, Parley lays out text**, from the same font bytes Skia draws.
- **The scene model is pure evaluation with content hashes, memo caches and a raster layer cache**, not a signal graph. Held frames get equal hashes by quantising before hashing: positions to 1/64 px, opacity in 8-bit steps; `spring` keeps its maths.
- **The browser film page is replaced in one go**, once every element, the camera, the terminal and every transition pass on the native renderer. There is no period with both renderers in the app.
- **Golden pictures are exact on one software setup**: the pinned SwiftShader and a fixed Skia CPU path. GPU runs compare with a tolerance.
- **No live preview in the editor in this plan.** A film is still watched as an export; a preview is a later decision.
- **Lottie plays through Skottie without expressions.** A file that contains any expression draws its keyframed values, and its card warns and points to Bodymovin's expression baking. Android, iOS and Windows Lottie don't run expressions either (Airbnb's `supported-features.md`), and the Lottie spec leaves them out (IANA `video/lottie+json`).
- **Element authoring is Rust for built-in elements** and WebAssembly components for packs, both returning the same scene nodes ([node-packs.md](node-packs.md)).

## Summary

| Part                 | What it changes                                                                    | Needs                 |
| -------------------- | ---------------------------------------------------------------------------------- | --------------------- |
| Scene                | scene types, clock and time maps, hashes, memo and layer caches, the resolve pass  | —                     |
| Skia                 | how Opsis calls Skia, the font manager, layers, filters, perspective               | Scene                 |
| Text and layout      | Parley and Taffy behind the scene's layout and text nodes                          | Scene, Skia           |
| Effects and 3D       | wgpu effects in WGSL, the software drivers, the 3D renderer, Skia–wgpu layers      | Scene, Skia           |
| Footage and terminal | decoded frames as image layers, the terminal drawn natively                        | Skia, Text and layout |
| Output               | raw frames into FFmpeg with colour tags, parallel chunks                           | Skia                  |
| Elements             | all 140 element and transition implementations, the camera, captions, the terminal | every part above      |
| Goldens and tests    | exact goldens on the software setup, tolerance on GPUs, render tests un-ignored    | Elements              |
| Cut-over             | delete the film page, the terminal page, the frame driver and the painter; docs    | everything            |

## Scene

### What it changes

- **A new crate, `opsis-scene`, holds what every element returns and how a frame is evaluated.** Its node types are the WIT `types` interface of [node-packs.md](node-packs.md): flat lists of nodes with child indices (WIT has no recursive types), CSS-like layout, boxes, paths, text runs, images and footage frames, SVG, Lottie, 3D models, WGSL effects over other nodes, slots for children and transitions, and per-node transform, opacity, blend mode, filters, backdrop filters, clip and mask.
- **Time is a value passed down, never a global.** `Clock` carries the frame (fractional, because remaps are), the rate and the length. `Sequence`, `Freeze`, `Loop`, group `remap`, `startFrom`, `loopEvery` and each motion-blur sample are pure maps from one clock to another. Elements registered with `.authored_clock` get a 30 fps clock, as `useAuthoredFrame` gives them today.
- **Every node gets a content hash when it is built.** A group's own transform and opacity stay out of its content hash, so moving or fading a group keeps what's inside cached.
- **Memo caches keyed by value**: text shaping, flex layout, path flattening, decoded images, parsed SVG, spring lengths. `measureSpring` loops frame by frame to the rest frame on every `spring({durationInFrames})` call today (`packages/runtime/src/spring.ts:76-111`).
- **A raster layer cache keyed by subtree hash and raster-scale bucket.** Because frame n+1 is known, a camera move rasters once at its largest scale. Footage and backdrop filters over moving content are never cached. A frame whose root hash equals the last one's reuses the last picture.
- **A resolve pass before the first frame** loads every font, image, SVG, model, Lottie file and footage decoder an element asks for, replacing `holdRender`, `useHeldLoad`, `untilPainted` and `document.fonts.ready`.

### Fit with Opsis

- **In place:** `spring` is closed form (`springProgress`), `random(seed)` is a hash, `interpolate` is stateless (`packages/runtime/src`), so frame n needs no earlier frame. Compile already produces a `TimelineSpec` that one `DemoTimeline` draws (`packages/video/src/demo-timeline.tsx`), and the flattener, cuts, remaps and envelopes are Rust (`crates/compile/src`).
- **Missing:** the scene crate, the clock maps, the hashing, both caches and the resolve pass. `DemoTimeline`'s tracks, `TransitionWrap`, `EnvelopeWrap`, `Group`, `CropBox`, `DesignBox` and `SceneSequence` become Rust that walks the spec and calls element functions.

### Work

1. `opsis-scene` with the node types, `Clock` and the clock maps, `Stage` (size, design size, overlay scale, shape insets), and the quantise-and-hash step.
2. The timeline walker: tracks, items with `from` and length, envelopes, transitions with their outgoing and incoming slots, groups with `remap`, `loopEvery`, `startFrom`, crop, design box, wrapper and sequencer.
3. Motion blur as N clocks for one subtree, accumulated with `sampleOpacity` weights (`camera/lib/shutter.ts` ported).
4. Memo caches with age-based eviction, and the layer cache with the planned raster scale.
5. The resolve pass, with the 30-second limit kept only for a footage decoder that stalls.
6. Property tests: a frame evaluated twice is equal; a frame evaluated after another equals a fresh evaluation; frame f equals frame f+1 after `animation_length`; the clock maps compose.

### Decisions

- **Where the scene types are written.** The WIT is a boundary, and AGENTS.md makes Rust the source of every type that crosses one. Either the Rust types are generated from the WIT by `wasmtime::component::bindgen!` and AGENTS.md records that exception, or they are hand-written in Rust and a test fails when they and the WIT disagree. Decide before Scene starts.
- **`comemo` or explicit hash keys** for the memo layer (`comemo` 0.5.1, MIT/Apache-2.0, thread-safe behind `parking_lot::RwLock`).

### Risks

- **A cache key that misses an input draws a stale picture.** Every input an element reads has to be in its hash; the property tests above and the goldens catch most of it.
- **Quantising changes pictures by up to 1/64 px and one 8-bit step**, within what the goldens are recorded at.

### Sources

- Skottie's seek and static properties: https://github.com/google/skia `modules/skottie/src/{Skottie.cpp,animator/Animator.cpp}`. Flutter's three-frame raster cache warm-up: https://github.com/flutter/flutter `engine/src/flutter/flow/raster_cache.h` (`access_threshold = 3`). `comemo`: https://github.com/typst/comemo `src/constraint.rs`, `src/accelerate.rs`.

## Skia

### What it changes

- **Skia's CPU raster backend draws every 2D frame** into an N32 or F16 surface: paths, strokes, dashes, every gradient kind with OKLCH interpolation, rounded rects and blurred shadows, `saveLayer` with opacity and blend modes and `SaveLayerRec::fBackdrop` for backdrop blur, `SkImageFilters` (blur, drop shadow, colour matrix, displacement map, merge, blend, offset, image), `SkPerlinNoiseShader` for turbulence, `SkM44` for perspective, `setDither`.
- **One font manager built from font bytes on every OS** (Fontations or an embedded FreeType), never the system's, so glyphs don't change by operating system. The film already renders unhinted (`--font-render-hinting=none`, `crates/capture/src/film.rs`).
- **Skottie plays Lottie files**, with no expression evaluator attached.

### Fit with Opsis

- **In place:** the film draws through Skia today, inside Chrome, so blur, gradient and anti-aliasing maths stay closest to current output. `usvg` (0.49) and `skrifa` are already dependencies (`crates/prepare`).
- **Missing:** Skia itself in the build, the drawing code for each scene node, the font manager, the Skottie bridge.

### Work

1. Skia in the build, by the route the decision below picks, on Linux x64 and arm64, Windows and macOS.
2. A painter from scene nodes to `SkCanvas` calls, with every filter, blend mode, clip, mask and perspective case listed in [Appendix: drawing features](#appendix-drawing-features).
3. The font manager from the project's and the app's font bytes; `family_for` keeps choosing faces as today ([fonts.md](../agent-docs/fonts.md)).
4. A fixed CPU path for golden runs: Skia picks AVX2+FMA code at run time on x86 (`src/core/SkOpts.cpp`, `Init_ml3`) and fuses `mad()` on AVX2, AVX-512 and NEON (`src/opts/SkRasterPipeline_opts.h`), so golden runs pin one path. x86 and ARM differ even then; goldens are recorded on x86 Linux.
5. Skottie for the Lottie element, with the expression check that feeds the card's warning.

### Decisions

- **How Opsis calls Skia: open.** No third-party sys crates or pins is the aim; Skia itself is given.
  - `skia-safe` (rust-skia 0.157.0 on Skia m157): its `unsafe` stays in the dependency, and its prebuilt binaries download at build time. pragmatrix wrote 449 of its 496 commits since 2026-05-05, its README calls it "a prerelease crate", and no prebuilt archive is CPU-only with Skottie.
  - Our own `cxx` wrapper over the Skia calls listed above, building Skia from a pinned commit with GN. It reuses the `cxx` bridge the editor uses for Qt ([ui.md](../agent-docs/ui.md)), and it is a third crate that needs `unsafe`, beside `opsis-vt` and `opsis-qt`.
- **Skia's text modules or Parley.** This plan uses Parley (see **Text and layout**); SkParagraph brings ICU and HarfBuzz into the Skia build.

### Risks

- **Skia's C++ API moves between milestones** (at m159 the gradient factories live only in `include/effects/SkGradient.h`).
- **Linking the system FreeType** can crash when the version differs from the one Skia was built with (rust-skia #1345, open since 2026-10-10); the font manager above avoids the system's.
- **No sourced measurement of Skia's speed on blur, filters, text or whole frames exists**; measure on the example projects early.

### Sources

- Skia (https://github.com/google/skia): `include/effects/{SkGradient.h,SkImageFilters.h,SkRuntimeEffect.h,SkPerlinNoiseShader.h}`, `include/core/SkCanvas.h`, `src/core/SkOpts.cpp`, `src/opts/SkRasterPipeline_opts.h`, `modules/skottie/include/Skottie.h`. rust-skia (https://github.com/rust-skia/rust-skia): `README.md`, `skia-bindings/build_support/`, issue #1345. Benchmarks: https://laurenzv.github.io/vello_chart/ (x86 and ARM data files).

## Text and layout

### What it changes

- **Taffy lays out every box**: flex and grid as CSS defines them. Its 1,689 fixtures take their expected numbers from Chrome for Testing (`scripts/gentest`).
- **Parley shapes and breaks text** (harfrust, ICU4X segmentation, `skrifa`, fontique fallback): rich runs, variable fonts, OpenType features, letter and word spacing, bidi, CSS whitespace. Its line breaks are tested against Chromium's (`parley_tests/linebreaking_cases`).
- **Text is a Taffy leaf whose measure function calls Parley**, and the same glyph runs are drawn by Skia from the same font bytes.
- **Elements that measure before laying out** (word-push, kinetic-center-build, slot-machine-roll, punch-lines, launch-anything's shader text reveal, the terminal view, which call `measureText` today) call the same measurer.

### Fit with Opsis

- **In place:** `skrifa` and the font library (`opsis-fonts`), `family_for` (`crates/compile/src/fonts/mod.rs`) and `face_for` (`crates/compile/src/elements/prepared.rs`), which the image nodes call (`crates/image/src/executors.rs`) and draw with `crates/image/src/text.rs`.
- **Missing:** Taffy and Parley in the workspace, the layout pass over scene nodes, the measurer.

### Work

1. Taffy and Parley pinned in the root table.
2. The layout pass: scene layout styles into Taffy, text leaves measured by Parley, results cached by hash.
3. Gradient-filled text (`background-clip: text`, 5 elements) as a clip from the glyph outlines.

### Decisions

- None beyond the decided Parley and Taffy.

### Risks

- **No `text-wrap: balance` and no hyphenation in Parley.** No element uses either today.
- **Taffy and Parley break their APIs each 0.x minor**, as their changelogs list.

### Sources

- Taffy (https://github.com/DioxusLabs/taffy): `README.md`, `CHANGELOG.md`, `test_fixtures/`, `scripts/gentest/src/main.rs`. Parley (https://github.com/linebender/parley): `CHANGELOG.md`, `parley_tests/linebreaking_cases`.

## Effects and 3D

### What it changes

- **Every shader effect is WGSL run by wgpu 30**: the 8 `@paper-design/shaders` programs (Apache-2.0: MeshGradient, GrainGradient, Warp, Swirl, GodRays, PerlinNoise, Dithering, SmokeRing), Opsis's own filters (crt-screen, vhs-filter, camera-lens, tv-power-off, halftone-print, mosaic), the transitions over two scenes (glitch-cut, particle-dissolve), light-leak, launch-anything's text reveal, lens-zoom's radial pass, and the terminal's texture and lens.
- **Effects read their input nodes as layers.** Skia draws each input into a pixel buffer, wgpu runs the program, the result comes back as an image layer. Canvas Draw Element (`captureElementImage`, `drawElementImage`) is no longer needed.
- **The adapter order:** a GPU; on Linux the SwiftShader in the pinned headless shell or the system's lavapipe; on Windows WARP; on macOS Metal. The renderer runs as a child process started with `VK_DRIVER_FILES` set, because pointing wgpu at a chosen Vulkan driver otherwise needs `unsafe` or `set_var`.
- **3D is a small wgpu renderer of our own**: metal/roughness PBR, ambient, hemisphere and up to three directional lights, one prefiltered environment map, ACES tone mapping, 4× MSAA, transparent planes, skinning, morph targets, animation sampling. glTF through the `gltf` crate; extrude, tube and shape geometry ported from three.js (MIT).
- **Integer hashes (PCG) in every WGSL program.** Four of today's shaders hash with `fract(sin(dot(…)) * 43758.5453)`, which differs between drivers.

### Fit with Opsis

- **In place:** `crates/prepare` makes typeface JSON with `skrifa`, normalises SVG with `usvg` and prepares GLB (Draco decoded with `draco-gltf`), so the 3D inputs are Rust already. The pinned Chrome Headless Shell ships SwiftShader on every platform (`libvk_swiftshader.so` 3,986,104 B on linux64).
- **Missing:** wgpu in the workspace, every WGSL port, the layer transfer, the 3D renderer, the child process.

### Work

1. wgpu in a renderer child process, with the adapter order and the "Graphics card / Software" render setting becoming the adapter choice.
2. The layer transfer between Skia's buffers and wgpu textures, with readback rows aligned to 256 bytes (`COPY_BYTES_PER_ROW_ALIGNMENT`) and double-buffered `map_async`.
3. WGSL ports of every program listed above, with PCG hashes. naga's GLSL front end takes only Vulkan GLSL 440 (partly), 450 and 460, so the GLSL ES 3.00 sources are ported by hand.
4. The 3D renderer and its geometry, against today's stills in `fixtures/prepare/stills`.
5. `screen-3d`'s clip drawn with an `SkM44` perspective from `three/project.ts`'s homography.

### Decisions

- **The environment map**: one precomputed prefiltered cube map shipped in `public/` in place of `RoomEnvironment` through `PMREMGenerator`, or one computed at load.

### Risks

- **SwiftShader through the headless shell's own Vulkan loader on macOS and Windows is untested.**
- **The 3D renderer's look against three.js's** (PMREM, ACES, MSAA) decides whether `three_stills.rs` passes with its slack of 24 over 99.5% of pixels.
- **Measured shader speed per 1080p frame** for one film-like filter: RTX 3060 1.7 ms, Intel Iris Xe 6.7 ms, lavapipe 49.8 ms, SwiftShader 143.9 ms. A 3D scene on the software drivers wasn't measured.

### Sources

- The shader timings come from a benchmark run on 5 November 2026 and not kept in the repository: one 1920×1080 WGSL filter shaped like the film's (32 blur taps in a spiral, colour fringing, 4-octave noise grain, a vignette), read back every frame, wgpu 30.0.1 on an i7-12700H, 60 frames per driver; the same kernel in plain Rust on rayon took 261.6 ms a frame. Rerun it on the target machines when this part starts. wgpu (https://github.com/gfx-rs/wgpu): `naga/src/front/glsl/mod.rs`, `wgpu-hal/src/{vulkan/instance.rs,dx12/adapter.rs}`, issue gfx-rs/wgpu#9293. WGSL §15.7: https://www.w3.org/TR/WGSL/.

## Footage and terminal

### What it changes

- **Footage frames reach the renderer as decoded frames.** The FFmpeg decoder per source and its grade graph stay (`crates/footage`); the JPEG 4:4:4 encode (42.3% of the app's CPU in the profiling pass), the HTTP serving and the page's decode go. The CPU path asks FFmpeg for RGBA with explicit `in_color_matrix` and `in_range`; the GPU path uploads Y, U and V as R8 textures and converts in WGSL.
- **Terminal pictures are drawn natively** from `opsis-vt`'s frames: cell backgrounds as rects, runs placed by column with the shared text stack and today's glyph narrowing, box-drawing and block sprites from `terminal/sprites.ts` as Rust paths so they stay sharp under camera zoom, texture and lens as WGSL passes.
- **`TerminalPainter` and the terminal page go**, and live terminal pictures for the desktop come from the same drawing.

### Fit with Opsis

- **In place:** `opsis-footage`'s decoders and grade chain (`decoder.rs`, `chain.rs`), Redact's areas in Rust, `opsis-vt` and the frames format, `crates/render/tests/sprites.rs` against Ghostty's atlases.
- **Missing:** the frame hand-over without JPEG, the native terminal drawing, the sprite port.

### Work

1. Footage frames as image layers, decoders per render chunk (in-order decoding per chunk, as `film.md` relies on today).
2. The terminal drawing and the sprite port, checked by `sprites.rs`.
3. Live terminal pictures for the desktop from the native drawing, replacing `window.opsisTerminal.paint` screenshots.

### Decisions

- None.

### Risks

- **Each parallel render chunk opens its own decoders**; footage FFmpeg peaked at 813–818 MiB in profiling, so the chunk count is set by measurement.

### Sources

- `crates/footage/src/{footage,decoder,chain}.rs`, `crates/capture/src/terminal_paint.rs`, `packages/video/src/terminal/{view,effects,sprites}.ts*`, the profiling pass's finding E1 (`git show 233701ad^:plans/profiling-findings.md`; the file was deleted in `233701ad`).

## Output

### What it changes

- **Finished frames go to FFmpeg raw**: `-f rawvideo -pix_fmt rgba -s WxH -framerate N -i -`, with `scale=out_color_matrix=bt709:out_range=tv` and `-colorspace`, `-color_primaries`, `-color_trc` and `-color_range` set. Today's film is JPEG q80 into `-c:v mjpeg` with no colour flags (`crates/media/src/codecs.rs` → `film_encode_args`).
- **Frames render in parallel chunks**: contiguous runs per worker, each with its own decoders, into a reorder buffer in front of one encoder.
- **Stills come from the same renderer** (PNG, JPEG, WebP through the `image` crate).
- **macOS and Windows no longer fall back to a screenshot per frame.**

### Fit with Opsis

- **In place:** the bundled FFmpeg sidecar and its required-format check (`crates/media/src/capabilities.rs`), the output executors (`crates/render/src/executors.rs`), chapters, sizes per shape (`variant_size`).
- **Missing:** `rawvideo` in the required formats, the raw pipe, the colour tags, the chunked scheduler.

### Work

1. `rawvideo` joins the required formats (`REQUIRED_FORMATS`); `film_encode_args` takes raw input with the tags above.
2. The chunk scheduler with the reorder buffer and progress per frame.
3. Stills and `render-elements` through the renderer directly.

### Decisions

- **Worker count** by measurement on the example projects.

### Risks

- **A 1080p RGBA frame is 8.3 MB**; the reorder buffer is bounded so a slow chunk doesn't hold memory.

### Sources

- `crates/render/src/frames.rs`, `crates/media/src/{codecs,capabilities}.rs`, FFmpeg 6.1.1 `libavcodec/mjpegdec.c:147`.

## Elements

### What it changes

- **Every element's draw function lives in its Rust file** in `crates/compile/src/elements/<category>/`, next to its `#[settings]` struct and `definition()`, and returns scene nodes through a builder API. One file per element instead of a Rust file and a TSX file.
- **All 140 implementations in `ELEMENTS` move**, by the capabilities they need ([Appendix: elements](#appendix-elements)): layout 71, text 84, vector 93, layer effects 96, SVG filters 12, perspective 7, pixel programs 24, 3D 6, media 20, canvas drawing 5.
- **The camera and takes move**: `core/recording-clip.tsx` and `camera/` (stage, cursor with the macOS cursor SVGs, window frame, keycap HUD, annotations, motion blur, demo clip), captions, the still and placeholder elements.
- **The film's JavaScript libraries are replaced:**
  - `roughjs` by `roughr` with rough.js's own Park–Miller generator (`Math.imul(48271, seed)`) in place of `rand`'s `StdRng`, so seeded strokes keep today's shapes;
  - `culori` by linebender `color` (CSS Color 4 parsing, OKLCH interpolation) plus our own chroma clamp, since `color` only clips to gamut;
  - `svg-path-commander` by `usvg` and `kurbo`;
  - `simplex-noise` by a port with the same output for the same seed;
  - `lottie-web` and `fflate` by Skottie and `zip`;
  - `three` and `@react-three/fiber` by the 3D renderer.

### Fit with Opsis

- **In place:** every element's settings, defaults, lengths, design size, fonts and themes are Rust already (`crates/compile/src/elements`, `lengths.rs`, `theme.rs`, `props.rs`). `colors.test.ts`'s allow-list and the theme presets carry over.
- **Missing:** every draw function, the builder API, the camera and terminal code in Rust.

### Work

1. The builder API over `opsis-scene` (`row`, `column`, `text`, `rect`, `path`, `image`, `effect`, placement, transforms).
2. Elements by category, simplest capabilities first; each checked against its still from `bun run elements:render` before the cut-over.
3. The camera, takes and captions.
4. Transitions, with `TransitionWrap`'s pairing of outgoing and incoming sides through slots, and `SceneSeries` for the scenes inside one element.

### Decisions

- None.

### Risks

- **Parity element by element.** The stills before the cut-over are the reference; a difference that is intended is reviewed and recorded.

### Sources

- `packages/video/src/elements/index.ts`, `packages/video/src/{demo-timeline.tsx,core,camera,lib,parts}`, rough-rs (https://github.com/orhanbalci/rough-rs) `roughr/src/core.rs:363`, the `roughjs` package's `bin/math.js`.

## Goldens and tests

### What it changes

- **Golden pictures are recorded on one software setup and compared exactly there**: x86 Linux, the fixed Skia CPU path, SwiftShader from the engine folder for effects and 3D.
- **GPU runs compare with a tolerance**: FLIP's mean (`nv-flip` 0.1.2, MIT/Apache/Zlib) or the slack and share `three_stills.rs` uses today.
- **Render tests stop needing a browser page.** The 11 tests `#[ignore]`d for the film or terminal page (`crates/render/tests/frames.rs` 5, `run.rs` 2, `terminal.rs` 2, `sprites.rs` 1, `three_stills.rs` 1) run in-process; those that need SwiftShader stay ignored where it isn't present.
- **Element stills** come from the renderer, and `elements:render` can join `bun run test` once shaders run on the software driver.
- **Scene goldens**: the serialised scene of chosen elements at chosen frames, compared through `opsis-golden`, with no rasterising.

### Fit with Opsis

- **In place:** `opsis-golden`, `fixtures/render`, `fixtures/prepare/stills`, the `OPSIS_WRITE_GOLDEN=1` flow.
- **Missing:** picture goldens per element, the exact-vs-tolerance switch, the property tests from **Scene**.

### Work

1. Picture goldens for every element at its middle frame, and scene goldens at three frames each.
2. CI on x86 Linux with SwiftShader from the engine download.

### Decisions

- None.

### Risks

- **`dssim` is AGPL-3.0** and stays out.

### Sources

- wgpu `examples/features/src/*/mod.rs` (FLIP thresholds), Chromium pixel testing with Gold, `crates/render/tests/three_stills.rs`.

## Cut-over

### What it changes

- **In one step, the browser stops drawing films.** Removed: `apps/film`, `apps/terminal`, `film-bundle`, `terminal-bundle`, `packages/runtime`, the film parts of `packages/video`, `FrameDriver` and `film_switches` (`crates/capture/src/film.rs`), `TerminalPainter`, the `CanvasDrawElement` feature switch and the begin-frame driver.
- **The recording browser stays** for capture, page stills, smooth takes, sign-in and the streamed views; its download is unchanged, and its SwiftShader doubles as the renderer's software driver.
- **The docs move with it**: `film.md` becomes the renderer's doc, `elements.md` and `terminal.md` lose their page parts, `scene.md` is new, AGENTS.md's film rules (`packages/video` purity, oxlint animation bans, `holdRender`) become their Rust equivalents.

### Fit with Opsis

- **In place:** everything above, by then.
- **Missing:** the deletions and the doc rewrite.

### Work

1. Delete the listed apps, packages, bundles and Rust paths; remove their scripts (`film:bundle`, `terminal:bundle`) and their table rows in AGENTS.md.
2. Rewrite the docs named above in the same change.

### Decisions

- None.

### Risks

- **There is no fallback after the cut-over**; the stills and goldens recorded before it are the safety net.

### Sources

- `AGENTS.md` §2, §6, §7; `agent-docs/{film,elements,terminal,footage,engine}.md`.

## Appendix: drawing features

What the film's 140 element and transition implementations use today, from a scan of each one's import closure in `packages/video/src` on 5 November 2026 (type-only imports, `@opsis/runtime` and generated files left out), with every perspective, canvas, shader and 3D hit checked by hand. Counts are elements, not occurrences.

- **Layout and text:** CSS flex in 71 and grid in 10; text wrapping 52, letter spacing 53, font variations and tabular numbers 18, `background-clip: text` 5, text shadow 3, text stroke 1 (the cursor label). Text measured in the page: `measureText` in word-push, kinetic-center-build, slot-machine-roll, punch-lines, launch-anything's shader text reveal and the terminal view; DOM measuring in announce-title, `core/hand-drawn.tsx`, `lib/layout/screen-fit.tsx`.
- **CSS filters:** `blur()` on subtrees in 87 (75 through the `blur` entrance in `core/anim.ts`), brightness 13, saturate 10, drop-shadow 8, contrast 5, sepia 2, hue-rotate 2. `backdrop-filter` in 3 (glass-code-block, frosted-glass-wipe, lower-third).
- **Blend modes:** screen 10, overlay 2, lighten 1 (13 elements).
- **Clips and masks:** `clip-path` inset 10, polygon 4, circle 1, path 1, `url(#…)` 2 (19 elements); gradient masks with size, position and repeat in 8.
- **Paint:** linear gradients 27, radial 26, conic 4, repeating 3; box shadows 31; rounded corners 52; colours mixed in OKLCH in 26.
- **SVG:** paths and shapes in 46, stroke-dash line drawing 15, masks, patterns and clip paths 13; filter primitives in 12: `feColorMatrix` 7, `feTurbulence` 4, `feGaussianBlur` 4, `feOffset` 3, `feDisplacementMap` 3, `feBlend` 2, `feMerge` 1, `feImage` 1.
- **Perspective:** real CSS 3D transforms in laptop-frame, phone-frame, announce-title, hero-launch, launch-anything, the flip transition, and screen-3d's page layer (`matrix3d` from `three/project.ts`).
- **Pixel programs:** 24 elements; see **Effects and 3D** for the list.
- **3D:** six elements on `Stage3D`; only `MeshStandardMaterial` and `MeshBasicMaterial`, no shadow maps, no post-processing.
- **Outside the elements:** `core/recording-clip.tsx` and `camera/` (footage frames, rough.js annotations, SVG filters, `foreignObject` for the terminal screen under a lens filter, conic gradients, text stroke); `terminal/view.tsx` (cell runs as positioned boxes, `measureText`, noise through `putImageData`, `feDisplacementMap` lens, `feTurbulence` grain, overlay blending).

## Appendix: elements

Each implementation in `ELEMENTS` with the capabilities its files use. "Plain boxes" means none of the scanned patterns matched; `page-turn` uses the CSS `rotate` property, which the scan didn't tag.

| Category   | Count | Elements                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| ---------- | ----- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| background | 17    | `shader-mesh-gradient` (pixel programs); `shader-grain-gradient` (pixel programs); `shader-warp` (pixel programs); `shader-swirl` (pixel programs); `shader-god-rays` (pixel programs); `shader-perlin-noise` (pixel programs); `shader-dithering` (pixel programs); `aurora-bg` (vector, layer effects); `particle-field` (vector, layer effects); `video-loop` (media); `solid` (plain boxes); `gradient` (vector, layer effects, SVG filters); `noise` (vector, layer effects, SVG filters); `starburst` (vector); `paper` (vector, layer effects, SVG filters); `backdrop` (vector, media); `shapes-3d` (3D)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| core       | 2     | `placeholder` (layout, text, vector); `still` (layout, text, vector, layer effects, media)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| data       | 10    | `animated-bar-chart` (layout, text, vector, layer effects); `animated-line-chart` (layout, text, vector, layer effects); `rolling-number` (layout, text, layer effects); `stat-card` (layout, text, vector, layer effects); `metric-ticker` (layout, text, vector, layer effects); `funnel-chart` (layout, text, vector, layer effects); `donut-chart` (layout, text, vector, layer effects); `comparison-bars` (layout, text, vector, layer effects); `bar-chart-race` (layout, text, vector, layer effects); `gauge-dial` (layout, text, vector, layer effects)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| device     | 6     | `laptop-frame` (layout, text, vector, layer effects, perspective, media); `phone-frame` (layout, text, vector, layer effects, perspective, media); `captured-video` (text, vector, layer effects, media); `device-mockup-zoom` (layout, text, vector, layer effects, media); `multi-device-lineup` (layout, text, vector, layer effects); `screen-3d` (vector, layer effects, perspective, 3D)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| effect     | 8     | `crt-screen` (text, vector, layer effects, pixel programs); `vhs-filter` (text, vector, layer effects, pixel programs); `halftone-print` (text, pixel programs); `camera-lens` (text, vector, layer effects, pixel programs); `polaroid` (layout, text, vector); `tv-power-off` (text, vector, layer effects, pixel programs); `speed-lines` (vector); `mosaic` (text, pixel programs)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| media      | 2     | `image` (layer effects, media); `video` (media)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| overlay    | 14    | `marker-highlight` (text, layer effects); `inline-highlight` (layout, text, layer effects); `ink-underline` (vector, SVG filters); `scribble-circle` (vector, SVG filters); `ink-arrow` (vector, SVG filters); `keystroke` (layout, text, vector); `confetti` (vector, layer effects); `radial-burst` (vector); `notification-stack` (layout, text, vector, layer effects); `lower-third` (layout, text, vector, layer effects); `callout` (text, vector, layer effects); `captions` (layout, text, vector); `lottie` (media); `light-leak` (layer effects, pixel programs)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| scene      | 19    | `spring-settle` (layer effects); `signup-flow` (layout, text, vector, layer effects); `ai-prompt-flow` (layout, text, vector); `command-menu` (layout, text, vector, layer effects); `release-teaser` (layout, text, vector, layer effects, media, canvas drawing); `launch-anything` (layout, text, vector, layer effects, perspective, pixel programs, media, canvas drawing); `logo-assemble` (layout, text, vector, layer effects, media); `logo-drift` (layout, text, vector, layer effects); `hero-launch` (text, vector, layer effects, perspective, media); `data-story` (layout, text, vector, layer effects, pixel programs, media); `logo-reveal` (layout, text, vector, layer effects, SVG filters); `pricing-card` (layout, text, vector, layer effects); `comparison-table` (layout, text, vector, layer effects); `changelog-entry` (layout, text, vector, layer effects); `logo-wall` (layout, text, vector, layer effects, media); `end-card` (layout, text, vector, layer effects, media); `logo-3d` (vector, 3D, canvas drawing); `card-3d` (vector, 3D, canvas drawing); `model-3d` (vector, 3D, canvas drawing)                                                                                                                             |
| text       | 24    | `soft-blur-in` (layout, text, layer effects); `typewriter` (layout, text, vector); `shimmer-sweep` (layout, text, vector, layer effects); `matrix-decode` (layout, text); `slot-machine-roll` (layout, text); `staggered-fade-up` (layout, text, layer effects); `kinetic-center-build` (layout, text, layer effects); `mask-reveal-up` (layout, text, layer effects); `tracking-in` (layout, text, layer effects); `word-push` (layout, text, layer effects); `rgb-glitch-text` (layout, text, layer effects); `gradient-scale-cut-text` (text, vector, layer effects); `strikethrough-replace` (layout, text, vector, layer effects); `line-by-line-slide` (layout, text, layer effects); `announce-title` (layout, text, vector, layer effects, SVG filters, perspective); `punch-lines` (layout, text, vector, SVG filters); `text-highlight` (layout, text, vector, layer effects, media); `status-cycle` (layout, text, vector, layer effects); `handwriting-text` (layout, text, vector, layer effects); `neon-flicker-text` (layout, text, layer effects); `text-mask-video` (layout, text, vector, layer effects, media); `title` (layout, text, vector, layer effects); `kinetic-headline` (layout, text, vector, layer effects); `text-3d` (text, 3D) |
| transition | 25    | `whip-pan` (layer effects); `push-through` (layer effects); `zoom-blur` (layer effects); `focus-pull` (layer effects); `lens-zoom` (vector, layer effects, SVG filters); `ripple-zoom` (layer effects, pixel programs); `wave-wipe` (layer effects, pixel programs); `smoke-dissolve` (layer effects, pixel programs); `glitch-cut` (text, vector, layer effects, pixel programs); `page-turn` (plain boxes); `grain-dissolve` (layer effects, pixel programs); `dither-dissolve` (layer effects, pixel programs); `swirl-dissolve` (layer effects, pixel programs); `particle-dissolve` (text, layer effects, pixel programs); `grid-pixelate-wipe` (vector, layer effects); `frosted-glass-wipe` (vector, layer effects); `chromatic-aberration-wipe` (vector, layer effects, SVG filters); `transition-blinds` (vector, layer effects); `transition-morph-shape` (vector, layer effects, SVG filters); `fade` (plain boxes); `slide` (plain boxes); `wipe` (vector); `flip` (perspective); `clock-wipe` (vector); `iris` (vector)                                                                                                                                                                                                                             |
| ui         | 13    | `terminal-simulator` (layout, text, vector, layer effects); `glass-code-block` (layout, text, vector, layer effects); `code-morph` (layout, text, vector, layer effects); `logo-enter` (layout, vector, media); `check-list` (layout, text, vector); `search-reveal` (layout, text, vector, layer effects); `agent-run` (layout, text, vector); `code-reveal` (layout, text, vector, layer effects); `code-diff-wipe` (layout, text, vector, layer effects); `kanban-move` (layout, text, vector, layer effects); `file-tree-reveal` (layout, text, vector, layer effects); `drag-drop-flow` (layout, text, vector, layer effects); `commit-graph` (layout, text, vector, layer effects)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
