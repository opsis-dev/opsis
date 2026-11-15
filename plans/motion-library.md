# Motion library

More elements and better ways to find them: frame-driven rewrites of web motion components, a Liquid contours background, two more hand-drawn looks, Rive animations, still thumbnails in the node list, and a CI check that renders every element. Every feature has its own section with the same parts, so the plan can grow one section at a time.

Status: plan only. No feature here is built; the summary marks the ones with pieces already in place.

Every element is a Rust draw function on the native renderer ([native-renderer.md](native-renderer.md), cut over): it returns scene nodes ([agent-docs/scene.md](../agent-docs/scene.md)), Skia paints them, and shaders and 3D run on Filament in the renderer program ([agent-docs/film.md](../agent-docs/film.md)). The elements in this plan are written the same way, as [agent-docs/elements.md](../agent-docs/elements.md) describes.

## How to read this plan

- Each feature is one section. The user picks which to build and when. A large one is split into phases, each checked before the next starts.
- **Status** in the summary is one of: `not built`, `partly in place` (some pieces exist).
- Each section has the same parts: **What it adds**, **Fit with Opsis** (In place / Missing), **Work**, **Licence**, **Risks**, and **Sources** where it cites outside facts.
- External facts carry the date they were checked. Licences were read from each project's own licence file.

## Conventions

These restate rules from `AGENTS.md` and `agent-docs/`; when they disagree with this plan, they win:

- Packages through `cargo add`, latest release.
- Never hand-edit installed or generated code (any `*.generated.*`). Code written from an upstream component is Opsis's own source.
- An element is one Rust file under `crates/compile/src/elements/<category>/`: its settings struct with `schemars` attributes (`x-` keywords), `definition()` and its draw function, with one arm in `draw.rs`.
- Every frame is a pure function of the element's settings, its `Clock` and its stage (`opsis-scene`): `interpolate`, `spring`, easing and `random(seed)` from `crates/scene/src`, sizes from `Measure`. Nothing reads a wall clock, a random source or the network.
- No library, tool or format names in titles, descriptions, labels or errors; the Lottie element is "Animation" ("Plays an animation file from your project…") and `nodes_c.rs` refuses "lottie" in a description.
- No comments anywhere.

**Checks when a feature is done.**

