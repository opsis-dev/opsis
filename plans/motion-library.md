# Motion library

More elements and better ways to find them: frame-driven rewrites of web motion components, a Liquid contours background, two more hand-drawn looks, Rive animations, still thumbnails in the node list, and a CI check that renders every element. Every feature has its own section with the same parts, so the plan can grow one section at a time.

Status: plan only. Nothing here is built yet.

## How to read this plan

- Each feature is one section. The user picks which to build and when. A large one is split into phases, each checked before the next starts.
- **Status** in the summary is one of: `not built`, `partly in place` (some pieces exist).
- Each section has the same parts: **What it adds**, **Fit with Opsis** (In place / Missing), **Work**, **Licence**, **Risks**, **Sources**.
- External facts carry the date they were checked. Licences were read from each project's own licence file.

## Conventions

These restate rules from `AGENTS.md`; when the two disagree, `AGENTS.md` wins:

- Every `remotion` / `@remotion/*` package stays at one exact version (today `4.0.537`, `bun add --exact`). Other packages through `bun add` / `cargo add`, latest release.
- Never hand-edit `packages/video/src/elements/**`, `lib/**`, `hooks/**` or any `*.generated.*`. Code written from an upstream component is Opsis's own and never goes there.
- Element settings are Rust structs in `opsis-compile` with `schemars` attributes (`x-` keywords); components are TS in `packages/video`, keyed separately.
- Every animation in `packages/video` is a pure function of `useCurrentFrame()`; randomness through `random(seed)`; async through `delayRender`. Nothing loads from the internet at preview or render time.
- No library, tool or format names in titles, descriptions, labels or errors (the Lottie element's description is the one precedent for naming the tool that made a file).
- No comments anywhere, kebab-case TS file names.

**Checks when a feature is done.**

- `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, and the no-comments check.
- `bun run video:bundle`, then export a short clip with the new feature and compare it with the editor Preview: preview must equal export. Scrub backwards in the Preview and check the picture matches a forward play.

## Summary

| Feature               | What users get                                                                                                    | Licence                                | Status          |
| --------------------- | ----------------------------------------------------------------------------------------------------------------- | -------------------------------------- | --------------- |
| Web motion components | 18 new elements: text effects, highlight frames, integration diagrams, a status pill, backgrounds, animated icons | MIT (+ ISC for icon shapes)            | not built       |
| Liquid contours       | A flowing two-colour contour background                                                                           | Remotion licence (`@remotion/effects`) | not built       |
| Crossed-off highlight | Hand-drawn crossing out of a part of the page in a take                                                           | MIT (`@remotion/rough-notation`)       | partly in place |
| Rive animations       | Play `.riv` files from the project, offline                                                                       | MIT (Rive runtime), Remotion licence   | not built       |
| Element thumbnails    | A picture of every element in the node list's hover card                                                          | Opsis's own                            | partly in place |
| Visual check in CI    | Every element rendered and compared on each change, so a registry update can't silently break one                 | Opsis's own                            | not built       |

## Web motion components

### What it adds

Opsis's own frame-driven versions of components from web motion libraries. The originals run on Framer Motion (`motion/react`), CSS keyframes, `requestAnimationFrame`, timers and `Math.random()`, none of which survive a render; Opsis's version keeps the look and takes its time from the frame.

For users, 18 elements that product demos use often and that Opsis doesn't have yet (each checked against the elements in `crates/compile/src/elements`):

| #   | Title (draft)         | Source file                                                                                                                                                                                                                                             | Category     | What the original does to time, and Opsis's version                                                                                                                                                         |
| --- | --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Morphing words        | Magic UI `morphing-text`                                                                                                                                                                                                                                | `text`       | `requestAnimationFrame` loop blurs one word into the next through a threshold filter. Word index and blend from `frame / framesPerWord`.                                                                    |
| 2   | Sparkling text        | Magic UI `sparkles-text`                                                                                                                                                                                                                                | `text`       | `setInterval` + `Math.random` respawn stars. Star `i` lives in slot `floor(t / life)`; position and size from `random(seed + "-" + i + "-" + slot)`.                                                        |
| 3   | Text around a circle  | Magic UI `spinning-text`                                                                                                                                                                                                                                | `text`       | Motion rotate loop. Rotation `= 360 * t / period`.                                                                                                                                                          |
| 4   | Morphing letters      | Motion Primitives `text-morph`                                                                                                                                                                                                                          | `text`       | `layoutId` per letter glides shared letters to new places. Measure both words once (see **Risks**), then interpolate each letter's x with `spring()`.                                                       |
| 5   | Aurora text           | Magic UI `aurora-text`                                                                                                                                                                                                                                  | `text`       | Tailwind `animate-aurora` moves a gradient. `backgroundPosition` from the frame.                                                                                                                            |
| 6   | Running border        | Magic UI `border-beam`                                                                                                                                                                                                                                  | `overlay`    | Motion animates `offsetDistance`. Same CSS property, value from the frame. Placed over a spot with `placedAt`.                                                                                              |
| 7   | Glow behind           | Motion Primitives `glow-effect`                                                                                                                                                                                                                         | `overlay`    | Motion rotates/pulses a blurred gradient. Angle and scale from the frame.                                                                                                                                   |
| 8   | Call-to-action button | Magic UI `shimmer-button`, `pulsating-button`, `rainbow-button`                                                                                                                                                                                         | `ui`         | CSS keyframes (`animate-shimmer-slide`, `animate-spin-around`, `animate-pulse`, `animate-rainbow`). One element with a `look` choice; every keyframe becomes an `interpolate`.                              |
| 9   | Status pill           | Cult UI `dynamic-island`                                                                                                                                                                                                                                | `ui`         | Motion layout springs and `setTimeout` between states. A list of states with times; width, height and radius sprung between them.                                                                           |
| 10  | Connected apps        | Magic UI `animated-beam`                                                                                                                                                                                                                                | `scene`      | Measures icons with `getBoundingClientRect` + `ResizeObserver`, Motion gradient runs along the path. Positions computed from settings, not measured; gradient offset from the frame.                        |
| 11  | Orbiting logos        | Magic UI `orbiting-circles`                                                                                                                                                                                                                             | `scene`      | `animate-orbit`. Angle per item from the frame.                                                                                                                                                             |
| 12  | Logo strip            | Magic UI `marquee`                                                                                                                                                                                                                                      | `scene`      | `animate-marquee`. `translateX` from the frame, wrapping by the strip width (logos are project images; width from their set sizes).                                                                         |
| 13  | Retro grid            | Magic UI `retro-grid`                                                                                                                                                                                                                                   | `background` | Canvas redrawn in a `requestAnimationFrame` loop. Redraw in a layout effect keyed on the frame.                                                                                                             |
| 14  | Flickering grid       | Magic UI `flickering-grid`                                                                                                                                                                                                                              | `background` | Canvas, `Math.random` per tick, `IntersectionObserver` pauses it. Cell opacity from `random(seed + "-" + cell + "-" + floor(t * rate))`; no observer.                                                       |
| 15  | Meteors               | Magic UI `meteors`                                                                                                                                                                                                                                      | `background` | `Math.random` delays and `animate-meteor`. Seeded start times; position from `(t - start) mod period`.                                                                                                      |
| 16  | Ripple rings          | Magic UI `ripple`                                                                                                                                                                                                                                       | `background` | `animate-ripple`. Ring scale and opacity from the frame, each ring offset.                                                                                                                                  |
| 17  | Assistant orb         | SmoothUI `siri-orb`                                                                                                                                                                                                                                     | `background` | Rotating conic gradients through CSS animation. Angles from the frame.                                                                                                                                      |
| 18  | Animated icon         | lucide-animated (about 20 icons: `check`, `bell`, `sparkles`, `rocket`, `send`, `download`, `upload`, `search`, `settings`, `heart`, `zap`, `lock`, `clock`, `refresh-cw`, `cloud-upload`, `bot`, `party-popper`, `arrow-right`, `copy`, `trending-up`) | `overlay`    | Motion `variants` started on hover (`useAnimation`). One element with an `icon` choice; each icon's variant keyframes become a table of `interpolate` ranges played once at the element's start, or looped. |

Left out on purpose because Opsis already has them: number tickers (Rolling Number, Slot Machine Roll), text scramble (Matrix Decode), shimmer text (Shimmer Sweep), typewriters, blur-in text (Soft Blur In), notification lists (Notification stack), terminals, device mockups, file trees, confetti, particles, gauges, hand-drawn highlights (Marker Highlight, Ink Underline, Scribble Circle, and the hand-drawn looks of Highlight box and Title card, built on `@remotion/rough-notation`). SmoothUI's `soft-blur-in`, `shimmer-sweep`, `mask-reveal-up`, `line-by-line-slide` and `kinetic-center-build` share names with Remocn elements already ingested; skip them. Hover-, scroll- and pointer-driven components (Magic UI `magic-card`, `lens`, `dock`, `smooth-cursor`, `scroll-progress`, Motion Primitives `magnetic`, `tilt`) have no meaning in a video.

### Fit with Opsis

In place:

- `ElementMeta` (`crates/compile/src/elements/mod.rs`) records nothing about where an element came from; that lives only in `THIRD_PARTY_NOTICES.md`.
- `elements()` lists every element's `definition()` in one ordered list; every entry becomes a node (`element_node_type`: `fx.<name>`) with its card, length handle and envelope. A file per element and a line in that list is all these elements need on the Rust side.
- Settings structs: every element's settings are a struct with `schemars` attributes; all of them, collected in `ElementSettings`, are exported to `packages/video/src/generated/spec.ts`, so components get typed props. The `font` widget (`x-widget = "font"`) and `font_stack` (`crates/compile/src/fonts/mod.rs`) turn a family into a CSS stack; `df-main`, `df-heading`, `df-code` follow the project fonts.
- Loading: `ELEMENTS` (`packages/video/src/elements/index.ts`) imports every element, and `DemoTimeline` looks each up synchronously (`resolveElement`).
- Each element places itself: centring, a spot's position (`numberOr(x, 0.5) * width`), screen fitting (`ScreenFit`, `packages/video/src/lib/layout/screen-fit.tsx`) and its background are done inside its own component; `packages/video/src/lib/settings/shape.ts` holds the setting conversions.
- Guardrails already cover any new folder in `packages/video`: `no-restricted-imports` bans `framer-motion`, `motion`, `gsap` and `useFrame` (`scripts/lib/oxlint-base.ts`), `opsis/no-animate-class` bans Tailwind `animate-*` and `opsis/no-three-clock` bans three.js clocks (`packages/video/oxlint.config.ts`, `scripts/oxlint-plugin-opsis.ts`).
- Placeholder images in `public/placeholders` (`logo-mark.svg`, `screen-wide.svg`, `screen-tall.svg`) for logo and image defaults.

Missing:

- A home for these components, their settings and their loader map.
- `THIRD_PARTY_NOTICES.md` entries for Magic UI, Motion Primitives, Cult UI, SmoothUI, lucide-animated and Lucide's icon shapes.
- A written recipe (below), so every component is done the same way.

### Work

1. **Where code goes.**
   - Components: `packages/video/src/elements/<category>/<name>.tsx`, like every other element; nothing in a path or name says where a component came from.
   - Settings: one file per element under `crates/compile/src/elements/<category>/`, like every other element, listed in `elements()` and added to the props map exported with the other spec types. Keys are the element's name (node type `fx.<name>`), like every other element; where it came from goes in `THIRD_PARTY_NOTICES.md` only.
   - Loading: one entry each in `ELEMENTS` (`packages/video/src/elements/index.ts`).
   - Looks: each element's `.look(…)` in its `definition()`, then `bun run glyphs:bundle`.
2. **Recipe** (one checklist per element, in the change description):
   - Copy the upstream file once into the target folder, then rewrite it. Delete `"use client"`, `cn()` imports from the web app and every comment.
   - `motion.div` with `initial` / `animate` / `transition` → a plain element whose style comes from `interpolate(frame, [start, end], [from, to], { extrapolateLeft: "clamp", extrapolateRight: "clamp", easing })` or `spring({ frame, fps, config })`. Keyframe arrays (`rotate: [0, -10, 10, -10, 0]`) → `interpolate` with as many input points, spread over the original `duration` in frames. Motion `ease` names map to `Easing.bezier` with the same control points.
   - `variants` + `useAnimation()` / hover handlers → a `start` (seconds, default 0) and `loop` setting; the variant plays from `start`.
   - CSS `@keyframes` / Tailwind `animate-*` → the same property computed from the frame. Only when a keyframe can't be rewritten (a long multi-stop gradient), a paused CSS animation (`animationPlayState: "paused"`, `animationDelay` set to minus the element's time in seconds) is allowed, with keyframe names prefixed `df-` and never `spin` (see **Animation names are global** in `AGENTS.md`); record each such case in `AGENTS.md`.
   - `useEffect` timers, `setInterval`, `requestAnimationFrame` → frame math. Canvas drawing happens in a `useLayoutEffect` keyed on the frame, drawing the whole picture each time (never incrementally).
   - `Math.random()` → `random(seed + "-" + index)` with a `seed` setting (min 1).
   - Measuring (`getBoundingClientRect`, `ResizeObserver`, `IntersectionObserver`) → compute layout from settings; when text must be measured, measure once after fonts load under `delayRender`, never per frame.
   - Fonts: a `font` setting defaulting to the project font; no `@remotion/google-fonts` or web font imports. Images: `asset` settings, defaults from `public/placeholders` as `app:` refs. Colours: `x-widget = "color"` settings.
   - Sizes in pixels at 1920×1080 with `design` set, so `DesignBox` scales them to any shape.
   - Length: an `element_timing` entry (`crates/compile/src/elements/lengths.rs`) when the animation has an end, so Length defaults to "Fit the animation".
3. **First batch** (one of each kind, to prove the recipe): Running border (6), Morphing words (1), Flickering grid (14), Animated icon (18) with five icons. Check, then do the rest in two batches.
4. **Copy.** Titles and descriptions in sentence case without source names (the table titles are drafts). `crates/compile/tests/unit/nodes_c.rs` checks the new elements' descriptions and banned words automatically.
5. **Licences.** One `THIRD_PARTY_NOTICES.md` section per source, listing the Opsis files that started from it like the Remocn entry, with the upstream copyright line, licence text and the upstream commit, plus Lucide (ISC) for the icon shapes. Settings → Licences shows the file.
6. **Tests.** Vitest for the pure timing helpers (keyframe table → value, orbit angle, flicker slot); the existing test that every element's defaults pass its own schema covers the settings. Add each element to the visual check (last section).
7. **Check.** Each element at its default length and at half length in 16:9 and 9:16; scrub backwards; export equals Preview; a second render of the same frame is pixel-identical.

### Licence

Checked 2026-10-16 from each repository's licence file:

| Source                                               | Licence                                                                                                                                                                                                                     | Use    |
| ---------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------ |
| Magic UI (`magicuidesign/magicui`)                   | MIT, "Copyright (c) Magic UI"                                                                                                                                                                                               | yes    |
| Motion Primitives (`ibelick/motion-primitives`)      | MIT, "Copyright (c) 2024 ibelick"                                                                                                                                                                                           | yes    |
| Cult UI (`nolly-studio/cult-ui`)                     | MIT, "Copyright (c) 2023 Jordan-Gilliam"                                                                                                                                                                                    | yes    |
| SmoothUI (`educlopez/smoothui`)                      | MIT, "Copyright (c) 2024 Eduardo Calvo"                                                                                                                                                                                     | yes    |
| lucide-animated (`pqoqubbw/icons`)                   | MIT, "Copyright (c) 2024-2026 pqoqubbw"; the shapes are Lucide icons (ISC, as in the installed `lucide-react`)                                                                                                              | yes    |
| Aceternity UI                                        | Its own licence: "You cannot re-distribute the Item as a stock image or its source files, regardless of modifications." Opsis's source is public, so using it would redistribute it.                                        | **no** |
| React Bits (`DavidHDev/react-bits`)                  | MIT + Commons Clause: commercial use allowed "so long as you do not sell, sublicense, or redistribute the components themselves".                                                                                           | **no** |
| Animate UI (`imskyleen/animate-ui`), incl. its icons | MIT + Commons Clause: rights granted only "as part of an application, website, or product", and "so long as you do not sell or redistribute the components themselves in their original form—whether alone or in a bundle". | **no** |

The Commons Clause sources and Aceternity are left out because Opsis's repository is GPL-3.0-or-later and public: the files would be redistributed source under terms that forbid it. MIT code keeps its own licence inside its folder, which the GPL allows.

### Risks

- **Derived, not copied.** The code is rewritten but still derives from the upstream file; keep the licence file and notice even when little original code is left.
- **Measuring text.** Morphing letters needs letter positions in two words. Measure with a hidden copy in a `useLayoutEffect` after `FontProvider` has loaded the font (under `delayRender`); measuring before fonts load gives wrong positions only in the render.
- **Paused CSS animations.** They are frame-exact in Chrome only while the element stays mounted; a remount restarts them at the delay. Prefer frame-computed styles; the escape hatch is for rare cases and must pass the scrub-backwards check.
- **Upstream drift.** These files don't follow upstream changes (unlike registry elements). The upstream commit in the notice lets a later update diff.
- **Canvas size.** Canvas backgrounds must size their backing store from `useVideoConfig()` times the render scale, or scaled-down stills and 4K exports blur.

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

- The Remotion Elements page is example source, not a registry: it gives no install command and no package (checked 2026-10-16). The example is a few lines around `liquidContours()` from `@remotion/effects/liquid-contours`, applied to Remotion's `<Solid>` through its `effects` prop, with `phase` interpolated from the frame.
- `@remotion/effects@4.0.537` is a dependency of `packages/video` (Blur areas and Picture look use it) and exports `./liquid-contours`; its options are `firstColor`, `secondColor`, `spacing` (2–300), `scale` (20–1000), `complexity` (0–1), `smoothness` (0–1), `seed`, `offsetX`, `offsetY`, `phase` (`dist/liquid-contours.d.ts`). It draws with a WebGL2 shader and fetches nothing.
- Background elements live in `packages/video/src/elements/background/`, one file each, with settings in `crates/compile/src/elements/background/`.
- Export's **Draw with** setting (`Gl` in `crates/compile/src/nodes/output.rs`) chooses software drawing (`swangle`).

Missing:

- No core element uses Remotion's `<Solid>`; `backgrounds.tsx` exports its own `Solid`, so import Remotion's under another name.

### Work

1. **Settings.** `crates/compile/src/elements/background/liquid_contours.rs` (`ElementCategory::Background`, 150 frames): `first_color`, `second_color`, `spacing`, `scale`, `complexity`, `smoothness` (advanced), `seed` (min 1), `flow` (how much `phase` moves over the element, default 1), with `design` at 1920×1080.
2. **Component.** `LiquidContours` in `packages/video/src/core/liquid-contours.tsx` draws `<Solid color={secondColor} effects={[liquidContours({ …, phase: 3.23 + flow * frame / durationInFrames })]} />`. It goes in `ELEMENTS` like every other element.
3. **Copy.** "Liquid contours", "A flowing background of two-colour bands."
4. **Check.** Export a still and a video with the default drawing and with Software, 9:16.

### Licence

`@remotion/effects` is under the Remotion licence (npm `UNLICENSED`), covered by the existing Remotion notice. The example on the Remotion Elements page states no licence; Opsis writes its own few lines rather than copying it.

### Risks

- **WebGL2 in software drawing.** If `swangle` can't create a WebGL2 context, the export fails; check, and if so, say so in the **Draw with** help.
- **Banding.** Smooth bands band under H.264; keep `smoothness` low by default or add a faint grain.

### Sources

- [Remotion Elements](https://www.remotion.dev/elements) and [Liquid Contours](https://www.remotion.dev/elements/backgrounds/liquid-contours/) (checked 2026-10-16)
- Package source of `@remotion/effects@4.0.537` (`dist/liquid-contours.d.ts`; checked 2026-10-26)

## Crossed-off highlight

### What it adds

A hand-drawn cross over a part of the page in a take. The Remotion Elements "Crossed Off" example is `<CrossedOff>` from `@remotion/rough-notation` with `progress` from `interpolate`, `iterations: 10`, `roughness: 2` (checked 2026-10-16).

### Fit with Opsis

- Title card already has it: its **Mark a word** look includes "Crossed out" (`MarkLook` in `crates/compile/src/elements/text/title.rs`, drawn by `HandDrawnMark` in `packages/video/src/core/hand-drawn.tsx`). Don't add a separate element.
- Close registry elements: Strikethrough Replace (Remocn) and Text Highlight (RemotionUI, includes a strikethrough).
- The gap: Highlight box's looks are Clean, Hand-drawn box, Circle, Underline and Marker (`HighlightLook`, `crates/types/src/recording.rs`, labels in `crates/compile/src/nodes/cursor.rs`).

### Work

1. Add `CrossedOff` and `StruckThrough` to `HighlightLook` with their labels ("Crossed out", "Struck through"), run `bun run types:export`, and draw them in `SketchedHighlight` (`packages/video/src/camera/annotations.tsx`) with `<CrossedOff>` and `<StrikeThrough>`, as `HandDrawnMark` already does for Title card.

### Licence

MIT (`@remotion/rough-notation`, `roughjs`), already listed in `THIRD_PARTY_NOTICES.md`.

### Risks

- Seed 0 is random in rough-notation; pass the highlight's own seed. Sizes come from the measured box, as for the other looks.

### Sources

- [Remotion Elements: Crossed Off](https://www.remotion.dev/elements/text/crossed-off/) (checked 2026-10-16)

## Rive animations

### What it adds

A **Rive animation** element that plays a `.riv` file from the project: pick the file, the artboard and the animation by name, and the fit. Like the Lottie element, the file is copied into the project's assets. It works with no internet: the runtime ships with the app.

### Fit with Opsis

In place:

- `LottieElement` (`packages/video/src/elements/overlay/lottie.tsx`) is the pattern: an `asset` setting given the host's project address by `hosted`, `fetch()` under `delayRender`, `cancelRender` on failure; its settings accept `.json,.lottie` through `x-accept`.
- `import_assets` (`crates/project/src/import.rs`) copies any file into `assets/`; the file servers are `tower-http`'s `ServeDir` (`crates/engine/src/http_files.rs`), whose MIME table already maps `.wasm` to `application/wasm` and serves unknown types such as `.riv` as `application/octet-stream`.
- Video code runs only in the headless shell (the preview page, `apps/preview`, and the render bundle), never in the editor's webview, so the editor's CSP doesn't apply; the preview page sets none.
- No bundler swaps an import today; the sidecar's Rspack override takes aliases (`makeBundlerOverride` in `apps/sidecar/src/bundler-override.ts`), and `apps/preview`'s Vite config can take a `resolve.alias`. Rive's runtime goes through those.

What the packages do (from the tarballs of `@remotion/rive` and `@rive-app/canvas-advanced@2.31.5`, checked 2026-10-19; `@remotion/rive@4.0.537` still pins 2.31.5, checked 2026-10-26):

- `<RemotionRiveCanvas>` takes `src`, `fit`, `alignment`, `artboard`, `animation`, `onLoad`, `assetLoader` and `enableRiveAssetCdn`. It loads the runtime with `riveCanvas({ locateFile: () => "https://unpkg.com/@rive-app/canvas-advanced@2.31.5/rive.wasm" })`, fixed in the component with no prop to change it.
- The runtime package ships `rive.wasm` itself (1,449,487 bytes, MIT). Its `package.json` has `main: "canvas_advanced.mjs"` and no `exports` field, so `@rive-app/canvas-advanced/canvas_advanced.mjs` can be imported directly.
- `enableRiveAssetCdn` lets the runtime fetch a file's referenced images and fonts from Rive's servers unless it's turned off.
- It advances the animation by `frame - lastFrame` on each frame instead of setting the time.

### Approach (decided)

Use Remotion's `<RemotionRiveCanvas>` and download nothing at run time: both bundlers send `@rive-app/canvas-advanced` to a small Opsis module that calls the real runtime with a local `locateFile`, so the hard-coded unpkg address is never used.

### Work

1. **Dependencies.** In `packages/video`: `bun add --exact @remotion/rive@<the Remotion version>`, and `bun add --exact @rive-app/canvas-advanced@<the version it pins>` because Opsis imports it directly.
2. **Bundled runtime.** `scripts/rive-runtime.ts` (`bun run rive:runtime`) copies `rive.wasm` from the installed package to `public/rive/rive.wasm`, like `fonts:bundle` copies fonts; add the row to the generated-files table in `AGENTS.md`. A test checks the copy matches the installed package byte for byte, so a version bump can't leave an old runtime behind. `rive_fallback.wasm` isn't needed: the advanced runtime never names it.
3. **Runtime redirect.** `packages/video/src/rive/runtime.ts` imports the real `canvas_advanced.mjs` and exports a default function that calls it with `locateFile: () => staticFile("rive/rive.wasm")`, whatever `locateFile` the caller passed. The Vite plugin and the Rspack hook resolve the bare `@rive-app/canvas-advanced` to that module, next to the font redirect; the `.mjs` subpath is left alone, so the wrapper doesn't redirect to itself. A test bundles a module that imports the bare name and checks it resolves to the wrapper in both bundlers.
4. **Settings.** `rive` in its own file under `crates/compile/src/elements/`: `src` (`asset`, `x-accept = ".riv"`), `artboard` and `animation` (text, empty = the default), `fit` ("Show all" / "Fill the frame"), `background`; 90 frames.
5. **Component.** `packages/video/src/elements/<category>/rive.tsx`, in `ELEMENTS`, renders `<RemotionRiveCanvas src={…} artboard={…} animation={…} fit={…} enableRiveAssetCdn={false} />`, with `src` resolved like Lottie's.
6. **Copy.** Title "Rive animation", description in the Lottie element's style: "Plays an animation made in Rive (.riv)."
7. **Licences.** `THIRD_PARTY_NOTICES.md`: `@rive-app/canvas-advanced` (MIT), including the bundled `rive.wasm`. `@remotion/rive` is under the Remotion licence, covered by the existing notice.
8. **Check.**
   - With networking off: the element loads in the Preview and in an export, and no request goes to unpkg or Rive's servers (the preview page's network log through CDP).
   - Frame stepping: an exported still of the middle frame (a fresh page jumping straight to one frame) matches the same frame in the export; scrubbing backwards and jumping around in the Preview match the export too. If they don't, use the fallback below.

### Fallback: Opsis's own component

If stepping from the previous frame gives wrong pictures, write `rive.tsx` on the runtime directly instead of `<RemotionRiveCanvas>`: load the runtime through the same wrapper, `rive.load(bytes, undefined, false)`, and on each frame set `LinearAnimationInstance.time` from the frame (wrapping for loop), then `apply(1)`, `artboard.advance(0)` and draw. That makes every frame a pure function of the frame number. State machines stay out of scope either way: they need inputs over time.

### Licence

`@rive-app/canvas-advanced` and its `rive.wasm`: MIT (from its `package.json`; the package ships no licence file, so the notice quotes the Rive runtime repository's). `@remotion/rive`: Remotion licence.

### Risks

- **Stepping from the previous frame.** `<RemotionRiveCanvas>` advances by `frame - lastFrame`. Renders use one tab in order, but exported stills render one frame of a fresh page and the Preview scrubs, so check both (Work step 8) before shipping.
- **Version drift.** The wrapper, the copied `rive.wasm` and `@remotion/rive`'s pinned runtime must stay the same version; the copy test and the exact pins catch drift on the next Remotion bump.

### Sources

- [Remotion: @remotion/rive](https://www.remotion.dev/docs/rive/) (checked 2026-10-16)
- [Rive: Preloading WASM](https://rive.app/docs/runtimes/web/preloading-wasm) and [Low-level API usage](https://rive.app/docs/runtimes/web/low-level-api-usage) (`locateFile`, self-hosting the WASM with a matching version; checked 2026-10-17)
- Package source and npm metadata of `@remotion/rive` (`dist/esm/index.mjs`: unpkg `locateFile`, `enableRiveAssetCdn`, `advance(diff / fps)`) and `@rive-app/canvas-advanced@2.31.5` (`package.json`, `rive.wasm`, `rive_advanced.mjs.d.ts`, MIT) (checked 2026-10-19); `npm view @remotion/rive@4.0.537 dependencies` (checked 2026-10-26)

## Element thumbnails

### What it adds

In the node list, hovering an element shows a picture of it at the top of the hover card. Works offline and costs nothing at run time.

### Fit with Opsis

In place:

- The hover card: `NodePreview` in `apps/ui/src/editor/palette.tsx`, inside `HoverCard`, shows the title, category, description and ports.
- `ElementMeta.thumbnail` exists in `crates/compile/src/elements/mod.rs` and is never filled.
- `render-elements` (`crates/render/src/bin/render-elements.rs`, `bun run elements:render`) already compiles each registry element with its defaults in Rust, wires transitions, wrappers and sequencers to solid clips, and renders one frame through the sidecar into `~/.cache/opsis-element-stills` as 480 px JPEG.

Missing:

- Core elements in `render-elements` (it skips `core/`), WebP output (`StillFormat` has PNG and JPEG; the render library's `renderStill` also takes `webp`), a fixed drawing mode, and a place in the app for the stills.

### Work

1. **Stills.** Extend `render-elements` with a `--thumbnails` mode: every element including core ones, overlays and text over a neutral dark `solid`, the frame at 60% of the default length (a per-key override list for exceptions), Software drawing (`Gl::Swangle`) so the result doesn't depend on the graphics card, 320×180, WebP quality 75 (add `Webp` to `StillFormat` and the sidecar's still job).
2. **Storage.** `apps/ui/src/editor/element-stills/<key with "/" as "--">.webp`, editor-only, loaded with `import.meta.glob("./element-stills/*.webp", { query: "?url", import: "default", eager: true })` in a small `element-stills.ts` next to `palette.tsx`. Add the folder and its command to the generated-files table in `AGENTS.md`; run it after changing or adding an element. Remove `ElementMeta.thumbnail`.
3. **Budget.** At most 24 KB per still and 3.5 MB in all (about 140 elements today, 160 with the web motion components, × 15–20 KB); the binary fails above either.
4. **Hover card.** `NodePreview` shows the still of the hovered element at the top.
5. **Check.** Offline, every element in the list shows a still.

### Decided

- **No live preview in the hover card.** The editor's webview has no Player; a streamed preview per hover isn't worth a headless shell tab. The card shows the still only.

### Licence

Opsis's own renders of its elements; the elements' own notices already cover them.

### Risks

- **Elements that need a file** (Image, Video, Lottie, Rive, Looping video) render empty at defaults; give them a placeholder from `public/placeholders` in the binary, or a fixed picture.

### Sources

- Remotion `renderStill` image formats in `@remotion/renderer` (`validStillImageFormats`: png, jpeg, pdf, webp; checked 2026-10-19)

## Visual check in CI

### What it adds

On every pull request that touches elements, CI renders one frame of every element and compares it with the stored still, so a registry update, a Remotion bump or a font change that breaks an element fails the build instead of reaching users.

### Fit with Opsis

In place:

- `.github/workflows/build.yml` has a `checks` job and an `e2e` job that already caches the dev engine, installs it with `install-engine` and runs `bun run video:bundle`.
- The thumbnail stills above are rendered deterministically (software drawing, fixed frame) and are the reference images.
- `opsis-media` runs FFmpeg from the engine.

Missing:

- Any image comparison.

### Work

1. **Compare mode.** `render-elements --check` renders every element as in the thumbnails section into `~/.cache/opsis-element-stills/` and compares each with its stored still with the engine's FFmpeg `ssim` filter (no new dependency). It fails when SSIM is below a threshold (start at 0.97; measure the spread across a few runs first), when an element has no stored still, or when a still has no element, and writes a side-by-side image (`hstack`: stored | new) for each failure.
2. **Workflow.** A job `elements` in `build.yml`, `needs: checks`, Linux, only when the change touches `packages/video/**`, `crates/compile/src/elements/**`, `public/fonts/**`, `package.json`, `packages/video/package.json` or `bun.lock` (a paths check step, since `build.yml` runs on several events). Steps as in `e2e` (engine cache, `install-engine`, `video:bundle`), then `bun run elements:render -- --check`, uploading the side-by-side images on failure. Check the workflow with `actionlint`.
3. **Updating references.** A `workflow_dispatch` input `update-stills` runs the binary in write mode and uploads the stills as an artifact, so references come from the same machine type as the check. Locally on Linux x64, `bun run elements:render -- --thumbnails` writes them directly.
4. **Check.** Break one element on purpose (change a default colour); the job fails with its side-by-side image. Revert; it passes twice in a row.

### Licence

No new packages.

### Risks

- **Time.** About 140 to 160 frames with one tab; shader elements take seconds each, so expect several minutes plus the bundle. Only run when relevant paths change.
- **Different machines draw differently.** Software drawing keeps CI stable; stills written on macOS or Windows may differ slightly, which is why references come from CI or Linux x64.
- **Noise is not a bug.** An element that differs between runs isn't seeded somewhere; fix the element, never lower the threshold.

### Sources

- `.github/workflows/build.yml`, `crates/render/src/bin/render-elements.rs` (read 2026-10-26)
- [FFmpeg filters: ssim](https://ffmpeg.org/ffmpeg-filters.html#ssim) (checked 2026-10-16)

## Decided

- **First batch:** the 18 web motion components and the four-element first batch as listed.
- **Thumbnails:** stills only, no live preview in the hover card.
