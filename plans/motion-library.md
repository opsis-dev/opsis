# Motion library

What's left of milestone M3 of the original spec ("port the top 20 web components", Remotion Elements): which web motion components Opsis should rewrite as frame-driven elements, the two missing Remotion Elements, Rive animations, still thumbnails for every element, and a CI check that renders each element. Every feature has its own section with the same parts, so the plan can grow one section at a time.

Status: plan only. Nothing here is built yet.

## How to read this plan

- Each feature is one section. The user picks which to build and when. A large one is split into phases, each checked before the next starts.
- **Status** in the summary is one of: `not built`, `partly in place` (some pieces exist), `covered elsewhere` (another plan builds it).
- Each section has the same parts: **What it adds**, **Fit with Opsis** (Already in place / Missing), **Work**, **Licence**, **Risks**, **Sources**.
- External facts carry the date they were checked. Licences were read from each project's own licence file.

## Conventions

These restate rules from `AGENTS.md`; when the two disagree, `AGENTS.md` wins:

- Every `remotion` / `@remotion/*` package stays at exact `4.0.536` (`bun add --exact`). Other packages through `bun add` / `bun add -d`, latest release. Never edit dependency blocks by hand.
- Never hand-edit `src/video/elements/**`, `src/video/lib/**`, `src/video/hooks/**` or any `*.generated.ts`. Ported code is Opsis's own code and never goes there.
- Every animation in `src/video` is a pure function of `useCurrentFrame()`; randomness through `random(seed)`; async through `delayRender`. Nothing loads from the internet at preview or render time.
- No library, tool or format names in titles, descriptions, labels or errors (the Lottie element's description is the one precedent for naming the tool that made a file).
- No comments anywhere, kebab-case file names.

**Checks when a feature is done.**

- `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`.
- The no-comments check from `AGENTS.md`.
- Export a short clip with the new feature and compare it with the editor Preview: preview must equal export. Scrub backwards in the Preview and check the picture matches a forward play.
- Don't commit or push.

## Summary

| Feature                      | What users get                                                                                                    | Licence                                | Status          |
| ---------------------------- | ----------------------------------------------------------------------------------------------------------------- | -------------------------------------- | --------------- |
| Ported web motion components | 18 new elements: text effects, highlight frames, integration diagrams, a status pill, backgrounds, animated icons | MIT (+ ISC for icon shapes)            | not built       |
| Liquid contours background   | A flowing two-colour contour background                                                                           | Remotion licence (`@remotion/effects`) | not built       |
| Crossed-off marker           | A hand-drawn cross over a word or a part of the page                                                              | MIT (`@remotion/rough-notation`)       | partly in place |
| Rive animations              | Play `.riv` files from the project, offline                                                                       | MIT (Rive runtime), Remotion licence   | not built       |
| Element thumbnails           | A picture (and on hover, a live preview) of every element in the node list                                        | Opsis's own                            | not built       |
| Visual check in CI           | Every element rendered and compared on each change, so a registry update can't silently break one                 | Opsis's own                            | not built       |

## Ported web motion components

### What it adds

Opsis's own frame-driven rewrites of components from web motion libraries. The originals run on Framer Motion (`motion/react`), CSS keyframes, `requestAnimationFrame`, timers and `Math.random()`, none of which survive a render; a port keeps the look and replaces the clock with the frame.

For users, 18 elements that product demos use often and that Opsis doesn't have yet (each checked against the 23 built-in and 109 registry elements in `src/shared/elements/core.ts` and `elements.generated.ts`):

| #   | Title (draft)         | Source file                                                                                                                                                                                                                                             | Category     | What the original does to time, and the port                                                                                                                                                                |
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
| 10  | Connected apps        | Magic UI `animated-beam`                                                                                                                                                                                                                                | `scene`      | Measures icons with `getBoundingClientRect` + `ResizeObserver`, Motion gradient runs along the path. Positions computed from params, not measured; gradient offset from frame.                              |
| 11  | Orbiting logos        | Magic UI `orbiting-circles`                                                                                                                                                                                                                             | `scene`      | `animate-orbit`. Angle per item from the frame.                                                                                                                                                             |
| 12  | Logo strip            | Magic UI `marquee`                                                                                                                                                                                                                                      | `scene`      | `animate-marquee`. `translateX` from the frame, wrapping by the measured strip width (logos are project images; width from `fixedProps` sizes).                                                             |
| 13  | Retro grid            | Magic UI `retro-grid`                                                                                                                                                                                                                                   | `background` | Canvas redrawn in a `requestAnimationFrame` loop. Redraw in a layout effect keyed on the frame.                                                                                                             |
| 14  | Flickering grid       | Magic UI `flickering-grid`                                                                                                                                                                                                                              | `background` | Canvas, `Math.random` per tick, `IntersectionObserver` pauses it. Cell opacity from `random(seed + "-" + cell + "-" + floor(t * rate))`; no observer.                                                       |
| 15  | Meteors               | Magic UI `meteors`                                                                                                                                                                                                                                      | `background` | `Math.random` delays and `animate-meteor`. Seeded start times; position from `(t - start) mod period`.                                                                                                      |
| 16  | Ripple rings          | Magic UI `ripple`                                                                                                                                                                                                                                       | `background` | `animate-ripple`. Ring scale and opacity from the frame, each ring offset.                                                                                                                                  |
| 17  | Assistant orb         | SmoothUI `siri-orb`                                                                                                                                                                                                                                     | `background` | Rotating conic gradients through CSS animation. Angles from the frame.                                                                                                                                      |
| 18  | Animated icon         | lucide-animated (about 20 icons: `check`, `bell`, `sparkles`, `rocket`, `send`, `download`, `upload`, `search`, `settings`, `heart`, `zap`, `lock`, `clock`, `refresh-cw`, `cloud-upload`, `bot`, `party-popper`, `arrow-right`, `copy`, `trending-up`) | `overlay`    | Motion `variants` started on hover (`useAnimation`). One element with an `icon` choice; each icon's variant keyframes become a table of `interpolate` ranges played once at the element's start, or looped. |

Left out on purpose because Opsis already has them: number tickers (Rolling Number, Slot Machine Roll), text scramble (Matrix Decode), shimmer text (Shimmer Sweep), typewriters, blur-in text (Soft Blur In), notification lists (Notification stack), terminals, device mockups, file trees, confetti, particles, gauges, hand-drawn highlights (Marker Highlight, Ink Underline, Scribble Circle, and the hand-drawn looks of Highlight box and Title card, built on `@remotion/rough-notation`). SmoothUI's `soft-blur-in`, `shimmer-sweep`, `mask-reveal-up`, `line-by-line-slide` and `kinetic-center-build` share names with Remocn elements already ingested; skip them. Hover-, scroll- and pointer-driven components (Magic UI `magic-card`, `lens`, `dock`, `smooth-cursor`, `scroll-progress`, Motion Primitives `magnetic`, `tilt`) have no meaning in a video.

### Fit with Opsis

Already in place:

- `ElementMeta` (`src/shared/elements/types.ts`) already has `source: { registry, url, license }`; registry elements fill it (`registry: "Remocn"`, `url: "https://remocn.dev/docs/components/…"`, `license: "MIT"` in `elements.generated.ts`), core ones use `registry: "Opsis"`.
- `ELEMENTS` (`src/shared/elements/index.ts`) concatenates `coreElementMeta` and `generatedElementMeta`; `elementNode()` (`src/shared/nodes/elements.ts`) makes a node for every entry, with its card, length handle and envelope. A third list is all a port needs on the metadata side.
- Lazy loading: `loadElement`, `useElement` and `isKnownElement` in `src/video/registry/index.ts` find a module through `lazyLoaderOf`, which looks in `generatedElementLoaders` (`components.generated.ts`) and `LAZY_CORE_ELEMENTS` (the 3D elements), and load it under `delayRender`.
- Framing helpers in `src/video/registry/framing.tsx`: `centered`, `fitScreen`, `placedAt` (spot placement for the `spot` widget), `pointsFromSpots`, `painted`, `playsAtEnd`.
- Guardrails already cover any new folder in `src/video`: `no-restricted-imports` bans `framer-motion`, `motion`, `motion/react`, `motion/react-client`, `gsap` and `useFrame` from `@react-three/fiber`, `opsis/no-animate-class` bans Tailwind `animate-*` and `opsis/no-three-clock` bans three.js clocks and animation loops (`oxlint.config.ts`, `scripts/oxlint-plugin-opsis.ts`). Ported folders are not in `ignorePatterns`, so they are linted.
- Fonts: the `font()` param in `core.ts` and `fontStack` turn a family into a CSS stack; `df-main`, `df-heading`, `df-code` follow the project fonts (`src/video/fonts/registry.ts`).
- Placeholder images in `public/placeholders` (`logo-mark.svg`, `screen-wide.svg`, `screen-tall.svg`) for logo and image defaults.

Missing:

- A home for ported code and its loader map.
- Any `THIRD_PARTY_NOTICES.md` entry for Magic UI, Motion Primitives, Cult UI, SmoothUI, lucide-animated or Lucide.
- A written porting recipe (below), so every port is done the same way.

### Work

1. **Where code goes.**
   - Components: `src/video/ported/<source>/<name>.tsx`, with `<source>` one of `magicui`, `motion-primitives`, `cult-ui`, `smoothui`, `lucide-animated`. Each source folder holds its `LICENSE.<source>` file (the upstream licence text, as `src/video/camera/LICENSE.programatic-demo` does), since source files can't carry a licence header comment.
   - Metadata: `src/shared/elements/ported.ts` exporting `portedElementMeta`, written with `defineElementMeta` like `core.ts`. Keys are `<source>/<name>` (node type `fx.<source>.<name>` through `elementNodeType`). `source` is `{ registry: "Magic UI", url: "<the component's docs page>", license: "MIT" }`.
   - `ELEMENTS` becomes `[...coreElementMeta, ...portedElementMeta, ...generatedElementMeta]`.
   - Loaders: `src/video/registry/ported.ts` exporting `portedElementLoaders: Record<string, () => Promise<ElementImpl>>` (dynamic `import()` plus the framing helper, the same shape as `generatedElementLoaders`). `lazyLoaderOf` looks in it too. `registry:ingest` never touches either file.
2. **Porting recipe** (one checklist per port, in the PR description):
   - Copy the upstream file once into the target folder, then rewrite it. Delete `"use client"`, `cn()` imports from the web app and every comment.
   - `motion.div` with `initial` / `animate` / `transition` → a plain element whose style comes from `interpolate(frame, [start, end], [from, to], { extrapolateLeft: "clamp", extrapolateRight: "clamp", easing })` or `spring({ frame, fps, config })`. Keyframe arrays (`rotate: [0, -10, 10, -10, 0]`) → `interpolate` with as many input points, spread over the original `duration` in frames. Motion `ease` names map to `Easing.bezier` with the same control points.
   - `variants` + `useAnimation()` / hover handlers → a `start` (seconds, default 0) and `loop` param; the variant plays from `start`.
   - CSS `@keyframes` / Tailwind `animate-*` → the same property computed from the frame. Only when a keyframe can't be rewritten (a long multi-stop gradient), a paused CSS animation (`animationPlayState: "paused"`, `animationDelay` set to minus the element's time in seconds) is allowed, with keyframe names prefixed `df-` and never `spin` (see **Animation names are global** in `AGENTS.md`); record each such case in `AGENTS.md`.
   - `useEffect` timers, `setInterval`, `requestAnimationFrame` → frame math. Canvas drawing happens in a `useLayoutEffect` keyed on the frame, drawing the whole picture each time (never incrementally).
   - `Math.random()` → `random(seed + "-" + index)` with a `seed` param (min 1).
   - Measuring (`getBoundingClientRect`, `ResizeObserver`, `IntersectionObserver`) → compute layout from params; when text must be measured, measure once after fonts load under `delayRender`, never per frame.
   - Fonts: a `font` param (`font()` from `core.ts`) defaulting to the project font; no `@remotion/google-fonts` or web font imports. Images: `asset` params, defaults from `public/placeholders` as `app:` refs.
   - Sizes in pixels at 1920×1080, scaled by `useVideoConfig()` like the core elements; colours as `zColor()` params.
