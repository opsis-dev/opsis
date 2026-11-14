# Native renderer

How Opsis stops drawing films in the recording browser: every element, the camera, the terminal and the transitions become Rust that returns scene nodes written in the CSS and SVG subset the film uses today, Taffy and Parley lay them out, Skia draws them on the CPU, Filament runs the shader effects and the 3D, and finished frames go straight into the bundled FFmpeg. Node packs draw through the same scene contract ([node-packs.md](node-packs.md)).

Status: built on Linux x64 and cut over; the film page, the terminal page, the frame driver and the oracle are gone, and the design lives in `agent-docs/scene.md` and `agent-docs/film.md`. Still missing: Skia and Filament on the other platforms (**Wrappers**). Facts were checked on 5 and 12 November 2026, in the libraries' own repositories and the Opsis code; everything this plan relies on is in it or linked from it.

## How to read this plan

- **Oracle** comes first: the reference pictures every later part is checked against.
- **Wrappers** come next: the two crates that hold Skia and Filament behind safe Rust.
- **Scene**, **Layout** and **Paint** follow: the types every element returns, how they are laid out and how they are drawn. No element is ported before their property cases pass.
- **Effects and 3D**, **Footage and terminal** and **Output** come next.
- **Elements** is the port of every element, the camera and the terminal view onto the scene contract.
- **Goldens and tests**, then **Cut-over**, close it. The cut-over is the merge of the branch.
- Each part has the same sections: **What it changes**, **Fit with Opsis** (In place / Missing), **Work**, **Decisions**, **Risks**, **Sources**.

## Conventions

These restate rules from `AGENTS.md` and `agent-docs/`; when they disagree with this plan, they win. When a part ships, its rules move into the docs that own the domain (`film.md`, `elements.md`, `terminal.md`, `footage.md`, `engine.md`, `runs.md`), and a new `agent-docs/scene.md` holds the scene contract, the clock and the caches.