- `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, and the no-comments check.
- `bun run elements:render <key> <positions>` to look at its stills, its golden recorded with `OPSIS_WRITE_GOLDEN=1` (`crates/render/tests/goldens.rs`), and an export of a short clip: a frame drawn on its own must equal the same frame drawn in order.

## Summary

| Feature               | What users get                                                                                                    | Licence                     | Status          |
| --------------------- | ----------------------------------------------------------------------------------------------------------------- | --------------------------- | --------------- |
| Web motion components | 18 new elements: text effects, highlight frames, integration diagrams, a status pill, backgrounds, animated icons | MIT (+ ISC for icon shapes) | not built       |
| Liquid contours       | A flowing two-colour contour background                                                                           | Opsis's own                 | not built       |
| Crossed-off highlight | Hand-drawn crossing out of a part of the page in a take                                                           | MIT (`roughjs`)             | partly in place |
| Rive animations       | Play `.riv` files from the project, offline                                                                       | MIT (Rive runtime)          | not built       |
| Element thumbnails    | A picture of every element in the node list's hover card                                                          | Opsis's own                 | partly in place |
| Visual check in CI    | Every element rendered and compared on each change, so a registry update can't silently break one                 | Opsis's own                 | partly in place |

## Web motion components

### What it adds

Opsis's own frame-driven versions of components from web motion libraries. The originals run on Framer Motion (`motion/react`), CSS keyframes, `requestAnimationFrame`, timers and `Math.random()`; Opsis's version is a Rust draw function that keeps the look and takes its time from the frame.

For users, 18 elements that product demos use often and that Opsis doesn't have yet (each checked against the elements in `crates/compile/src/elements`):

| #   | Title (draft)         | Source file                                                                                                                                                                                                                                             | Category     | What the original does to time, and Opsis's version                                                                                                                                                                                                                                                                             |
| --- | --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Morphing words        | Magic UI `morphing-text`                                                                                                                                                                                                                                | `text`       | `requestAnimationFrame` loop blurs one word into the next through a threshold filter. Word index and blend from `frame / framesPerWord`.                                                                                                                                                                                        |
| 2   | Sparkling text        | Magic UI `sparkles-text`                                                                                                                                                                                                                                | `text`       | `setInterval` + `Math.random` respawn stars. Star `i` lives in slot `floor(t / life)`; position and size from `random(seed + "-" + i + "-" + slot)`.                                                                                                                                                                            |
| 3   | Text around a circle  | Magic UI `spinning-text`                                                                                                                                                                                                                                | `text`       | Motion rotate loop. Rotation `= 360 * t / period`.                                                                                                                                                                                                                                                                              |
| 4   | Morphing letters      | Motion Primitives `text-morph`                                                                                                                                                                                                                          | `text`       | `layoutId` per letter glides shared letters to new places. Measure both words once (see **Risks**), then interpolate each letter's x with `spring()`.                                                                                                                                                                           |
| 5   | Aurora text           | Magic UI `aurora-text`                                                                                                                                                                                                                                  | `text`       | Tailwind `animate-aurora` moves a gradient. `backgroundPosition` from the frame.                                                                                                                                                                                                                                                |
| 6   | Running border        | Magic UI `border-beam`                                                                                                                                                                                                                                  | `overlay`    | Motion animates `offsetDistance`. Same CSS property, value from the frame. Placed over a spot as a fraction of the frame, like Ink arrow.                                                                                                                                                                                       |
| 7   | Glow behind           | Motion Primitives `glow-effect`                                                                                                                                                                                                                         | `overlay`    | Motion rotates/pulses a blurred gradient. Angle and scale from the frame.                                                                                                                                                                                                                                                       |
| 8   | Call-to-action button | Magic UI `shimmer-button`, `pulsating-button`, `rainbow-button`                                                                                                                                                                                         | `ui`         | CSS keyframes (`animate-shimmer-slide`, `animate-spin-around`, `animate-pulse`, `animate-rainbow`). One element with a `look` choice; every keyframe becomes an `interpolate`.                                                                                                                                                  |
| 9   | Status pill           | Cult UI `dynamic-island`                                                                                                                                                                                                                                | `ui`         | Motion layout springs and `setTimeout` between states. A list of states with times; width, height and radius sprung between them. Overlaps Status cycle (`text/status_cycle.rs`), which already rolls a pill's label through states with its width sprung, so it may be new settings on Status cycle rather than a new element. |
| 10  | Connected apps        | Magic UI `animated-beam`                                                                                                                                                                                                                                | `scene`      | Measures icons with `getBoundingClientRect` + `ResizeObserver`, Motion gradient runs along the path. Positions computed from settings, not measured; gradient offset from the frame.                                                                                                                                            |
| 11  | Orbiting logos        | Magic UI `orbiting-circles`                                                                                                                                                                                                                             | `scene`      | `animate-orbit`. Angle per item from the frame.                                                                                                                                                                                                                                                                                 |
| 12  | Logo strip            | Magic UI `marquee`                                                                                                                                                                                                                                      | `scene`      | `animate-marquee`. `translateX` from the frame, wrapping by the strip width (logos are project images; width from their set sizes).                                                                                                                                                                                             |
| 13  | Retro grid            | Magic UI `retro-grid`                                                                                                                                                                                                                                   | `background` | Canvas redrawn in a `requestAnimationFrame` loop. The whole grid as a 2D drawing (`Drawing`) each frame.                                                                                                                                                                                                                        |
| 14  | Flickering grid       | Magic UI `flickering-grid`                                                                                                                                                                                                                              | `background` | Canvas, `Math.random` per tick, `IntersectionObserver` pauses it. Cell opacity from `random(seed + "-" + cell + "-" + floor(t * rate))`; no observer.                                                                                                                                                                           |
| 15  | Meteors               | Magic UI `meteors`                                                                                                                                                                                                                                      | `background` | `Math.random` delays and `animate-meteor`. Seeded start times; position from `(t - start) mod period`.                                                                                                                                                                                                                          |
| 16  | Ripple rings          | Magic UI `ripple`                                                                                                                                                                                                                                       | `background` | `animate-ripple`. Ring scale and opacity from the frame, each ring offset.                                                                                                                                                                                                                                                      |
| 17  | Assistant orb         | SmoothUI `siri-orb`                                                                                                                                                                                                                                     | `background` | Rotating conic gradients through CSS animation. Angles from the frame.                                                                                                                                                                                                                                                          |
| 18  | Animated icon         | lucide-animated (about 20 icons: `check`, `bell`, `sparkles`, `rocket`, `send`, `download`, `upload`, `search`, `settings`, `heart`, `zap`, `lock`, `clock`, `refresh-cw`, `cloud-upload`, `bot`, `party-popper`, `arrow-right`, `copy`, `trending-up`) | `overlay`    | Motion `variants` started on hover (`useAnimation`). One element with an `icon` choice; each icon's variant keyframes become a table of `interpolate` ranges played once at the element's start, or looped.                                                                                                                     |

Left out on purpose because Opsis already has them: number tickers (Rolling Number, Slot Machine Roll), text scramble (Matrix Decode), shimmer text (Shimmer Sweep), typewriters, blur-in text (Soft Blur In), notification lists (Notification stack), terminals, device mockups, file trees, confetti, particles, gauges, hand-drawn highlights (Marker Highlight, Ink Underline, Scribble Circle, and the hand-drawn looks of the Highlight box annotation and the Title card element, drawn by `HandDrawnMark` on `roughjs`). SmoothUI's `soft-blur-in`, `shimmer-sweep`, `mask-reveal-up`, `line-by-line-slide` and `kinetic-center-build` share names with elements Opsis already has; skip them. Hover-, scroll- and pointer-driven components (Magic UI `magic-card`, `lens`, `dock`, `smooth-cursor`, `scroll-progress`, Motion Primitives `magnetic`, `tilt`) have no meaning in a video.

### Fit with Opsis

In place:

- `ElementMeta` (`crates/compile/src/elements/mod.rs`) records nothing about where an element came from; that lives only in `THIRD_PARTY_NOTICES.md`.
- `elements()` lists every element's `definition()` in one ordered list; every entry becomes a node (`element_node_type`: `fx.<name>`) with its card, length handle and envelope. A file per element and a line in that list is all these elements need on the Rust side.
- Settings structs: every element's settings are a struct with `schemars` attributes, collected in `ElementSettings` (`props.rs`); its draw function reads the struct as compile wrote it (`drawn`, `draw.rs`). The `font` widget (`x-widget = "font"`) and `font_stack` (`crates/compile/src/fonts/mod.rs`) turn a family into a stack; `df-main`, `df-heading`, `df-code` follow the project fonts.
- Drawing: `draw.rs` has one arm per key (`element`, `wrapper`, `presentation`, `scenes`); a draw function returns scene nodes in the film's CSS and SVG subset, with leaves for 2D drawings (`Drawing`, the Canvas 2D calls as data), pictures, Lottie, shader effects and 3D stages (`crates/scene/src/node.rs`).
- Each element places itself in its own draw function: centring, a spot's position (a fraction of the frame times its size), screen fitting and its background; `parts/` holds what several elements draw (buttons, toasts, hand-drawn marks, device screens).
- The scene is pure by construction: no clock, random source or file reaches a draw function except through `Clock`, `random(seed)` and `Measure`.
- Placeholder images in `public/placeholders` (`logo-mark.svg`, `screen-wide.svg`, `screen-tall.svg`) for logo and image defaults.

Missing:

- A home for these components, their settings and their loader map.
- `THIRD_PARTY_NOTICES.md` entries for Magic UI, Motion Primitives, Cult UI, SmoothUI, lucide-animated and Lucide's icon shapes.
- A written recipe (below), so every component is done the same way.

### Work

1. **Where code goes.** One file per element under `crates/compile/src/elements/<category>/`, with its settings struct, `definition()` and draw function, listed in `elements()`, `ElementSettings` and `draw.rs` ([elements.md](../agent-docs/elements.md), Adding an element). Keys are the element's name (node type `fx.<name>`); nothing in a path or name says where it came from, which goes in `THIRD_PARTY_NOTICES.md` only. Looks: each element's `.look(…)` in its `definition()`, then `bun run glyphs:bundle`.
2. **Recipe** (one checklist per element, in the change description):
   - Read the upstream file and write the element in Rust from it; nothing of the upstream file is kept as source.
   - `motion.div` with `initial` / `animate` / `transition` → a `div` whose style comes from `interpolate` (clamped, with the original easing) or `spring` from `crates/scene/src`. Keyframe arrays (`rotate: [0, -10, 10, -10, 0]`) → `interpolate` with as many input points, spread over the original `duration` in frames. Motion `ease` names map to cubic Bézier easings with the same control points.
   - `variants` + `useAnimation()` / hover handlers → a `start` (seconds, default 0) and `loop` setting; the variant plays from `start`.
   - CSS `@keyframes` / Tailwind `animate-*` → the same property computed from the frame; the scene has no CSS animations.
   - `useEffect` timers, `setInterval`, `requestAnimationFrame` → frame math. Canvas drawing becomes a 2D drawing (`Drawing`) of the whole picture each frame, never incremental.
   - `Math.random()` → `random(seed)` per index with a `seed` setting (min 1).
   - Measuring (`getBoundingClientRect`, `ResizeObserver`, `IntersectionObserver`) → compute layout from settings; when text must be measured, ask `Measure` from the same fonts the film draws with.
   - Fonts: a `font` setting defaulting to the project font, or `.fonts(&[…])` for a family the settings don't name. Images: `asset` settings, defaults from `public/placeholders` as `app:` refs. Colours: `x-widget = "color"` settings.
   - Sizes in pixels at 1920×1080 with `design` set, so the design box scales them to any shape.
   - Length: an `element_timing` entry (`crates/compile/src/elements/lengths.rs`) when the animation has an end, so Length defaults to "Fit the animation".
3. **First batch** (one of each kind, to prove the recipe): Running border (6), Morphing words (1), Flickering grid (14), Animated icon (18) with five icons. Check, then do the rest in two batches.
4. **Copy.** Titles and descriptions in sentence case without source names (the table titles are drafts). `crates/compile/tests/unit/nodes_c.rs` checks the new elements' descriptions and banned words automatically.
5. **Licences.** One `THIRD_PARTY_NOTICES.md` section per source, listing the Opsis files that started from it like the existing entries, with the upstream copyright line, licence text and the upstream commit, plus Lucide (ISC) for the icon shapes. Settings → Licences shows the file.
6. **Tests.** `proptest` for the pure timing helpers (keyframe table → value, orbit angle, flicker slot); the existing test that every element's defaults pass its own schema covers the settings, and each element's middle frame becomes a golden (`goldens.rs`).
7. **Check.** Each element at its default length and at half length in 16:9 and 9:16; a still of a frame equals the same frame of an export; a second render of the same frame is pixel-identical.

### Licence

Checked 2026-10-16 from each repository's licence file:

| Source                                               | Licence                                                                                                                                                                                                                     | Use    |
| ---------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------ |
| Magic UI (`magicuidesign/magicui`)                   | MIT, "Copyright (c) Magic UI"                                                                                                                                                                                               | yes    |
| Motion Primitives (`ibelick/motion-primitives`)      | MIT, "Copyright (c) 2024 ibelick"                                                                                                                                                                                           | yes    |
| Cult UI (`nolly-studio/cult-ui`)                     | MIT, "Copyright (c) 2023 Jordan-Gilliam"                                                                                                                                                                                    | yes    |
| SmoothUI (`educlopez/smoothui`)                      | MIT, "Copyright (c) 2024 Eduardo Calvo"                                                                                                                                                                                     | yes    |
| lucide-animated (`pqoqubbw/icons`)                   | MIT, "Copyright (c) 2024-2026 pqoqubbw"; the shapes are Lucide icons (ISC, as in the installed `lucide`)                                                                                                                    | yes    |
| Aceternity UI                                        | Its own licence: "You cannot re-distribute the Item as a stock image or its source files, regardless of modifications." Opsis's source is public, so using it would redistribute it.                                        | **no** |
| React Bits (`DavidHDev/react-bits`)                  | MIT + Commons Clause: commercial use allowed "so long as you do not sell, sublicense, or redistribute the components themselves".                                                                                           | **no** |
| Animate UI (`imskyleen/animate-ui`), incl. its icons | MIT + Commons Clause: rights granted only "as part of an application, website, or product", and "so long as you do not sell or redistribute the components themselves in their original form—whether alone or in a bundle". | **no** |

The Commons Clause sources and Aceternity are left out because Opsis's repository is GPL-3.0-or-later and public: the files would be redistributed source under terms that forbid it. MIT code keeps its own licence inside its folder, which the GPL allows.

### Risks

- **Derived, not copied.** The code is rewritten but still derives from the upstream file; keep the licence file and notice even when little original code is left.
- **Measuring text.** Morphing letters needs letter positions in two words, from `Measure` (text widths and ink), never from a guess at the font's metrics.
- **Upstream drift.** These elements don't follow upstream changes. The upstream commit in the notice lets a later update diff.
- **Drawings and scale.** A 2D drawing is laid out in design pixels and painted at the render scale, so scaled-down stills and 4K exports stay sharp only when the drawing uses no fixed pixel sizes outside the design box.

### Sources

- [Magic UI components](https://magicui.design/docs/components) and [licence](https://github.com/magicuidesign/magicui/blob/main/LICENSE.md); file contents scanned for `motion/react`, `requestAnimationFrame`, `Math.random`, `animate-*`, observers (checked 2026-10-16)
- [Motion Primitives](https://motion-primitives.com) and [licence](https://github.com/ibelick/motion-primitives/blob/main/LICENCE.md) (checked 2026-10-16)
- [Cult UI](https://www.cult-ui.com) and [licence](https://github.com/nolly-studio/cult-ui/blob/main/LICENSE.md) (checked 2026-10-16)
- [SmoothUI](https://smoothui.dev) and [licence](https://github.com/educlopez/smoothui/blob/main/LICENSE) (checked 2026-10-16)
- [lucide-animated](https://lucide-animated.com) and [licence](https://github.com/pqoqubbw/icons/blob/main/LICENSE), 469 icons (checked 2026-10-16)
- [Aceternity UI licence](https://ui.aceternity.com/licence) (checked 2026-10-16)
- [React Bits licence](https://github.com/DavidHDev/react-bits/blob/main/LICENSE.md), [Animate UI licence](https://github.com/imskyleen/animate-ui/blob/main/LICENSE.md) (checked 2026-10-16)

## Liquid contours

### What it adds

A flowing background of alternating bands in two colours, like the Remotion Elements "Liquid Contours" example. Settings: two colours, band width, shape size, detail, edge softness, seed and how far the bands flow over the element's length.

### Fit with Opsis

In place:

- Background elements live in `crates/compile/src/elements/background/`, one file each; the shader backgrounds (`shader_swirl.rs`, `shader_grain_gradient.rs`, `shader_mesh_gradient.rs`…) return an effect node that runs a Filament material from `crates/effects/materials/*.mat` in the renderer program ([film.md](../agent-docs/film.md), Effects and 3D).
- Every hash in the materials is integer (PCG), so a pass gives the same pixels on every driver.
- Export's **Draw with** setting (`Gl` in `crates/compile/src/nodes/output.rs`) chooses the graphics card or Software (the dev engine's SwiftShader).

Missing:

- The contour shader: bands of a seeded noise field, thresholded into two colours with a soft edge, the field moved by a phase.

### Work

1. **Settings.** `crates/compile/src/elements/background/liquid_contours.rs` (`ElementCategory::Background`, 150 frames): `first_color`, `second_color`, `spacing`, `scale`, `complexity`, `smoothness` (advanced), `seed` (min 1), `flow` (how much the phase moves over the element, default 1), with `design` at 1920×1080.
2. **Drawing.** A material `crates/effects/materials/shader-liquid-contours.mat` with an integer hash, and the element's draw function returning its effect node with the phase `flow * frame / length` from its `Clock`, like the other shader backgrounds.
3. **Copy.** "Liquid contours", "A flowing background of two-colour bands."
4. **Check.** Export a still and a video with the default drawing and with Software, 9:16.

### Licence

Opsis's own shader. The example on the Remotion Elements page states no licence and is only the look to aim for; nothing is copied from it.

### Risks

- **Banding.** Smooth bands band under H.264; keep `smoothness` low by default or add a faint grain.

### Sources

- [Remotion Elements: Liquid Contours](https://www.remotion.dev/elements/backgrounds/liquid-contours/) (the look; checked 2026-10-16)

## Crossed-off highlight

### What it adds

A hand-drawn cross over a part of the page in a take. The Remotion Elements "Crossed Off" example is `<CrossedOff>` from `@remotion/rough-notation` with `progress` from `interpolate`, `iterations: 10`, `roughness: 2` (checked 2026-10-16).

### Fit with Opsis

In place:

- The hand-drawn marks (`mark_paths`, `crates/compile/src/elements/parts/hand_drawn.rs`) draw Highlight box's Box, Circle, Underline and Marker looks with `roughr`, seeded and stroke by stroke; none crosses anything out.
- Text highlight (`crates/compile/src/elements/text/text_highlight.rs`) has a Strikethrough preset: a straight line through text in an element, not a mark over a take. Don't add a separate element.

Missing:

- Highlight box's looks are Clean, Hand-drawn box, Circle, Underline and Marker (`HighlightLook` and its labels in `crates/types/src/recording.rs`).

### Work

1. Add `CrossedOff` and `StruckThrough` to `HighlightLook` with their labels ("Crossed out", "Struck through"), and draw them in the highlight annotations (`crates/compile/src/elements/camera/annotations.rs`) through two new marks in `mark_paths` (`parts/hand_drawn.rs`): crossed out is strokes corner to corner both ways, and struck through is strokes back and forth across the middle of the box.

### Licence

MIT (`roughjs`), already listed in `THIRD_PARTY_NOTICES.md`.

### Risks

- Pass the highlight's own seed to the marks, so each highlight's strokes stay the same on every frame. Sizes come from the measured box, as for the other looks.

### Sources

- [Remotion Elements: Crossed Off](https://www.remotion.dev/elements/text/crossed-off/) (checked 2026-10-16)

## Rive animations

### What it adds

A **Rive animation** element that plays a `.riv` file from the project: pick the file, the artboard and the animation by name, and the fit. Like the Lottie element, the file is copied into the project's assets. It works with no internet: the runtime ships with the app.

### Fit with Opsis

In place:

- The Lottie element is the pattern: an `asset` setting accepting `.json,.lottie` through `x-accept`, drawn as a Lottie leaf (`Kind::Lottie`, `LottieFrame` in `crates/scene/src/node.rs`) that Skia's Skottie paints at the frame's time, with its timing from `Measure`.
- `import_assets` (`crates/project/src/import.rs`) copies any file into `assets/`.
- Every film is drawn by the renderer in Rust and C++ (Skia, Filament); nothing runs a web page or WASM.

Missing:

- A Rive runtime in the renderer. The web runtime (`@rive-app/canvas-advanced` and its `rive.wasm`) needs a web page, and the renderer runs none. The candidate is Rive's own C++ runtime, built like Skia and drawn into a leaf of its own, which needs its build, its renderer and its licence checked before any work.

### Work

1. **Runtime.** Decide how Rive's C++ runtime draws in the renderer (see **Decisions**), pin and build it through `bun run setup` like Skia, and add a `Rive` leaf to the scene beside `Lottie`.
2. **Settings.** `rive` in its own file under `crates/compile/src/elements/`: `src` (`asset`, `x-accept = ".riv"`), `artboard` and `animation` (text, empty = the default), `fit` ("Show all" / "Fill the frame"), `background`; 90 frames. Every frame sets the animation's time from the frame (wrapping for loop); state machines stay out of scope, since they need inputs over time.
3. **Copy.** No tool or format name in the title or description: title "Interactive animation", description in the Lottie element's style: "Plays an interactive animation file from your project." Add "rive" to the banned words in `crates/compile/tests/unit/nodes_c.rs`.
4. **Licences.** `THIRD_PARTY_NOTICES.md`: the Rive runtime.
5. **Check.** With networking off, an export draws the element; a still of a frame equals the same frame of an export.

### Decisions

- **How the runtime draws**: through Skia in the renderer, so its frames composite like every other layer, or through a renderer of its own into a picture.

### Licence

To check from the C++ runtime's own repository before it is used.

### Risks

- **Referenced files.** A `.riv` file can reference images and fonts by URL; the runtime must load only what the file embeds.

## Element thumbnails

### What it adds

In the node list, hovering an element shows a picture of it at the top of the hover card. Works offline and costs nothing at run time.

### Fit with Opsis

In place:

- The hover card: the node list's `HoverCard` (`crates/qt/cpp/node_list.cc`) shows the title, category, description and ports.
- `ElementMeta.thumbnail` exists in `crates/compile/src/elements/mod.rs` and is never filled.
- `render-elements` (`crates/render/src/bin/render-elements.rs`, `bun run elements:render`) already compiles each element with its defaults, wires transitions, wrappers and sequencers to Background clips, and draws one frame through the renderer into `~/.cache/opsis-dev/element-stills` as 480 px JPEG (`STILL_WIDTH`).

Missing:

- WebP output (`StillFormat` has PNG and JPEG), and a place in the app for the stills.

### Work

1. **Stills.** Extend `render-elements` with a `--thumbnails` mode: every element, overlays and text over a neutral dark `solid`, the frame at 60% of the default length (a per-key override list for exceptions), Software drawing (`Gl::Software`) so the result doesn't depend on the graphics card, 320×180, WebP quality 75 (add `Webp` to `StillFormat`).
2. **Storage.** `crates/qt/resources/element-stills/<key with "/" as "--">.webp`, editor-only, listed in `crates/qt/resources/resources.qrc` (the one resource file `crates/qt/build.rs` compiles with `rcc`) and read by the hover card from `:/opsis/element-stills/…`.
3. **Budget.** At most 24 KB per still and 3.5 MB in all (about 140 elements today, 160 with the web motion components, × 15–20 KB); the binary fails above either.
4. **Hover card.** The node list's hover card (`NodeList::ShowCard`, `crates/qt/cpp/node_list.cc`) shows the still of the hovered element at the top; the element key it needs reaches it through `bridge::PaletteEntry` (`crates/qt/src/bridge.rs`).
5. **Check.** Offline, every element in the list shows a still.

### Decided

- **No live preview in the hover card.** The editor has no Player and nothing streams a film. The card shows the still only.

### Licence

Opsis's own renders of its elements; the elements' own notices already cover them.

### Risks

- **Elements that need a file** (Image, Video, Lottie, Rive, Looping video) render empty at defaults; give them a placeholder from `public/placeholders` in the binary, or a fixed picture.

## Visual check in CI

### What it adds

On every change that touches elements, CI draws every element and compares it with its stored picture, so a change that breaks an element fails the build instead of reaching users.

### Fit with Opsis

In place:

- **Every element's middle frame is a golden picture** (`fixtures/elements/<key>.png`, `crates/render/tests/goldens.rs`), compared through `opsis-golden`, and twelve chosen elements' scenes at three frames (`fixtures/scene/elements`).
- **The `checks` job of `.github/workflows/build.yml` runs them** on the software graphics driver ("Compare every element with its golden picture on the software graphics driver": `cargo build -p opsis-effects --bin opsis-renderer`, then `cargo test -p opsis-render --test goldens -- --ignored`).
- A changed picture is recorded again with `OPSIS_WRITE_GOLDEN=1` and reviewed.

Missing:

- `pull_request` and `workflow_dispatch` triggers: `build.yml` runs on `push` only.
- A side-by-side picture (stored | new) uploaded for each failing element, so a failure can be seen without running it locally.

### Work

1. **Failure pictures.** The goldens test writes the new picture beside the stored one for each element that differs, into a folder the workflow uploads on failure.
2. **Triggers.** `pull_request` on `build.yml`; check the workflow with `actionlint`.
3. **Check.** Break one element on purpose (change a default colour); the job fails with its pictures. Revert; it passes twice in a row.

### Licence

No new packages.

### Risks

- **Noise is not a bug.** An element that differs between runs isn't seeded somewhere; fix the element, never lower the tolerance.

## Decided

- **First batch:** the 18 web motion components and the four-element first batch as listed.
- **Thumbnails:** stills only, no live preview in the hover card.