3. **First batch** (one of each kind, to prove the recipe): Running border (6), Morphing words (1), Flickering grid (14), Animated icon (18) with five icons. Check, then port the rest in two batches.
4. **Copy.** Titles and descriptions in sentence case without source names (the table titles are drafts). `src/shared/nodes/card.test.ts` runs over the new elements automatically.
5. **Licences.** One `THIRD_PARTY_NOTICES.md` section per source, headed `<Source> — src/video/ported/<source>` like the Remocn entry, with the upstream copyright line and licence text, plus Lucide (ISC) for the icon shapes. The Settings → Licences page shows the file.
6. **Tests.** Unit-test the pure timing helpers (keyframe table → value, orbit angle, flicker slot). Add each port to the CI visual check (last section).
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
| Aceternity UI                                        | Its own licence: "You cannot re-distribute the Item as a stock image or its source files, regardless of modifications." Opsis's source is public, so a port would redistribute it.                                          | **no** |
| React Bits (`DavidHDev/react-bits`)                  | MIT + Commons Clause: commercial use allowed "so long as you do not sell, sublicense, or redistribute the components themselves".                                                                                           | **no** |
| Animate UI (`imskyleen/animate-ui`), incl. its icons | MIT + Commons Clause: rights granted only "as part of an application, website, or product", and "so long as you do not sell or redistribute the components themselves in their original form—whether alone or in a bundle". | **no** |