- **Every frame is a pure function of the frame number.** An element is `fn(&Settings, &Clock, &Stage) -> Node`; no element reads a wall clock, a random source, a file or the network. Randomness is Opsis's `random(seed)`, ported exactly.
- **Rust owns the open project and every file, as today.** The renderer is Rust in the app's process tree; nothing draws a film in a page.
- **The work happens on a branch in its own worktree.** `main` is not touched until the merge, and its release build stays as the oracle.
- **An element never works around a missing feature.** A construct with no row in the transform table stops the port of that element; the construct is added to the renderer and the table, and the port goes on.
- **No comments**, in Rust, C++, GLSL and every new file.
- **No number is converted with `as`**, and floats are compared exactly only through `exact_eq`, in renderer code as everywhere else.
- **Nothing uses the internet unless the user asks.** Fonts, images, models and footage reach the renderer from the project, the data folder and `public/`, as `app:` and project refs do today.
- **Checks when a part is done:** `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, plus the part's cases against the oracle.

## Decided

- **The scene contract is the film's own CSS and SVG subset.** A node carries the CSS properties and SVG tags the elements use today, under the same names, with CSS meaning and the defaults of `packages/video/src/video.css`, so porting an element is a lookup in the transform table. It is Opsis's own code on Taffy and Parley.
- **Skia draws every 2D frame, on the CPU.** It is the only library that covers every 2D feature the film uses on the CPU: all blend modes, layers with backdrop blur, every CSS filter function, turbulence, displacement and colour-matrix filters, `SkM44` perspective, OKLCH gradient interpolation (`SkGradient.h`), dithering, F16 surfaces.
- **Opsis calls Skia through its own wrapper crate, `opsis-skia`**: C++ over the Skia calls the film needs, behind a safe Rust API, with Skia built from a pinned commit. No third-party sys crate.
- **Filament runs every shader effect and the 3D**, through a second wrapper crate, `opsis-filament`. Shaders stay GLSL ES 3.0, as materials. It draws on a GPU when there is one and otherwise on a software Vulkan driver: the SwiftShader inside the pinned Chrome Headless Shell, or the system's lavapipe on Linux. macOS draws on Metal.
- **Filament comes from Google's release archives**, pinned by sha256 and size for each platform, with the `matc` material compiler from the same archive.
- **`unsafe` is allowed in `opsis-skia` and `opsis-filament`**, beside `opsis-vt` and `opsis-qt`.
- **Taffy lays out boxes, Parley lays out text**, from the same font bytes Skia draws.
- **The scene model is pure evaluation with content hashes, memo caches and a raster layer cache**, not a signal graph. Held frames get equal hashes by quantising before hashing: positions to 1/64 px, opacity in 8-bit steps; `spring` keeps its maths.
- **The browser film page is replaced in one go: the merge of the branch.** Every element, the camera, the terminal and every transition pass on the native renderer first. There is no period with both renderers on `main`.
- **`main`'s release build is the oracle.** It renders the reference pictures, and the commit it was built from is recorded with them.
- **Golden pictures are exact on one software setup**: the pinned SwiftShader and a fixed Skia CPU path. GPU runs compare with a tolerance.
- **No live preview in the editor in this plan.** A film is still watched as an export; a preview is a later decision.
- **Lottie plays through Skottie without expressions.** A file that contains any expression draws its keyframed values, and its card warns and points to Bodymovin's expression baking. Android, iOS and Windows Lottie don't run expressions either (Airbnb's `supported-features.md`), and the Lottie spec leaves them out (IANA `video/lottie+json`).
- **Element authoring is Rust for built-in elements** and WebAssembly components for packs, both returning the same scene nodes. Packs ship GLSL shaders, compiled when the pack loads ([node-packs.md](node-packs.md)).

## Summary

| Part                 | What it changes                                                                      | Needs            |
| -------------------- | ------------------------------------------------------------------------------------ | ---------------- |
| Oracle               | reference frames of every element and example film, and one case per CSS property    | —                |
| Wrappers             | `opsis-skia` and `opsis-filament`, the Filament pin, the readback measurement        | —                |
| Scene                | scene types in the CSS and SVG subset, clock and time maps, hashes, caches, resolve  | —                |
| Layout               | `opsis-layout`: block, inline, flex and grid on Taffy and Parley, stacking order     | Scene            |
| Paint                | `opsis-paint`: positioned boxes, text and inline SVG into Skia calls                 | Wrappers, Layout |
| Effects and 3D       | `opsis-effects` and `opsis-stage3d`: shader passes and 3D scenes on Filament         | Wrappers, Paint  |
| Footage and terminal | decoded frames as image layers, the terminal drawn natively                          | Paint            |
| Output               | raw frames into FFmpeg with colour tags, parallel chunks                             | Paint            |
| Elements             | the transform table, all 140 element and transition implementations, the camera      | every part above |
| Goldens and tests    | exact goldens on the software setup, tolerance on GPUs, render tests un-ignored      | Elements         |
| Cut-over             | the merge: delete the film page, the terminal page, the frame driver and the painter | everything       |

## Oracle

### What it changes

- **`main`'s release build renders the reference pictures once**, into `~/.cache/opsis-dev/oracle/<commit>`: every element at five frames (first, the two quarter points, middle, last), and every project in `examples/` as whole films.
- **One case per CSS property and SVG tag in [Appendix: constructs](#appendix-constructs)**: a small page drawn by the pinned browser with `video.css` and the bundled fonts, beside the same case written as scene nodes.
- **Every later part compares against these pictures**, never against the browser running in the branch.

### Fit with Opsis

- **In place:** the branch's first commit, built in release, is the oracle; `main` itself stays untouched so the branch fast-forwards onto it at the cut-over.
  - `bun run oracle [elements|cases|boxes|films|all] [part of a name]` (`scripts/oracle.ts`) builds from a clean checkout of a commit (`--commit=`, `HEAD` by default) in `~/.cache/opsis-dev/oracle/src`, so changes in the working tree never reach it: it builds the film and oracle pages there, stages FFmpeg and runs `crates/oracle/src/bin/oracle.rs` in release with its own target folder, `~/.cache/opsis-dev/oracle/target`. The pictures go to `~/.cache/opsis-dev/oracle/<commit>`, or with `--into=<commit>` into an earlier oracle, `oracle.json` recording which commit drew each part.
  - Boxes: the oracle page's `window.oracleBoxes()` (`apps/oracle/src/boxes.ts`) lists every element's border box and every text node's line boxes by child-index path; `BoxReader` (`crates/oracle/src/boxes.rs`) loads each case at 640×360 with the film's launch switches and writes `cases/<group>/<name>.boxes.json`. The scene tree mirrors the DOM, so the layout's results are checked path by path against Chrome's without drawing.
  - Everything draws on Software (`ORACLE_GL`, `crates/oracle/src/pictures.rs`) through the frame driver at full size as PNG, so the pictures don't depend on the graphics card.
  - Elements: each element as `render-elements` builds it (`crates/oracle/src/elements.rs` → `workflow_for`, `element_spec`), at `ORACLE_POSITIONS`, into `elements/<key>/<frame>.png` beside the `spec.json` it drew; `elements.json` lists them.
  - Cases: one TSX file per construct in `apps/oracle/src/cases/<group>/<name>.tsx`, drawn by the oracle page (`apps/oracle`, `oracle-bundle`) on the film's own runtime, `video.css` and bundled fonts (`CASE_FONTS`, `crates/oracle/src/cases.rs`) at 640×360, into `cases/<group>/<name>.png`; `cases.json` lists them with the fonts they loaded. The scene-node version of each case lives once in `opsis-cases` (`crates/cases`), which the layout and paint tests share; the SVG cases are their React markup rendered once and parsed into SVG nodes (`crates/cases/svg`).
  - Films: each example copied into `projects/<name>`, every Render node set to MKV lossless on Software (`lossless_renders`, `crates/oracle/src/films.rs`), run once with the browser, terminal, media and output executors, and its exports copied into `films/<name>`. The project keeps its takes in `.opsis/cache`, so the native renderer draws the same takes without recording again.
  - `oracle.json` records the commit, the browser and the drawing choice.
- **Missing:** nothing.

### Work

1. Done: `render-elements` takes a comma-separated list of positions (`bun run elements:render -- <key> 0,0.5,1`), and the oracle writes each element's frames with the spec it drew.
2. Done: the example films as lossless MKV with their recorded takes.
3. Done: the property cases, grouped as the appendix groups them.

### Decisions

- None.

### Risks

- **A picture the oracle draws wrong is copied into the port.** The paint traps in [film.md](../agent-docs/film.md) are the known ones.

### Sources

- `crates/oracle/src/`, `apps/oracle/src/`, `scripts/oracle.ts`, `crates/project/src/templates.rs` → `examples()`.

## Wrappers

### What it changes

- **`opsis-skia` wraps the Skia calls the film needs**, in C++ behind the `cxx` bridge, as `opsis-qt` wraps Qt ([ui.md](../agent-docs/ui.md)): raster surfaces (N32 and F16), the canvas with layers and backdrops, paths, paints, gradients, image and colour filters, blend modes, `SkM44`, the font manager, glyph runs, images and Skottie. Skia is built from a pinned commit with GN.
- **`opsis-filament` wraps Filament**: the engine on a chosen backend, a headless swap chain (`Engine::createSwapChain(width, height, flags)`), views with post-processing off, render targets, textures, materials, lights, the indirect light, glTF loading (`gltfio`) and readback (`Renderer::readPixels`).
- **`opsis-filament` exposes plain C functions on every platform.** Filament's Linux build is clang 17 with libc++, while the rest of the app's C++ on Linux is built with GCC (`gcc_64` Qt, `scripts/qt-install.ts`), so no C++ type crosses that boundary.
- **Filament is pinned like the browser and FFmpeg**: a script writes each platform's archive, sha256 and size into a generated file, and the archives are kept in `~/.cache/opsis-dev/filament`.
- **Rust reaches neither library except through these two crates.**

### Fit with Opsis

- **In place:** the `cxx` bridge and its build from Cargo (`crates/qt/build.rs`), the pin scripts for the browser and FFmpeg (`scripts/engine-downloads.ts`, `scripts/ffmpeg-pin.ts`).
  - Skia `chrome/m157` (`fef60e46`, the milestone of the oracle's Chrome 157) is built by `bun run skia:build` (`scripts/skia-build.ts`), directly or with `--docker` in an `ubuntu:26.04` container, into static libraries in `~/.cache/opsis-dev/skia/<commit>/out/<platform>`: CPU only, its own FreeType, PNG, JPEG, WebP and zlib, Skottie, the primitive shaper, no fontconfig, no PartitionAlloc and no Perfetto (PartitionAlloc as `malloc` would replace the allocator of the whole app). The build writes `consumer.json`, the defines, include folders and libraries `crates/skia/build.rs` compiles and links with.
  - `opsis-skia` draws and reads back its first test picture (`crates/skia/tests/surface.rs`).
  - The Filament pin: `bun scripts/filament-pin.ts [--fetch]` writes `crates/filament/downloads.generated.json` (v1.80.1, each desktop archive's URL, size and sha256 from the release) and unpacks this platform's archive into `~/.cache/opsis-dev/filament/<version>/<platform>`.
  - The libc++ link, measured on 12 November 2026 in an `ubuntu:26.04` container: C functions compiled with clang and libc++ over Filament, partly linked (`ld -r`) with Filament's archives (`libfilament`, `libbackend`, `libbluevk`, `libbluegl`, `libwebgpu_dawn`, `libfilabridge`, `libfilaflat`, `libutils`, `libgeometry`, `libibl-lite`, `libshaders`, `libsmol-v`, `libabseil`, `libperfetto`, `libzstd`) and the static `libc++.a` and `libc++abi.a`, then `objcopy --keep-global-symbol='opsis_*'`, leave two global symbols; a program built with GCC and libstdc++ links that object (with `-lX11`, which Dawn's surface code needs) and still throws and catches its own exceptions.
  - `opsis-filament`: `bun run filament:build [--docker]` seals `crates/filament/cpp` (an engine on a chosen backend, materials, and one full-screen pass from an RGBA picture to an RGBA picture) the same way into `libopsis_filament.a`; `build.rs` compiles `crates/filament/materials/*.mat` with the pinned `matc` and links the archive. `crates/filament/tests/pass.rs` runs a pass on lavapipe and on the RTX 3060; `readPixels` returns rows bottom-up, so `Filament::pass` turns them upright.
- **Missing:** Skia and Filament on the other platforms, clang and libc++ on Linux build machines.

### Work

1. `opsis-skia`: Skia built on Linux x64 and arm64, Windows and macOS, and one test picture drawn through the wrapper. Linux x64 is done.
2. Done on Linux x64: `opsis-filament`, the pin, the archive unpacked for the build, and one test picture drawn without a window.
3. Done: the readback measurement, 1920×1080, the film-like filter of **Effects and 3D** (32 spiral taps with colour fringing, 4-octave PCG grain, a vignette) as an unlit material compiled by `matc`, drawn over a full-screen triangle and read back with `readPixels` and `flushAndWait` every frame, 60 frames, Filament 1.80.1 on Vulkan without a window: RTX 3060 6.21 ms (clear and readback alone 4.19 ms), Intel Mesa 10.32 ms (6.97), lavapipe 74.64 ms (5.50). A frame-by-frame render affords it, so shader effects stay on Filament. The same program in a container limited to 8 CPUs took 380 ms on lavapipe, which scales with the cores it gets.
4. Drawing without a window on macOS (Metal) and on Windows, and the software driver on Windows.

### Decisions

- **Whether one Filament engine serves every render worker through a queue, or each worker holds its own.** `Renderer::renderStandaloneView` "must be called from the Engine's main thread" (`Renderer.h`).

### Risks

- **Filament calls `readPixels` "intended for debugging and testing. It will impact performance significantly"** (`Renderer.h`), and every frame with a shader effect or a 3D layer needs one. Work 3 measured it as affordable; reading back while the next frame draws would hide part of the 4–7 ms it costs.
- **The libc++ object stays sealed**: every C++ name in it is local after `objcopy`, so no Filament or libc++ type may cross into the app's C++; only the `opsis_*` C functions do.
  - `objcopy` also renames `DW.ref.__gxx_personality_v0` (`PERSONALITY_REF`, `scripts/filament-build.ts`): it sits in a COMDAT group, and when Skia's libstdc++ objects bring a group of the same name the linker keeps theirs and the sealed object's exception tables point into a discarded section.
- **Skia's C++ API moves between milestones** (at m159 the gradient factories live only in `include/effects/SkGradient.h`); the pinned tree is m157.
- **Linking the system FreeType** can crash when the version differs from the one Skia was built with (rust-skia #1345); the font manager in **Paint** avoids the system's.

### Sources

- Filament v1.80.0 (2026-11-02, Apache-2.0, https://github.com/google/filament): `README.md` (backends: OpenGL, Metal, Vulkan, WebGPU), `BUILDING.md` (clang 17, libc++, software rendering through `VK_ICD_FILENAMES`), `CMakeLists.txt` (`FILAMENT_ENABLE_EXPERIMENTAL_GCC_SUPPORT`), `filament/include/filament/{Engine.h,Renderer.h,SwapChain.h}`, `filament/backend/src/vulkan/platform/VulkanPlatformSwapChainImpl.cpp` (`VulkanPlatformHeadlessSwapChain`).
- The v1.80.0 archives: `filament-v1.80.0-linux.tgz` 76,102,096 B, `-arm-linux.tgz` 74,355,070 B, `-mac.tgz` 45,445,964 B, `-windows.tgz` 867,739,530 B. The Linux one holds `lib/x86_64/libfilament.a` (5,189,728 B), `libbackend.a` (5,053,894 B), `libfilamat.a` (35,180,872 B), `libgltfio_core.a`, `libdracodec.a`, `libbluevk.a`, and `bin/matc` (17,137,880 B) and `bin/cmgen`.
- Skia (https://github.com/google/skia): `include/core/SkMilestone.h`, `gn/skia.gni`. rust-skia issue #1345.

## Scene

### What it changes

- **A new crate, `opsis-scene`, holds what every element returns and how a frame is evaluated.** It draws nothing, so `opsis-compile` depends on it without Skia or Filament.
- **A node is a box, a text run, an inline SVG tree or a leaf**, with a style struct whose fields are the CSS properties in [Appendix: constructs](#appendix-constructs): size and position, flex and grid, padding, margin and border, background layers, radius, shadows, opacity, `transform` with `translate`, `scale` and `rotate`, `filter`, `backdrop-filter`, `mix-blend-mode`, `clip-path`, masks, overflow, `z-index` and the text properties.
- **Leaves are what CSS has no word for**: an image or footage frame, a Lottie file at a frame, a 3D scene, a shader effect over other nodes, a 2D drawing callback, and slots for children and transitions.
- **Inline SVG keeps its tags and attributes** (`path`, `circle`, `rect`, `line`, `ellipse`, `g`, `defs`, `clipPath`, `mask`, `linearGradient`, `radialGradient`, `stop`, `text`, `filter` and the eight filter primitives in use).
- **The node types are Rust** (`Node`, `Element`, `Kind`, `crates/scene/src/node.rs`): an element is a `div`, `span`, image, inline SVG tree or leaf with a sparse `Style` (one `Prop` per CSS property set, `crates/scene/src/style.rs`) and its children, as the film's React tree is. The flat WIT form with child indices comes with [node-packs.md](node-packs.md).
- **Time is a value passed down, never a global.** `Clock` carries the frame (fractional, because remaps are), the rate and the length. `Sequence`, `Freeze`, `Loop`, group `remap`, `startFrom`, `loopEvery` and each motion-blur sample are pure maps from one clock to another. Elements registered with `.authored_clock` get a 30 fps clock, as `useAuthoredFrame` gives them today.
- **Every node gets a content hash when it is built.** A group's own transform and opacity stay out of its content hash, so moving or fading a group keeps what's inside cached.
- **Memo caches keyed by value**: text shaping, layout, path flattening, decoded images, parsed SVG, spring lengths. `measureSpring` loops frame by frame to the rest frame on every `spring({durationInFrames})` call today (`packages/runtime/src/spring.ts:76-111`).
- **A raster layer cache keyed by subtree hash and raster-scale bucket.** Because frame n+1 is known, a camera move rasters once at its largest scale. Footage and backdrop filters over moving content are never cached. A frame whose root hash equals the last one's reuses the last picture.
- **A resolve pass before the first frame** loads every font, image, SVG, model, Lottie file and footage decoder an element asks for, replacing `holdRender`, `useHeldLoad`, `untilPainted` and `document.fonts.ready`.

### Fit with Opsis

- **In place:** `spring` is closed form (`springProgress`), `random(seed)` is a hash, `interpolate` is stateless (`packages/runtime/src`), so frame n needs no earlier frame. Compile already produces a `TimelineSpec` that one `DemoTimeline` draws (`packages/video/src/demo-timeline.tsx`), and the flattener, cuts, remaps and envelopes are Rust (`crates/compile/src`). The film styles inline: its one stylesheet, `video.css`, is a reset, so there is no selector or cascade to carry over.
- **In place on the branch:** `opsis-scene` with the style struct, the clock maps and the timeline walker (`draw_film`) for tracks, transitions, envelopes, groups, wrappers and sequencers.
- **In place on the branch too:** the caches. A frame whose scene hashes the same as the last reuses its picture (`Film::scaled_frame`); decoded images, SVG trees, typefaces, Lottie files and composited layers stay in `opsis_paint::Cache` for as long as each was used in the last two frames, a layer keyed by its element, its laid-out subtree and its raster transform; shaped paragraphs stay in `Fonts` the same way, keyed by their text, styles and boxes; picture sizes are kept per film. The resolve pass: `NativeFiles::resolve` checks every file the spec names before the first frame, preparing what is prepared, and fails the render naming what is missing; a footage decoder that sends nothing for 30 s fails the render (`FootageError::Stalled`). The property tests run in `crates/scene/tests/{clock,timeline}.rs` and `crates/render/tests/goldens.rs` → `elements_hold_still_after_their_animation`, which skips the elements that keep moving on purpose (`KEEPS_MOVING`, with reasons) and compares the rest within a hundred-thousandth, so springs settling in their last digits pass.
- **In place on the branch too:** the planned raster scale: a zooming camera marks its layer `will-change: scale` on frames whose scale differs from the next one's, and paint rasters it at the scale stepped up to the next eighth of an octave over a region snapped to a 256 px grid, keyed by its content without its own placement, so a zoom rasters once per step; `crates/paint/tests/layers.rs` checks a warm cache draws what a cold one does.
- **Missing:** nothing.

### Work

1. `opsis-scene` with the node types and the style struct, `Clock` and the clock maps, `Stage` (size, design size, overlay scale, shape insets), and the quantise-and-hash step.
2. The timeline walker: tracks, items with `from` and length, envelopes, transitions with their outgoing and incoming slots, groups with `remap`, `loopEvery`, `startFrom`, crop, design box, wrapper and sequencer.
3. Motion blur as N clocks for one subtree, accumulated with `sampleOpacity` weights (`camera/lib/shutter.ts` ported).
4. Memo caches with age-based eviction, and the layer cache with the planned raster scale.
5. The resolve pass, with the 30-second limit kept only for a footage decoder that stalls.
6. Property tests: a frame evaluated twice is equal; a frame evaluated after another equals a fresh evaluation; frame f equals frame f+1 after `animation_length`; the clock maps compose.

### Decisions

- Decided: **the scene types are Rust only** (`crates/scene/src`), since node packs aren't built in this plan; the WIT and its agreement test come with [node-packs.md](node-packs.md).
- Decided: **`comemo`** for the memo layer. On the spring rest measurement it gave the same results at about 20× the speed (100,000 calls: 113 ms uncached, 5.7 ms through `#[comemo::memoize]`) and served eight threads at once. Floats reach a memoized function through the `Num`, `Px` and `Alpha` newtypes (`crates/scene/src/values.rs`), which hash exactly, at 1/64 px and at 8-bit steps.

### Risks

- **A cache key that misses an input draws a stale picture.** Every input an element reads has to be in its hash; the property tests above and the goldens catch most of it.
- **Quantising changes pictures by up to 1/64 px and one 8-bit step**, within what the goldens are recorded at.

### Sources

- Skottie's seek and static properties: https://github.com/google/skia `modules/skottie/src/{Skottie.cpp,animator/Animator.cpp}`. Flutter's three-frame raster cache warm-up: https://github.com/flutter/flutter `engine/src/flutter/flow/raster_cache.h` (`access_threshold = 3`). `comemo`: https://github.com/typst/comemo `src/constraint.rs`, `src/accelerate.rs`.

## Layout

### What it changes

- **A new crate, `opsis-layout`, turns scene nodes into positioned boxes** with CSS meaning for the properties the film uses.
- **Taffy lays out every box**: block, flex and grid as CSS defines them. Its fixtures take their expected numbers from Chrome for Testing (`scripts/gentest`).
- **Parley shapes and breaks text** (harfrust, ICU4X segmentation, `skrifa`, fontique fallback): rich runs, variable fonts, OpenType features, letter and word spacing, bidi, CSS whitespace. Its line breaks are tested against Chromium's (`parley_tests/linebreaking_cases`).
- **Inline content is a Taffy leaf whose measure function calls Parley**: text with `span`s of their own style, and inline boxes for the characters and words elements animate one by one.
- **Text properties inherit** (`color`, the font properties, `letter-spacing`, `line-height`, `text-align`, `white-space`), and `em`, `%`, `calc()` and the container query units resolve against the same boxes CSS names.
- **Paint order follows CSS stacking**: `z-index`, positioned boxes, and the properties that make a stacking context (`opacity`, `transform`, `filter`, `mix-blend-mode`, `isolation`).
- **Elements that measure before laying out** (word-push, kinetic-center-build, slot-machine-roll, punch-lines, launch-anything's shader text reveal, the terminal view, which call `measureText` today, and the seven files that read boxes from the page) call one measurer.

### Fit with Opsis

- **In place:** `skrifa` and the font library (`opsis-fonts`), `family_for` (`crates/compile/src/fonts/mod.rs`) and `face_for` (`crates/compile/src/elements/prepared.rs`), which the image nodes call (`crates/image/src/executors.rs`) and draw with `crates/image/src/text.rs`.
  - `opsis-layout` (`crates/layout`): `lay_out` builds a Taffy tree of its own (`crates/layout/src/tree.rs`, Taffy's low-level traits, `calc()` as formula tokens) from scene nodes, with inherited text styles and units (`crates/layout/src/computed.rs`: `em`, `rem`, `%`, `vw`/`vh`, `lh`, `ch` and `ex` from the face's `0` advance and x-height, `calc`/`min`/`max`/`clamp`, container units), and returns every border box, text line box and client rect after transforms and perspective (`crates/layout/src/output.rs`, `crates/layout/src/transform.rs`).
  - Fonts are registered by CSS family name from their bytes (`crates/layout/src/fonts.rs`); Parley never reads the system's fonts.
  - Inline content (`crates/layout/src/inline.rs`) follows Chrome's line box model: ascent and descent rounded, half leading floored, line heights floored to 1/64 px, Chromium's break table (`CHROMIUM_LINE_BREAK_OVERRIDE`) except in `word-break: break-all` paragraphs, tab stops at `tab-size` spaces, soft hyphens drawn and fitted, hanging trailing spaces, `vertical-align` with Blink's shifts, span padding, borders and margins as zero-height inline boxes, relative offsets on spans, inline-block margins and baselines.
  - Container units lay the tree out again with each container's measured content box, up to four passes, since a container's size never depends on its contents.
  - Replaced elements keep their picture's ratio when one side is set; picture sizes come from the caller (`Images`).
  - The layout property cases (`crates/layout/tests/{size,flex,text,values,boxes,transforms,media}.rs`) match Chrome's boxes from the oracle, every element's border box within 0.5 px and every text line within 1.5 px; `support::check` reports both sides' lines on a difference.
- **In place on the branch too:** the measurer elements call (`Measure`, `crates/scene/src/measure.rs`, served by `opsis-layout`'s `Measurer`), and CSS stacking order in paint (`Painter::collect`, `crates/paint/src/lib.rs`: negative, zero and positive `z-index` lists, positioned boxes as their own entries, and the properties that make a stacking context).
- **Missing:** nothing. Shaped paragraphs are cached across frames and an unchanged scene reuses its picture (**Scene**); Taffy lays the boxes out again each frame.

### Work

1. Done: Taffy and Parley pinned in the root table.
2. Styles into Taffy and inline content measured by Parley: done; results cached by hash to do.
3. Done: inheritance, units and stacking order.
4. Done: the layout property cases against the oracle (50 cases; the canvas, SVG and paint-only cases are checked by paint).

### Decisions

- Decided: **bundled fallbacks for characters the bundled faces lack: Noto Sans, then Noto Sans Symbols 2, then DejaVu Sans.** The bundled faces are Latin subsets; `fonts:bundle` adds Noto Sans's other subsets as the hidden `Opsis Fallback` family (`public/fonts/bundled/fallback`), the terminal's Noto Sans Symbols 2 (`opsis-terminal-symbols-2`) follows it for symbols such as ⌘, and the whole of DejaVu Sans (`opsis-fallback-dejavu`) comes last for what neither has, such as the block elements (▍). Each subset is registered by `Fonts::register_fallback` in that order (`FALLBACK_FAMILIES`, `crates/film/src/lib.rs`) and appended to every font stack, never the computer's fonts. The oracle's Chrome drew those characters from the computer's fonts, so the mixed-scripts cell of `text/wrapping` stays out of the comparison.

### Risks

- **The subset is Opsis's own code**, so its correctness rests on the property cases. Blitz (`blitz-dom`, MIT/Apache-2.0) does the same job on Taffy and Parley and is the reference to read; it is at `0.3.0-beta.4` with one main author, drops 3D transforms and has no `filter: url(…)`, so it is not a dependency.
- **No `text-wrap: balance` and no automatic hyphenation in Parley.** No element uses either today; soft hyphens are handled by the layout.
- **`vertical-align: top` and `bottom` on text spans align as `baseline`**; on inline-blocks they follow the line box. No element sets them on spans.
- **A soft hyphen is drawn with `-`**, as Chrome does when the face has no U+2010, which is true of every bundled Latin subset.
- **Taffy and Parley break their APIs each 0.x minor**, as their changelogs list (Taffy 0.15.1, Parley 0.12.0).

### Sources

- Taffy (https://github.com/DioxusLabs/taffy): `README.md`, `CHANGELOG.md`, `test_fixtures/`, `scripts/gentest/src/main.rs`. Parley (https://github.com/linebender/parley): `CHANGELOG.md`, `parley_tests/linebreaking_cases`. Blitz (https://github.com/DioxusLabs/blitz): `packages/blitz-dom/src/stylo_to_kurbo.rs`, `packages/blitz-paint/src/filters.rs`.

## Paint

### What it changes

- **A new crate, `opsis-paint`, draws positioned boxes through `opsis-skia`** into an N32 or F16 surface: backgrounds in layers with size, position and repeat, every gradient kind with OKLCH interpolation, borders by side, rounded corners, blurred shadows, outlines, `saveLayer` with opacity and blend modes and `SaveLayerRec::fBackdrop` for backdrop blur, `SkImageFilters` (blur, drop shadow, colour matrix, displacement map, merge, blend, offset, image), `SkPerlinNoiseShader` for turbulence, `SkM44` for perspective, `setDither`.
- **Text is drawn from Parley's glyph runs**, from the same font bytes, with shadows, stroke, decorations and gradient fills (`background-clip: text`, 5 elements) as a clip from the glyph outlines.
- **Inline SVG is drawn from its tree** through `usvg` and the same Skia calls, filters included.
- **A 2D drawing callback gets a canvas**, for the seven files that draw on a page canvas today.
- **One font manager built from font bytes on every OS** (Fontations or an embedded FreeType), never the system's, so glyphs don't change by operating system. The film already renders unhinted (`--font-render-hinting=none`, `crates/capture/src/film.rs`).
- **Skottie plays Lottie files**, with no expression evaluator attached.

### Fit with Opsis

- **In place:** the film draws through Skia today, inside Chrome, so blur, gradient and anti-aliasing maths stay closest to current output. `usvg` and `skrifa` are already dependencies (`crates/prepare`).
  - `opsis-skia` builds Skia with Chrome's Linux defines (`CHROME_LINUX_DEFINES` in `scripts/skia-build.ts`: text gamma 1.2 and contrast 0.2 without gamma on greyscale masks, and the legacy blur, rounded-rect and sampling flags Chrome sets) and wraps a canvas, paints, shaders (CSS gradients interpolated in any CSS colour space by `SkGradient`, pictures, noise), image filters, paths, typefaces from font bytes (FreeType through the empty font manager) and pictures.
  - `opsis-paint` (`crates/paint`) draws `lay_out`'s boxes in CSS stacking order (`Painter::paint_context`, `crates/paint/src/lib.rs`): shadows, background colours and layers (gradients, pictures and SVG pictures, sized, positioned, repeated and blended in an isolated group), borders by side with Chrome's dash and dot ratios, outlines, replaced pictures with `object-fit` snapped to whole pixels as Chrome does, text from the layout's glyph runs with shadows, strokes, paint order and decorations, `background-clip: text`, and effects in CSS order (backdrop filter, clip path, then opacity and blend, filter, mask).
  - Colours resolve through linebender's `color` (`crates/paint/src/color.rs`); `color-mix` percentages are Opsis's own.
  - Inline SVG, SVG pictures and `clip-path: url(#…)` go through `usvg` (`crates/paint/src/svg.rs`, `crates/paint/src/clips.rs`), its fonts the registered ones; SVG filters become Skia image-filter graphs with `color-interpolation-filters` (`crates/paint/src/filters.rs`).
  - 2D canvas drawing is a scene leaf, `Kind::Drawing` (`crates/scene/src/drawing.rs`): the Canvas 2D calls the film makes, as data, so it hashes and caches like every node. Layout shapes its `fillText` and `strokeText` (`crates/layout/src/drawing.rs`) and paint replays it in an isolated layer clipped to the canvas box (`crates/paint/src/canvas.rs`), path points mapped by the transform current when each is added, as Canvas 2D does.
  - SVG `<image>` and `feImage` read `app:` pictures through the same file reader (`picture_kind`, `crates/paint/src/resources.rs`); `filter: url(#…)` on an HTML element runs the SVG filter in the element's own coordinates.
  - The paint property cases (`crates/paint/tests/cases.rs`, `every_case_draws_like_the_browser`) draw every case, write ours and the error map into `~/.cache/opsis-dev/paint-cases`, and fail when FLIP's mean against the oracle picture is above 0.05; every case is at 0.046 or below.
  - Skottie plays the Lottie leaf (`opsis_skia::Lottie`, `boxes::draw_lottie`): a `.lottie` file is unpacked with its pictures embedded as data URIs (`crates/paint/src/lottie.rs`), the animation seeks to the leaf's frame and fits the content box by `object-fit`; `uses_expressions` finds the expressions Skottie doesn't evaluate, for the element card's warning.
- Decided: **goldens use the app's own Skia**, recorded and compared exactly on x86 Linux with AVX2, so they test the path users run; no second Skia build.
- **In place on the branch too:** the Lottie reader moved into `opsis-probe` (`crates/probe/src/lottie.rs`); importing a Lottie file that uses expressions records `Measured::Expressions`, and the Media file card warns that they don't play.
- **In place on the branch too:** the SVG primitives no element uses (component transfer, convolution, morphology, lighting, tile) through Skia's own filters, checked in `crates/paint/tests/filter_primitives.rs`; no oracle case covers them.
- **Missing:** nothing.

### Work

1. The painter, with every filter, blend mode, clip, mask and perspective case listed in [Appendix: drawing features](#appendix-drawing-features).
2. The font manager from the project's and the app's font bytes; `family_for` keeps choosing faces as today ([fonts.md](../agent-docs/fonts.md)).
3. A fixed CPU path for golden runs: Skia picks AVX2+FMA code at run time on x86 (`src/core/SkOpts.cpp`, `Init_ml3`) and fuses `mad()` on AVX2, AVX-512 and NEON (`src/opts/SkRasterPipeline_opts.h`), so golden runs pin one path. x86 and ARM differ even then; goldens are recorded on x86 Linux.
4. Skottie for the Lottie element, with the expression check that feeds the card's warning.
5. The paint property cases against the oracle; their differences set the tolerance **Goldens and tests** uses.

### Decisions

- Decided: **text edges are greyscale anti-aliased.** Chrome on Linux draws film text with LCD subpixel edges (coloured fringes); native films don't, so text differs from today's exports by the edge colour, within the text cases' tolerance.
- Decided: **glyph advances stay fractional**, as Chrome lays text out on macOS. The Linux recording browser rounds each advance to a whole pixel, so very large text in today's Linux exports sits up to a pixel per word away; fractional advances keep spacing as drawn and widths smooth while sizes animate. The element test records the frames this moves (`ALLOWANCES`, `crates/film/tests/elements.rs`).
- Decided: **3D-transformed content is drawn as vectors through the perspective.** Chrome rasterises the layer first and projects it, which leaves stair-stepped edges at steep angles; the element test records the frame it changes.
- **Filters see content beyond the frame.** A filtered element without a blend mode or backdrop filter is painted unclipped on a surface grown by the filter's reach, then drawn under its ancestors' clips (`paint_offscreen`, `crates/paint/src/lib.rs`), as Chrome feeds a filter the element's whole content.

### Risks

- **Chrome's text and edge smoothing won't match bit for bit**, so the comparison with the oracle carries a tolerance.
- **HTML inside SVG (`foreignObject`) is not drawn.** The film uses it once, the terminal under the lens filter, which ports as an HTML element with `filter: url(#…)`; `svg/foreign-object` stays out of the paint cases.
- **No sourced measurement of Skia's speed on blur, filters, text or whole frames exists**; measure on the example projects early.

### Sources

- Skia (https://github.com/google/skia): `include/effects/{SkGradient.h,SkImageFilters.h,SkPerlinNoiseShader.h}`, `include/core/SkCanvas.h`, `src/core/SkOpts.cpp`, `src/opts/SkRasterPipeline_opts.h`, `modules/skottie/include/Skottie.h`. `usvg` 0.50.0 (https://github.com/linebender/resvg). Benchmarks: https://laurenzv.github.io/vello_chart/ (x86 and ARM data files).

## Effects and 3D

### What it changes

- **A new crate, `opsis-effects`, holds the shader materials and runs their passes** through `opsis-filament`.
- **Every shader effect is a Filament material**: unlit, with its fragment code in GLSL ES 3.0 and its inputs as `sampler2d` parameters, drawn as one full-screen pass into a render target with post-processing off (`samples/procedural_effect.cpp`). The programs are the 8 from `@paper-design/shaders` (Apache-2.0: MeshGradient, GrainGradient, Warp, Swirl, GodRays, PerlinNoise, Dithering, SmokeRing), Opsis's own filters (crt-screen, vhs-filter, camera-lens, tv-power-off, halftone-print, mosaic), the transitions over two scenes (glitch-cut, particle-dissolve), light-leak, launch-anything's text reveal, lens-zoom's radial pass, and the terminal's texture and lens.
- **Built-in materials are compiled by `matc` when Opsis is built**; pack materials are compiled when the pack loads ([node-packs.md](node-packs.md), Shaders).
- **Effects read their input nodes as layers.** Skia draws each input into a pixel buffer, Filament takes it as a texture and runs the pass, and the result comes back as an image layer. Canvas Draw Element (`captureElementImage`, `drawElementImage`) is no longer needed.
- **The backend order:** a GPU through Vulkan on Linux and Windows and Metal on macOS; without one, a software Vulkan driver: the SwiftShader in the pinned headless shell, or the system's lavapipe on Linux. The renderer runs as a child process started with `VK_DRIVER_FILES` set, because choosing the driver inside the running app needs `set_var`.
- **3D is a Filament scene**, built by a new crate, `opsis-stage3d`: lit materials for today's `MeshStandardMaterial`, unlit ones for `MeshBasicMaterial`, up to three directional lights, the indirect light from one prefiltered environment map, the ACES tone mapper, 4× MSAA, transparent planes, and glTF through `gltfio`. Extrude, tube and shape geometry is ported from three.js (MIT).
- **Ambient and hemisphere lights become the indirect light.** Filament's light types are sun, directional, point and two spots (`LightManager.h`).
- **Integer hashes (PCG) in every program.** Four of today's shaders hash with `fract(sin(dot(…)) * 43758.5453)`, which differs between drivers.

### Fit with Opsis

- **In place:** `crates/prepare` makes typeface JSON with `skrifa`, normalises SVG with `usvg` and prepares GLB (Draco decoded with `draco-gltf`), so the 3D inputs are Rust already. The film's shaders are GLSL ES 3.00 today (`packages/video/src/lib/shaders/quad.ts`), with constant-bound loops and `texture()` samples. The pinned Chrome Headless Shell ships SwiftShader on every platform (`libvk_swiftshader.so` 3,986,104 B on linux64).
- **In place on the branch:** `opsis-effects` (materials in `crates/effects/materials`, compiled by `matc` from `DEP_OPSIS_FILAMENT_MATC`), the effect node painting its children into layers, running the program through `Resources::run` and drawing the result back (`paint_effect`, `crates/paint/src/lib.rs`), the film starting Filament the first time a frame has an effect, and the programs so far: `halftone-print`, `crt-screen`, `vhs-filter`, `camera-lens`, `tv-power-off` and the seven `shader-*` backgrounds (Paper Shaders' sizing and noise texture in `materials/include` and `textures`; a pass renders at twice the box like Paper's mount, capped at 1920 × 1080 × 4 pixels). Grain hashed with PCG and Perlin's lattice hash differ from the browser pixel for pixel; `crates/film/tests/elements.rs` allows for them with the reasons written there.
- **Also in place on the branch:** `light-leak`, `redact`'s pixelate, and the shader transitions (`dither-`, `grain-`, `swirl-`, `smoke-` and `particle-dissolve`, `glitch-cut`, `wave-wipe`, `ripple-zoom`). A two-scene transition pairs its sides through slots: the outgoing side wraps its scene in a `Kind::Slot` named by the cut, the incoming side's effect holds an empty slot of that name as its first input, and the painter paints the named scene into that input and hides it where it stands (`Slots`, `crates/paint/src/lib.rs`); with no outgoing side the pass runs at progress 1. Hash-driven frames (glitch-cut's and particle-dissolve's middles, light-leak's glows) can't match the browser pixel for pixel, since its `fract(sin(...))` hash also differs on a GPU; `crates/film/tests/elements.rs` lists them with the reason.
- **3D on the branch:** `opsis_filament_scene` draws meshes with the `standard` lit material, directional lights, the room's prefiltered reflections and SH irradiance (ambient and hemisphere lights added to its first coefficient), four samples and three.js's ACES filmic curve as a custom `ToneMapper`; `opsis-geometry` holds three.js's sphere, torus, rounded-box, plane, shape, extrude (with `earcut` for the lids and `ShapePath.toShapes`'s hole rules) and tube geometry, so compile can measure what it places; `opsis-stage3d` holds the lighting looks and the materials (`standard`, its two-sided copy and the unlit `basic`); `Kind::Stage3d` carries a stage through the scene with its textures (pictures paint resolves, and the card's shadow glow), and `shapes-3d`, `card-3d`, `text-3d` (prepared typefaces through `Measure::typeface`) and `logo-3d` (SVG paths through `Measure::vector`, or a picture on the card) are drawn this way. The room map comes from `bun run room:environment`, which ray-casts three.js's `RoomEnvironment` and runs `cmgen` (mirrored on x to match three.js's lookups).
- **Models and screens:** `opsis_filament_model` loads a prepared GLB through gltfio (ubershaders from `uberarchive`, stb and KTX2 textures) and reports its clips and its bounds at a clip time; `opsis-stage3d` fits each `Model3d` from the bounds sampled over its clip, as `model-3d.tsx` does, adds the ground shadow, and gives quantized positions unit bounds first, since gltfio reads a normalized accessor's `min` and `max` as they are. `screen-3d` draws the device in Filament and lays the page on the projected quad with a `matrix3d`, the glare's repeated tiles as positioned gradients. `crates/render/tests/three_stills.rs` → `draws_3d_elements_natively_as_recorded` compares the 3D stills, allowing the metallic models and the SVG logo a little for the room's reflections.
- **The child process:** Filament runs only in `opsis-renderer` (`crates/effects/src/bin/opsis-renderer.rs`, the loop in `crates/effects/src/child.rs`), one per render worker, started by `opsis-renderer`'s client (`crates/renderer`) with `VK_DRIVER_FILES` pointing at a written driver file when the render draws with Software (the headless shell's SwiftShader). Passes and 3D stages go over its standard input and output as a JSON header and the layer pixels (`crates/renderer/src/protocol.rs`); models and the room's map go once per child. Skia stays on the CPU in the app, so the app links no Filament. `renderer_program` finds the program beside the app or up to five folders above a test; the installers place it beside `opsis` and `opsis-ffmpeg`, `bun start` and the e2e runner build it.
- **Missing:** nothing.

### Work

1. Done: the renderer child process, with the backend order and the "Graphics card / Software" render setting becoming the backend choice.
2. Done: the layer transfer between Skia's buffers and Filament textures, as pixel buffers over the child's pipe.
3. Each program as a material, with PCG hashes, checked against its oracle frames.
4. The 3D scenes and their geometry, against today's stills in `fixtures/prepare/stills`.
5. `screen-3d`'s clip drawn with an `SkM44` perspective from `three/project.ts`'s homography.

### Decisions

- Decided: **the environment map is one prefiltered map made by `cmgen` when Opsis is built and shipped in `public/`**, in place of `RoomEnvironment` through `PMREMGenerator`; it is the same on every machine and costs nothing at the first 3D frame.

### Risks

- **Readback speed**, measured in **Wrappers**.
- **SwiftShader through the headless shell's own Vulkan loader on Windows is untested**, and so is drawing without a window on macOS.
- **Filament's lighting against three.js's** (indirect light for ambient and hemisphere, ACES, MSAA) decides whether `three_stills.rs` passes with its slack of 24 over 99.5% of pixels.
- **Software drivers are slow.** One film-like 1080p filter measured through wgpu on 5 November 2026: RTX 3060 1.7 ms, Intel Iris Xe 6.7 ms, lavapipe 49.8 ms, SwiftShader 143.9 ms a frame. The drivers are the same under Filament; the timings are measured again in **Wrappers**.

### Sources

- Filament: `docs_src/src_markdeep/Materials.md.html` (`shadingModel : unlit`, `sampler2d`, fragment code in ESSL 3.0), `samples/{procedural_effect.cpp,frame_generator.cpp,texturedquad.cpp}`, `filament/include/filament/LightManager.h`, `README.md` (tone mappers, MSAA, image-based lighting, glTF with Draco and morph animation). The Vulkan loader's `VK_DRIVER_FILES`.
- The 5 November timings come from a benchmark not kept in the repository: one 1920×1080 filter shaped like the film's (32 blur taps in a spiral, colour fringing, 4-octave noise grain, a vignette), read back every frame, wgpu 30.0.1 on an i7-12700H, 60 frames per driver; the same kernel in plain Rust on rayon took 261.6 ms a frame.

## Footage and terminal

### What it changes

- **Footage frames reach the renderer as decoded frames.** The FFmpeg decoder per source and its grade graph stay (`crates/footage`); the JPEG 4:4:4 encode (42.3% of the app's CPU in the profiling pass), the HTTP serving and the page's decode go. FFmpeg hands over RGBA with explicit `in_color_matrix` and `in_range`.
- **Terminal pictures are drawn natively** from `opsis-vt`'s frames: cell backgrounds as rects, runs placed by column with the shared text stack and today's glyph narrowing, box-drawing and block sprites from `terminal/sprites.ts` as Rust paths so they stay sharp under camera zoom, texture and lens as shader passes.
- **`TerminalPainter` and the terminal page go**, and live terminal pictures for the desktop come from the same drawing.

### Fit with Opsis

- **In place:** `opsis-footage`'s decoders and grade chain (`decoder.rs`, `chain.rs`), Redact's areas in Rust, `opsis-vt` and the frames format, `crates/render/tests/sprites.rs` against Ghostty's atlases.
- **In place on the branch:** the terminal screen drawn natively (`crates/compile/src/elements/terminal/`: `view.rs`, `screen.rs`, the lens, vignette and grain as SVG filters in `effects.rs`), the sprites ported (`sprites.rs`, matched against `fixtures/scene/terminal-sprites.json` from `sprites.ts`), the frames format in `opsis-types` (`terminal_frames.rs`) so the film reads it through `Measure::terminal`. The terminal example's frame 150 is 0.29% wrong pixels against the 0.2% limit: rows of text sit 0.3 to 1 px off the browser's inside the zoomed window.
- **In place on the branch too:** `image`, `video`, `text-mask-video` and `framed-image` drawn natively (`elements/media/`, `text/text_mask_video.rs`, `elements/framed_image.rs`); the oracle has no case for `framed-image`, so it is not yet compared with the browser.
- **In place on the branch too:** footage frames handed over decoded (`Footage::shown` → `NativeFiles::frame`; the JPEG path serves only the browser page), and live terminal pictures drawn natively (`opsis_film::TerminalPainter`, `crates/film/src/terminal.rs`, run by `crates/session/src/terminal_paint.rs` on a blocking thread; `crates/render/tests/terminal.rs` → `paints_a_live_terminal_frame_natively_at_its_screen_size`).
- **Texture and lens as shader passes:** `terminal-grain` runs Skia's own Perlin noise (the lattice and gradients from `perlin_tables`, `crates/compile/src/elements/terminal/effects.rs`, ported from `SkPerlinNoiseShader::PaintingData`, sent as uniform arrays) and converts from linear RGB as the SVG filter did; `terminal-lens` displaces through the bulge map and splits the channels, sampling the nearest pixel as Skia's displacement does. `crates/render/tests/terminal_effects.rs` checks both against the SVG filters they replace.
- **Missing:** nothing.

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
  - `rawvideo` is a required format, and `film_raw_encode_args` (`crates/media/src/codecs.rs`) takes RGBA frames raw, converts them to BT.709 limited range for the YUV codecs and tags every output's colour; `crates/media/tests/raw_frames.rs` checks a colour survives each codec.
  - `render_in_order` (`crates/render/src/chunks.rs`) hands contiguous chunks to workers, each keeping its own state (its decoders) across a chunk, and passes frames through a reorder buffer that holds at most `Schedule::held` frames ahead of the encoder.
  - The exports draw natively: `render_native` and `render_native_still` (`crates/render/src/native.rs`) run `Film` workers through `render_in_order` into `film_raw_encode_args`, resolving `app:`, project and prepared files from disk and footage frames from `Footage::shown` as decoded RGB, with no file server (`NativeFiles`, addresses under `NATIVE_BASE`). `crates/oracle/tests/native_films.rs` runs each example from the oracle's cached takes and compares the films frame by frame.
- **In place on the branch too:** take previews (`crates/render/src/preview.rs`) and `render-elements` (`crates/render/src/bin/render-elements.rs`, with the element specs in `crates/render/src/element_spec.rs`, which the oracle also uses) through the renderer.
- **Missing:** nothing.

### Work

1. `rawvideo` joins the required formats (`REQUIRED_FORMATS`); `film_encode_args` takes raw input with the tags above.
2. Done: the chunk scheduler with the reorder buffer and progress per frame.
3. Done: stills, take previews and `render-elements` through the renderer.

### Decisions

- **Worker count** by measurement on the example projects.

### Risks

- **A 1080p RGBA frame is 8.3 MB**; the reorder buffer is bounded so a slow chunk doesn't hold memory.

### Sources

- `crates/render/src/frames.rs`, `crates/media/src/{codecs,capabilities}.rs`, FFmpeg 6.1.1 `libavcodec/mjpegdec.c:147` (6.1.1 is the Windows program's version; the Linux program is 7.0.2, `plans/ffmpeg.md`).

## Elements

### What it changes

- **Every element's draw function lives in its Rust file** in `crates/compile/src/elements/<category>/`, next to its `#[settings]` struct and `definition()`, and returns scene nodes through a builder whose methods are the CSS property names. One file per element instead of a Rust file and a TSX file.
- **The transform table decides every construct before any element is ported**: one row per CSS property, SVG tag, hook and page call in [Appendix: constructs](#appendix-constructs), with its target. It is `plans/native-renderer-transforms.md` until the cut-over.
- **All 140 implementations in `ELEMENTS` move**: the 138 elements into their own Rust files, and `placeholder` and `still`, which have no element file or category in Rust, with the camera and takes below. By the capabilities they need ([Appendix: elements](#appendix-elements)): layout 71, text 84, vector 93, layer effects 96, SVG filters 12, perspective 7, pixel programs 24, 3D 6, media 20, canvas drawing 5.
- **The camera and takes move**: `core/recording-clip.tsx` and `camera/` (stage, cursor with the macOS cursor SVGs, window frame, keycap HUD, annotations, motion blur, demo clip), captions, the still and placeholder elements.
- **The film's JavaScript libraries are replaced:**
  - `roughjs` by `roughr` as it is, with its own seeded generator, so a hand-drawn mark keeps its seed but not rough.js's exact strokes (`crates/compile/src/elements/parts/hand_drawn.rs`);
  - `culori` by linebender `color` (CSS Color 4 parsing, OKLCH interpolation) plus our own chroma clamp, since `color` only clips to gamut;
  - `svg-path-commander` by `usvg` and `kurbo`;
  - `simplex-noise` by a port with the same output for the same seed;
  - `lottie-web` and `fflate` by Skottie and `zip`;
  - `three` and `@react-three/fiber` by the Filament scene;
  - `@paper-design/shaders` by its GLSL as materials.

### Fit with Opsis

- **In place:** every element's settings, defaults, lengths, design size, fonts and themes are Rust already (`crates/compile/src/elements`, `lengths.rs`, `theme.rs`, `props.rs`). `colors.test.ts`'s allow-list and the theme presets carry over. Most of the film is stateless: `useState` is in 15 of its 275 files and `useEffect` in 11.
- **In place on the branch:** the transform table, the builder, the camera, takes and terminal, and all 132 of the film's implementations drawn natively.
- **Missing:** nothing.

### Work

1. The transform table, and the builder over `opsis-scene`'s style struct.
2. Elements by category, simplest capabilities first; each compared with its oracle frames, one element a unit of work with the same brief: its TSX, the transform table, its oracle frames and the tolerance.
3. The camera, takes and captions.
4. Transitions, with `TransitionWrap`'s pairing of outgoing and incoming sides through slots, and `SceneSeries` for the scenes inside one element.

### Decisions

- None.

### Risks

- **Parity element by element.** The oracle frames are the reference; a difference that is intended is reviewed and recorded.
- **The constructs that are not a lookup** are settled once, in the renderer, before the elements that use them: the shader programs behind 24 elements, the 6 3D elements, page measuring (7 files) and `measureText` (4 files), state and effects (15 and 11 files), 2D canvas drawing (7 files), the one `foreignObject` (the terminal under the lens filter) and container query units (2 files).

### Sources

- `packages/video/src/elements/index.ts`, `packages/video/src/{demo-timeline.tsx,core,camera,lib,parts}`, rough-rs (https://github.com/orhanbalci/rough-rs) `roughr/src/core.rs:363`, the `roughjs` package's `bin/math.js`.

## Goldens and tests

### What it changes

- **The oracle decides parity during the port**: each property case and each element's frames compare with a tolerance, FLIP's mean (`nv-flip` 0.1.2, MIT/Apache/Zlib) or the slack and share `three_stills.rs` uses today, set from the paint property cases.
- **Golden pictures are recorded on one software setup and compared exactly there**: x86 Linux, the fixed Skia CPU path, SwiftShader from the engine folder for effects and 3D.
- **GPU runs compare with a tolerance.**
- **Render tests stop needing a browser page.** The 11 tests `#[ignore]`d for the film or terminal page (`crates/render/tests/frames.rs` 5, `run.rs` 2, `terminal.rs` 2, `sprites.rs` 1, `three_stills.rs` 1) run in-process; those that need SwiftShader stay ignored where it isn't present.
- **Element stills** come from the renderer, and `elements:render` can join `bun run test` once shaders run on the software driver.
- **Scene goldens**: the serialised scene of chosen elements at chosen frames, compared through `opsis-golden`, with no rasterising.

### Fit with Opsis

- **In place:** `opsis-golden`, `fixtures/render`, `fixtures/prepare/stills`, the `OPSIS_WRITE_GOLDEN=1` flow.
- **In place on the branch too:** `crates/render/tests/goldens.rs`: every element's middle frame at a quarter size (`fixtures/elements/<key>.png`) and the scenes of twelve chosen elements at three frames (`fixtures/scene/elements/<key>@<frame>.json`), drawn with the effects and 3D on SwiftShader. On x86 Linux with AVX2 the pictures match exactly, except elements drawn through Filament, which may differ in 0.05% of their pixels, since SwiftShader's multisampling moves a few single highlight pixels between runs; elsewhere FLIP's mean stays under 0.05. The app's own Skia is the golden path: no second Skia build. CI (`.github/workflows/build.yml`) builds Skia and the sealed Filament wrapper with a cache, installs the engine and runs the goldens.
- **Missing:** nothing.

### Work

1. Done: picture goldens for every element at its middle frame, and scene goldens at three frames for chosen elements.
2. Done: CI on x86 Linux with SwiftShader from the engine download.

### Decisions

- None.

### Risks

- **`dssim` is AGPL-3.0** and stays out.

### Sources

- wgpu `examples/features/src/*/mod.rs` (FLIP thresholds), Chromium pixel testing with Gold, `crates/render/tests/three_stills.rs`.

## Cut-over

### What it changes

- **The merge of the branch is the one step in which the browser stops drawing films.** Removed: `apps/film`, `apps/terminal`, `film-bundle`, `terminal-bundle`, `packages/runtime`, the film parts of `packages/video`, `FrameDriver` and `film_switches` (`crates/capture/src/film.rs`), `TerminalPainter`, the `CanvasDrawElement` feature switch and the begin-frame driver.
- **The recording browser stays** for capture, page stills, smooth takes, sign-in and the streamed views; its download is unchanged, and its SwiftShader doubles as the renderer's software driver.
- **The docs move with it**: `film.md` becomes the renderer's doc, `elements.md` and `terminal.md` lose their page parts, `scene.md` is new, AGENTS.md's film rules (`packages/video` purity, oxlint animation bans, `holdRender`) become their Rust equivalents.

### Fit with Opsis

- **In place:** the deletions (`apps/`, `packages/`, the bundles, `crates/oracle`, `scripts/oracle.ts`, the frame driver and the page painter, the footage and prepared-file HTTP routes, the JPEG footage frames, turbo), `scripts/lib/common` and the narrowed `scripts/lib/generated/spec.ts` for what the scripts still took from `packages/video`, `public/` shipped in place of the bundles, the colour check as `scripts/element-colors.test.ts`, the oracle comparisons recorded as goldens (`fixtures/layout`, `fixtures/cases`, `fixtures/sprites`), and the docs (`scene.md`, `film.md` and every doc that named the page).
- **Missing:** nothing.

### Work

1. Move or drop each of those uses of `packages/video` (Fit with Opsis, Missing) before it goes.
2. Delete the listed apps, packages, bundles and Rust paths, the oracle's page and crate (`apps/oracle`, `oracle-bundle`, `crates/oracle`, `scripts/oracle.ts`, with `render-elements` moving onto the renderer as **Output** says) and `plans/native-renderer-transforms.md`; remove their scripts (`film:bundle`, `terminal:bundle`, `oracle:bundle`, `oracle`) and their table rows in AGENTS.md.
3. In AGENTS.md: add `opsis-skia`, `opsis-filament`, `opsis-scene`, `opsis-layout`, `opsis-paint`, `opsis-effects`, `opsis-geometry` and `opsis-stage3d` to the crate table, name `opsis-skia` and `opsis-filament` in the rule on the crates allowed `unsafe`, and add the Filament pin to the table of generated files.
4. Rewrite the docs named above in the same change.

### Decisions

- None.

### Risks

- **There is no fallback after the merge**; the oracle frames and the goldens are the safety net.

### Sources

- `AGENTS.md` §2, §6, §7; `agent-docs/{film,elements,terminal,footage,engine}.md`.

## Appendix: constructs

What the film's source writes, from a scan of `packages/video/src` on 12 November 2026 (275 files, 42,117 lines; tests and generated files left out). A unit is one element, or one file outside `elements/`. The property scan matches object keys against CSS property names, so a few of its hits are not styles (`cx`, `cy`, `offset`, `page`).

- **Styling:** inline style objects only; `className` is passed through in 11 files and names no rule. `video.css` is a reset: `box-sizing: border-box`, zero margin and padding, `border: 0 solid`, `line-height: 1.5`.
- **Size and position, by units:** `width` 107, `height` 106, `position` 92, `left` 62, `inset` 57, `top` 55, `right` 29, `bottom` 22, `maxWidth` 17, `minWidth` 10, `zIndex` 7.
- **Flex and grid:** `display` 80, `alignItems` 72, `justifyContent` 71, `gap` 37, `flexDirection` 27, `flexShrink` 19, `flex` 7, `flexWrap` 6, `placeItems` 6, `gridTemplateColumns` 1.
- **Box:** `background` 82, `borderRadius` 61, `overflow` 59, `padding` 38, `border` 37, `boxShadow` 36, `backgroundColor` 11, `objectFit` 11.
- **Effects:** `opacity` 103, `scale` 50, `transform` 48, `translate` 45, `filter` 43, `transformOrigin` 34, `clipPath` 16, `mixBlendMode` 14, `rotate` 10, `maskImage` 6, `backdropFilter` 4, `backgroundClip` 4, `perspective` 3.
- **Text:** `color` 70, `fontFamily` 69, `fontSize` 61, `letterSpacing` 48, `whiteSpace` 44, `fontWeight` 42, `textAlign` 37, `lineHeight` 29, `fontVariantNumeric` 19, `textTransform` 13, `textOverflow` 10.
- **Values:** `color-mix()`, `oklch()` and `oklab()` 100 times in 29 files; `calc()` 10 times in 4; container query units 6 times in 2.
- **Tags, by occurrences:** `div` 510, `span` 194, `path` 74, `svg` 68, `circle` 34, `g` 20, `defs` 10, `rect` 10, `filter` 10, `text` 9, `stop` 8, `line` 8, `ellipse` 5, `clipPath` 5, `canvas` 5, `img` 3, `linearGradient` 3, `radialGradient` 1, `mask` 1, `foreignObject` 1; filter primitives `feColorMatrix` 7, `feGaussianBlur` 4, `feTurbulence` 4, `feDisplacementMap` 3, `feOffset` 3, `feBlend` 2, `feMerge` 1, `feImage` 1.
- **State and the page, by files:** `useMemo` 24, `useState` 15, `holdRender` or `useHeldLoad` 14, `useRef` 13, `useEffect` 11, `useLayoutEffect` 9, 2D canvas contexts 7, page measuring (`getBoundingClientRect`, `offsetWidth`, `getBBox`, `getTotalLength`, `ResizeObserver` and the like) 7, `measureText` 4.

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