The Commons Clause sources and Aceternity are left out because Opsis's repository is Apache-2.0 and public: ported files would be redistributed source under terms that forbid it. MIT code keeps its own licence inside its folder, which Apache-2.0 allows.

### Risks

- **Ported, not copied.** A port is a rewrite, but it still derives from the upstream file; keep the licence file and notice even when little original code is left.
- **Measuring text.** Morphing letters needs letter positions in two words. Measure with a hidden copy in a `useLayoutEffect` after `FontProvider` has loaded the font (under `delayRender`); measuring before fonts load gives wrong positions only in the render.
- **Paused CSS animations.** They are frame-exact in Chrome only while the element stays mounted; a remount restarts them at the delay. Prefer frame-computed styles; the escape hatch is for rare cases and must pass the scrub-backwards check.
- **Upstream drift.** The ported files don't follow upstream changes (unlike registry elements). Record the upstream commit in the THIRD_PARTY entry so a later port can diff.
- **Canvas size.** Canvas backgrounds must size their backing store from `useVideoConfig()` times the render scale, or scaled-down stills and 4K exports blur.

### Sources

- [Magic UI components](https://magicui.design/docs/components) and [licence](https://github.com/magicuidesign/magicui/blob/main/LICENSE.md); file contents scanned for `motion/react`, `requestAnimationFrame`, `Math.random`, `animate-*`, observers (checked 2026-10-16)
- [Motion Primitives](https://motion-primitives.com) and [licence](https://github.com/ibelick/motion-primitives/blob/main/LICENCE.md) (checked 2026-10-16)
- [Cult UI](https://www.cult-ui.com) and [licence](https://github.com/nolly-studio/cult-ui/blob/main/LICENSE.md) (checked 2026-10-16)
- [SmoothUI](https://smoothui.dev) and [licence](https://github.com/educlopez/smoothui/blob/main/LICENSE) (checked 2026-10-16)
- [lucide-animated](https://lucide-animated.com) and [licence](https://github.com/pqoqubbw/icons/blob/main/LICENSE), 469 icons (checked 2026-10-16)
- [Aceternity UI licence](https://ui.aceternity.com/licence) (checked 2026-10-16)
- [React Bits licence](https://github.com/DavidHDev/react-bits/blob/main/LICENSE.md), [Animate UI licence](https://github.com/imskyleen/animate-ui/blob/main/LICENSE.md) (checked 2026-10-16)

## Liquid contours background

### What it adds

A flowing background of alternating bands in two colours, like the Remotion Elements "Liquid Contours" example. Settings: two colours, band width, shape size, detail, edge softness, seed and how far the bands flow over the element's length.

### Fit with Opsis

Already in place:

- The Remotion Elements page is example source, not a registry: it gives no install command and no package (checked 2026-10-16). The example is a few lines around `liquidContours()` from `@remotion/effects/liquid-contours`, applied to Remotion's `<Solid>` through its `effects` prop, with `phase` interpolated from the frame.
- `@remotion/effects@4.0.536` is a direct dependency (Blur areas and Picture look use it), exports `./liquid-contours`, and its params are `firstColor`, `secondColor`, `spacing` (2–300), `scale` (20–1000), `complexity` (0–1), `smoothness` (0–1), `seed`, `offsetX`, `offsetY`, `phase` (from `dist/liquid-contours.d.ts`). It draws with a WebGL2 shader and fetches nothing.
- `remotion@4.0.536`'s `<Solid>` accepts `effects`.
- Background elements live in `src/video/core/backgrounds.tsx`; keys in `core.ts` and `src/video/registry/index.ts`.

Missing:

- No core element uses Remotion's `<Solid>`; `backgrounds.tsx` exports its own `Solid`, so import Remotion's under another name.

### Work

1. **Element.** `core/liquid-contours` in `core.ts` (`category: "background"`, `defaultDurationInFrames: 150`): `firstColor`, `secondColor`, `spacing`, `scale`, `complexity`, `smoothness` (advanced), `seed` (min 1), `flow` (how much `phase` moves over the element, default 1). Component `LiquidContours` in `backgrounds.tsx` draws `<Solid color={secondColor} effects={[liquidContours({ …, phase: 3.23 + flow * frame / durationInFrames })]} />`, sizes scaled from 1920×1080. Load lazily: put it in `LAZY_CORE_ELEMENTS` (`src/video/registry/index.ts`) rather than the eager `core` map, so projects without it never load the effects package.
2. **Copy.** "Liquid contours", "A flowing background of two-colour bands."
3. **Check.** Export a still and a video with the default renderer and with software drawing (**Draw with**, `src/shared/nodes/output.ts`), 9:16.

### Licence

`@remotion/effects` is under the Remotion licence (npm `UNLICENSED`), covered by the existing Remotion notice. The example on the Remotion Elements page states no licence; Opsis writes its own few lines rather than copying it.

### Risks

- **WebGL2 in software drawing.** If `swangle` can't create a WebGL2 context, the export fails; check, and if so, say so in the **Draw with** help.
- **Banding.** Smooth bands under H.264 band (a known gotcha from the original spec); keep `smoothness` low by default or add a faint grain.

### Sources

- [Remotion Elements](https://www.remotion.dev/elements) and [Liquid Contours](https://www.remotion.dev/elements/backgrounds/liquid-contours/) (checked 2026-10-16)
- Package source of `@remotion/effects@4.0.536` (`dist/liquid-contours.d.ts`, `dist/esm/liquid-contours.mjs`: WebGL2, no network) and `remotion@4.0.536` `<Solid>` (checked 2026-10-19)

## Crossed-off marker

### What it adds

A hand-drawn cross over a word (a title word, an old price) or over a part of the page in a take. The Remotion Elements "Crossed Off" example is `<CrossedOff>` from `@remotion/rough-notation` with `progress` from `interpolate`, `iterations: 10`, `roughness: 2` (checked 2026-10-16).

### Fit with Opsis

- Title card already has it: its `markLook` setting includes "Crossed out" (`src/video/core/title.tsx`, drawn by `HandDrawnMark` in `src/video/core/hand-drawn.tsx`). Don't add a separate element here.
- Close existing elements: Strikethrough Replace (Remocn) and Text Highlight (RemotionUI, includes a strikethrough).
- One gap: Highlight box's looks are Clean, Hand-drawn box, Circle, Underline and Marker (`HIGHLIGHT_LOOKS`, `src/shared/compile/recording.ts`). Add "Crossed out" (`<CrossedOff>`) and "Struck through" (`<StrikeThrough>`) to them, for crossing out part of the page.

### Work

1. Add `"crossed-off"` and `"strike-through"` to `HAND_DRAWN_LOOKS` (`src/shared/compile/recording.ts`), their labels to `HIGHLIGHT_LOOK_LABELS` (`src/shared/nodes/cursor.ts`), and the two components to `HandDrawnMark` (`src/video/core/hand-drawn.tsx`, which already draws `<CrossedOff>` for Title card). Nothing else.

### Licence

MIT (`@remotion/rough-notation`, `roughjs`), already listed in `THIRD_PARTY_NOTICES.md`.

### Risks

See that plan's **Risks** (seed 0 is random, sizes from layout).

### Sources

- [Remotion Elements: Crossed Off](https://www.remotion.dev/elements/text/crossed-off/) (checked 2026-10-16)

## Rive animations

### What it adds

A **Rive animation** element that plays a `.riv` file from the project: pick the file, the artboard and the animation by name, and the fit. Like the Lottie element, the file is copied into the project's assets. It works with no internet: the runtime ships with the app.

### Fit with Opsis

Already in place:

- `LottieElement` (`src/video/core/lottie.tsx`) is the pattern: an `asset` param resolved by `resolveProps` (`src/video/assets.tsx`) to `opsis-file://project/…` in the editor or the worker's asset server in renders, `fetch()` under `delayRender`, `cancelRender` on failure. `core/lottie` in `core.ts` uses `asset("Animation file", ".json,.lottie")`.
- `importAssets` (`src/main/projects.ts`) lists `lottie` among its file types; the asset server's `MIME` table (`src/worker/render/asset-server.ts`) maps `.lottie`.
- Both bundlers already swap one import for Opsis's own module: the font redirect (`GOOGLE_FONT` in `vite.renderer.config.ts`, a Vite plugin's `resolveId`; the same pattern in `src/worker/render/webpack-override.ts`). Rive's runtime uses the same mechanism.

What the packages do (from the tarballs of `@remotion/rive@4.0.536` and `@rive-app/canvas-advanced@2.31.5`, checked 2026-10-19):

- `<RemotionRiveCanvas>` takes `src`, `fit`, `alignment`, `artboard`, `animation`, `onLoad`, `assetLoader` and `enableRiveAssetCdn`. It loads the runtime with `riveCanvas({ locateFile: () => "https://unpkg.com/@rive-app/canvas-advanced@2.31.5/rive.wasm" })`, fixed in the component with no prop to change it.
- The runtime package ships `rive.wasm` itself (1,449,487 bytes, MIT). Its `package.json` has `main: "canvas_advanced.mjs"` and no `exports` field, so `@rive-app/canvas-advanced/canvas_advanced.mjs` can be imported directly.
- `enableRiveAssetCdn` lets the runtime fetch a file's referenced images and fonts from Rive's servers unless it's turned off.
- It advances the animation by `frame - lastFrame` on each frame instead of setting the time.
- The packaged editor's CSP (`src/main/protocol.ts`) is `script-src 'self'` without `'wasm-unsafe-eval'`, so WebAssembly can't compile in the packaged UI. The `MIME` table has no `.wasm` or `.riv`.

### Approach (decided)

Use Remotion's `<RemotionRiveCanvas>` and download nothing at run time: both bundlers send `@rive-app/canvas-advanced` to a small Opsis module that calls the real runtime with a local `locateFile`, so the hard-coded unpkg address is never used.

### Work

1. **Dependencies.** `bun add --exact @remotion/rive@4.0.536`, and `bun add --exact @rive-app/canvas-advanced@2.31.5` because Opsis imports it directly (the version `@remotion/rive` pins).
2. **Bundled runtime.** A script `scripts/rive-runtime.ts` (`bun run rive:runtime`) copies `node_modules/@rive-app/canvas-advanced/rive.wasm` to `public/rive/rive.wasm`, like `fonts:bundle` copies fonts; add the row to the generated-files table in `AGENTS.md`. A test checks the copy matches the installed package byte for byte, so a version bump can't leave an old runtime behind. `rive_fallback.wasm` isn't needed: the advanced runtime never names it.
3. **Runtime redirect.** `src/video/rive/runtime.ts`: `import real from "@rive-app/canvas-advanced/canvas_advanced.mjs"` and export a default function that calls `real({ ...options, locateFile: () => staticFile("rive/rive.wasm") })`, whatever `locateFile` the caller passed. In `vite.renderer.config.ts` and `webpack-override.ts`, resolve the bare `@rive-app/canvas-advanced` to that module, next to the font redirect; the `.mjs` subpath is left alone, so the wrapper doesn't redirect to itself. A test bundles a module that imports the bare name and checks it resolves to the wrapper in both configs.
4. **Element.** `core/rive` in `core.ts`: `src` (`asset("Animation file", ".riv")`), `artboard` and `animation` (text, empty = the default), `fit` ("Show all" / "Fill the frame"), `background`. `defaultDurationInFrames: 90`. Component `src/video/core/rive.tsx`, loaded lazily, renders `<RemotionRiveCanvas src={…} artboard={…} animation={…} fit={…} enableRiveAssetCdn={false} />`, with `src` resolved like Lottie's.
5. **Hosting.** Add `'wasm-unsafe-eval'` to `script-src` in `src/main/protocol.ts` (decided: it allows WebAssembly compilation only, not `eval`); add `.wasm: application/wasm` and `.riv: application/octet-stream` to `MIME`; add `riv` to `importAssets`' file types.
6. **Copy.** Title "Rive animation", description in the Lottie element's style: "Plays an animation made in Rive (.riv)."
7. **Licences.** `THIRD_PARTY_NOTICES.md`: `@rive-app/canvas-advanced` (MIT), including the bundled `rive.wasm`. `@remotion/rive` is under the Remotion licence, covered by the existing notice.
8. **Check.**
   - With networking off: the element loads in the Preview and in an export, and the worker's request log shows no request to unpkg or Rive's servers.
   - Frame stepping: an exported still of the middle frame (a fresh page jumping straight to one frame) matches the same frame in the export; scrubbing backwards and jumping around in the Preview match the export too. If they don't, use the fallback below.

### Fallback: Opsis's own element

If stepping from the previous frame gives wrong pictures, write `src/video/core/rive.tsx` on the runtime directly instead of `<RemotionRiveCanvas>`: load the runtime through the same wrapper, `rive.load(bytes, undefined, false)`, and on each frame set `LinearAnimationInstance.time` from the frame (wrapping for loop), then `apply(1)`, `artboard.advance(0)` and draw. That makes every frame a pure function of the frame number. State machines stay out of scope either way: they need inputs over time.

### Licence

`@rive-app/canvas-advanced` and its `rive.wasm`: MIT (from its `package.json`; the package ships no licence file, so the notice quotes the Rive runtime repository's). `@remotion/rive`: Remotion licence.

### Risks

- **Stepping from the previous frame.** `<RemotionRiveCanvas>` advances by `frame - lastFrame`. Renders are sequential (`concurrency: 1`), but exported stills render one frame of a fresh page and the Preview scrubs, so check both (Work step 8) before shipping.
- **Version drift.** The wrapper, the copied `rive.wasm` and `@remotion/rive`'s pinned runtime must stay the same version; the copy test and the exact pins catch drift on the next Remotion bump.
- **CSP.** `'wasm-unsafe-eval'` is still a change to the editor's security policy; call it out in the change.

### Sources

- [Remotion: @remotion/rive](https://www.remotion.dev/docs/rive/) (checked 2026-10-16)
- [Rive: Preloading WASM](https://rive.app/docs/runtimes/web/preloading-wasm) and [Low-level API usage](https://rive.app/docs/runtimes/web/low-level-api-usage) (`locateFile`, self-hosting the WASM with a matching version; checked 2026-10-17)
- Package source and npm metadata of `@remotion/rive@4.0.536` (`dist/esm/index.mjs`: unpkg `locateFile`, `enableRiveAssetCdn`, `advance(diff / fps)`) and `@rive-app/canvas-advanced@2.31.5` (`package.json`, `rive.wasm`, `rive_advanced.mjs.d.ts`, MIT), unpacked in `~/.cache/opsis-research/rive-check` (checked 2026-10-19)

## Element thumbnails

### What it adds

In the node list, hovering an element shows a picture of it in the hover card, and after a moment the picture starts playing. Works offline and costs nothing at run time.

### Fit with Opsis

Already in place:

- The hover card: `NodePreview` in `src/renderer/editor/palette.tsx`, inside `HoverCard`, shows the title, category, description and ports.
- `ElementMeta.thumbnail?: string` exists in `src/shared/elements/types.ts` and is never read.
- `renderFrame`, `openRenderBrowser` and `getServeUrl` (`src/worker/render/render.ts`) and `bundleRemotionProject` (`src/worker/render/bundle.ts`) render one frame of a spec; `itemClip` and `clipToSpec` (`src/shared/compile/clip.ts`) build a spec for one element.
- `scripts/brand-icons.ts` already finds the recording browser for a dev script (`resources/engine` or `OPSIS_HEADLESS_SHELL`).

Missing:

- No stills exist; `renderFrame` takes `"png" | "jpeg"` only (`renderStill` itself also takes `"webp"`).
- Stills put in `public/` would also be copied into the Remotion bundle (`bundleRemotionProject` passes `publicDir: public`) and shipped twice.

### Work

1. **Script.** `scripts/element-stills.ts` (`bun run elements:stills`, run with `tsx` like the other scripts). For every element in `ELEMENTS`: build a 1920×1080 spec with `itemClip` + `clipToSpec` at default props; overlays and text over a neutral dark `core/solid`; transitions between two `core/placeholder` clips. Render the frame at 60% of `defaultDurationInFrames` (a `STILL_FRAME` override map for exceptions), `gl: "swangle"` so the result doesn't depend on the machine's graphics card, one shared browser, 320×180, WebP quality 75 (add `"webp"` to `renderFrame`'s `format`).
2. **Storage.** `src/renderer/editor/element-stills/<key with "/" as "--">.webp`, renderer-only, loaded with `import.meta.glob("./element-stills/*.webp", { query: "?url", import: "default", eager: true })` in a small `element-stills.ts` next to `palette.tsx`. Add the folder and its command to the generated-files table in `AGENTS.md`; run it after `registry:ingest` and after adding a port.
3. **Budget.** At most 24 KB per still and 3.5 MB in all (about 160 elements × 15–20 KB); the script fails above either. Remove `ElementMeta.thumbnail`, or leave it unused; don't fill it from `src/shared`, which can't use `import.meta.glob`.
4. **Hover card.** `NodePreview` finds the element for `def.type` (reverse of `elementNodeType`) and shows its still at the top. After 600 ms it mounts a muted `<Player>` of `DemoTimeline` with the same spec over the still, looping the element's length, at most one at a time, unmounted when the card closes, and not under reduced motion (`MotionProvider`, `src/renderer/app/motion.tsx`). No animated files are stored: animated WebP at 240 px would be roughly 60–150 KB each, 10–20 MB in all.
5. **Check.** Offline, every element in the list shows a still; the live preview starts and stops with the card; shader elements don't leave WebGL contexts behind after many hovers.

### Licence

Opsis's own renders of its elements; the elements' own notices already cover them.

### Risks

- **Elements that need a file** (Image, Video, Lottie, Rive, Looping video) render empty at defaults; give them a placeholder in the script (`public/placeholders`), or a fixed picture.
- **Live previews of shader elements** may stutter in the hover card; that's acceptable since the still stays underneath.

### Sources

- Remotion `renderStill` image formats in `@remotion/renderer@4.0.536` (`validStillImageFormats`: png, jpeg, pdf, webp; checked 2026-10-19)

## Visual check in CI

### What it adds

On every pull request that touches elements, CI renders one frame of every element and compares it with the stored still, so a registry update, a Remotion bump or a font change that breaks an element fails the build instead of reaching users.

### Fit with Opsis

Already in place:

- `.github/workflows/build.yml` has a `checks` job (typecheck, lint, format, test on `ubuntu-latest`) and a `build` matrix; it already caches the engine for builds.
- `installEngine` (`src/engine/install.ts`) installs the headless shell and FFmpeg into a folder; `enginePaths` (`src/engine/paths.ts`) finds them.
- The thumbnail stills above are rendered deterministically (software drawing, fixed frame) and are the natural reference images.

Missing:

- Any rendering in CI outside `make`, and any image comparison. No image-diff package is installed.

### Work

1. **Compare mode.** `scripts/element-stills.ts --check` renders every element as in the thumbnails section to `~/.cache/opsis-element-stills/` and compares each with its stored still using the engine's FFmpeg `ssim` filter (no new dependency). Fail when SSIM is below a threshold (start at 0.97; measure the spread across a few runs first), when an element has no stored still, or when a still has no element. For each failure, write a side-by-side image (`hstack`: stored | new).
2. **Workflow.** A job `elements` in `build.yml`, `needs: checks`, `ubuntu-latest`, only when the change touches `src/video/**`, `src/shared/elements/**`, `scripts/registry-ingest*`, `public/fonts/**`, `package.json` or `bun.lock` (use a paths check step, since `build.yml` runs on several events). Steps: install, restore `~/.cache/opsis-engine` (key on `src/engine/**` and `bun.lock`), install the engine there if missing, run `bun run elements:stills -- --check`, upload the side-by-side images on failure. Check the workflow with `actionlint`.
3. **Updating references.** A `workflow_dispatch` input `update-stills` runs the script in write mode and uploads the stills as an artifact, so references come from the same machine type as the check. Locally on Linux x64, `bun run elements:stills` writes them directly.
4. **Check.** Break one element on purpose (change a default colour); the job fails with its side-by-side image. Revert; it passes twice in a row.

### Licence

No new packages.

### Risks

- **Time.** About 160 frames with one tab; shader elements take seconds each, so expect several minutes plus the one-time bundle. Only run when relevant paths change.
- **Different machines draw differently.** Software drawing (`swangle`) keeps CI stable; stills written on macOS or Windows may differ slightly, which is why references come from CI or Linux x64.
- **Noise is not a bug.** Elements with grain or dithering still differ a little between runs if anything isn't seeded; that shows up as low SSIM and should be fixed in the element, not by lowering the threshold.

### Sources

- `.github/workflows/build.yml`, `src/engine/install.ts` (read 2026-10-16)
- [FFmpeg filters: ssim](https://ffmpeg.org/ffmpeg-filters.html#ssim) (checked 2026-10-16)

## Rust

How this plan is built if it lands after the move to Tauri (`plans/tauri-rewrite/`). There, element metadata and node definitions are Rust, there is no Remotion Player in the webview, and Remotion renders run in a Bun sidecar.

**Elements stay TS.** Ported components, Liquid contours, the crossed-off looks and the Rive element are video code in `packages/video` and are unchanged by the move. The porting recipe and the frame-purity rules hold as written.

**Metadata moves.** Element metadata becomes Rust data with `schemars` field keywords (`plans/tauri-rewrite/types-and-contracts.md`), so `portedElementMeta` is a module in `opsis-compile` next to the core elements instead of `src/shared/elements/ported.ts`. The loaders (`portedElementLoaders`) stay TS in `packages/video`.

**Rive hosting.** The `'wasm-unsafe-eval'` CSP change moves to the Tauri config in `crates/desktop` (`app.security.csp`). The `.wasm` and `.riv` MIME types go into the asset server in `opsis-render`. `riv` joins the asset import in `opsis-project`.

**Element thumbnails.** The stills are rendered at development time by a script through the sidecar's `render.still` (or `renderFrame` under Bun), as before. The live hover preview can't mount a `<Player>`, because the webview has none. The hover card shows the still only; a streamed preview for a hover card isn't worth a headless shell tab.

**Visual check in CI.** Unchanged: the script renders through the sidecar and compares with FFmpeg's `ssim` filter.

## Decided

- **First batch (decided):** the 18 ports and the four-element first batch as listed.
