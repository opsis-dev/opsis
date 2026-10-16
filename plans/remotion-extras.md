# Remotion extras

Which libraries from the Remotion ecosystem Demoflow should add, what each one gives users, and the work needed to add it. Every library has its own section with the same parts, so the plan can grow one section at a time.

Status: plan only. Nothing here is built yet.

Written by two sessions: demoflow-92 wrote the frame of this file and `@remotion/three`; demoflow-8c wrote the sections for `@remotion/effects`, `@remotion/rough-notation` and `@remotion/mac-cursors`. Later updates add a section per library in the same format.

## How to read this plan

- Each library is one feature. The user picks which to build and when. A large one is split into phases, each checked before the next starts.
- **Status** is one of: `in use` (already imported by Demoflow), `installed indirectly` (in `node_modules` only as another package's dependency), `not installed`.
- Each library section has the same parts: **What it adds**, **Fit with Demoflow**, **Work**, **Licence**, **Risks**, **Sources**.

## Conventions

These apply to every library in this plan. They restate rules from `AGENTS.md`; when the two disagree, `AGENTS.md` wins.

**Versions.**

- Every `remotion` / `@remotion/*` package stays at exact `4.0.535`: `bun add --exact @remotion/<name>@4.0.535`. 4.0.536 is out, but moving to it is a separate step that moves every `@remotion/*` package together; don't do it as part of this plan.
- A package that Demoflow imports directly must be a direct dependency, added with `bun add`, even when it's already in `node_modules` through another package.
- Other packages: the latest release, through `bun add` / `bun add -d`. Never edit dependency blocks by hand. If a peer range conflicts with what the project has (React, TypeScript), stop and report instead of forcing it.

**Where code goes.**

- Never hand-edit `src/video/elements/**`, `src/video/lib/**`, `src/video/hooks/**` (registry-owned) or any generated file.
- Demoflow's own elements: metadata in `src/shared/elements/core.ts` (zod schema with `.meta()`, `kind`, `category`, `defaultDurationInFrames`), component under `src/video/core/` or a new folder in `src/video/`, keyed in `src/video/registry/index.ts`. `elementNode()` (`src/shared/nodes/elements.ts`) then makes the node, its card, length handle and envelope with no extra work.
- Libraries that are large or have side effects load lazily (`import()`), like the registry elements, so projects that don't use them never load them.
- Nothing from `node:*`, `electron`, `playwright-core`, `@remotion/renderer` or `@remotion/bundler` in `src/shared` or `src/video`.

**Behaviour.**

- Every animation in `src/video` is a pure function of `useCurrentFrame()`. No clocks, `requestAnimationFrame`, animation loops or library-driven timers. Randomness uses `random(seed)`. Async loading uses `delayRender` / `continueRender` and `cancelRender` on failure.
- Nothing loads from the internet at preview or render time. Files a library would fetch from a CDN (decoders, fonts, images, sounds) are bundled under `public/` and referenced as `app:` paths, or imported as modules.
- Card stills always draw with `gl: "angle"` (`src/worker/render/card-still.ts`). Exports use it by default, but the export node's advanced **Draw with** setting (`src/shared/nodes/output.ts`) can switch one to a software renderer, so every feature here that draws with WebGL must also be checked with software drawing. Renders use one tab (`concurrency: 1`). Measure before changing the default renderer or the tab count.

**User-facing copy.**

- No library, tool or format names in titles, descriptions, field labels, badges, toasts or errors. Sentence case, durations in seconds. Technical settings go under `advanced`.
- `src/shared/nodes/card.test.ts` checks every node's description and the banned words; it runs for new elements automatically.

**Licences.**

- MIT and Apache-2.0 packages get an entry in `THIRD_PARTY_NOTICES.md` (shown in Settings → Licences).
- Packages under the Remotion licence (npm shows `UNLICENSED` or `Remotion License`) are covered by the existing Remotion licence notice; name them in the section so the user knows.

**Checks when the feature is done.**

- `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`.
- The no-comments check from `AGENTS.md`.
- Export a short clip that uses the new feature and compare it with the editor Preview: preview must equal render.
- Don't commit or push.

## Summary

| Library                    | What users get                                                                                                                                                                         | Licence          | Status               |
| -------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- | -------------------- |
| `@remotion/three`          | 3D elements (logo, card, floating shapes, 3D text), a 3D model element with its animations, recordings on a 3D screen, and 3D camera moves keyframed on the Timeline                   | MIT              | not installed        |
| `@remotion/effects`        | Blurred areas on a recording that stay on the page while the camera moves, with start and end times and areas drawn on any frame of the take; a picture look (vignette, colour, grain) | Remotion licence | installed indirectly |
| `@remotion/rough-notation` | A hand-drawn look for Highlight box (box, circle, underline, marker) and hand-drawn marks on title words                                                                               | MIT              | not installed        |
| `@remotion/mac-cursors`    | The real macOS cursor, turning into a hand over links and a text cursor over fields                                                                                                    | Remotion licence | not installed        |

## @remotion/three

### What it adds

3D scenes drawn with Three.js through React Three Fiber, inside a Remotion composition. The package itself is small:

- `<ThreeCanvas>`: React Three Fiber's `<Canvas>` with Remotion's hooks (`useCurrentFrame()` and the rest) working inside it.
- `<ThreeWebGPUCanvas>`: the same on WebGPU. Not used here (see **Risks**).
- `useVideoTexture()` / `useOffthreadVideoTexture()`: a video as a texture. The second is render-only and one copy of the docs marks it deprecated in favour of `@remotion/media`; neither is needed by the approach below.

Everything else is plain Three.js and React Three Fiber.

For users:

- **3D elements**: a 3D logo (a project SVG extruded, or an image on a card), a 3D card (an image or still tilting in), floating shapes as a background, and 3D text in any project font.
- **3D model**: drop in a 3D model file (a single file, or one with separate texture files), turn it on a turntable or orbit it, light it, and play its built-in animation.
- **3D screen**: a recording (or any clip) on a 3D laptop, monitor, phone or floating screen, with camera moves that end face-on so it cuts cleanly to the flat recording.
- **3D camera keyframes**: every 3D element's camera can follow a preset move or keyframes set in the Inspector and shown on the Timeline.

### Fit with Demoflow

Already in place:

- `gl: "angle"`, which Three.js needs to render headless on the graphics card, is the export default and is always used by the card-still worker.
- One render tab, so several 3D canvases never compete for WebGL contexts in a render.
- Elements already load lazily, take asset refs resolved by `resolveProps` (`src/video/assets.tsx`) and wait with `delayRender`; `LottieElement` (`src/video/core/lottie.tsx`) is the pattern to copy for loading a file.
- Wrapper elements (`kind: "wrapper"`) receive the inner clip as `children`, which the 3D screen uses.
- `fontkit` is already a dependency, used by `src/main/fonts.ts` to read font files, including woff2.
- Envelope keyframes already have a schema, Inspector widget and Timeline display (`src/shared/timeline/envelope.ts`, `src/renderer/editor/timeline/envelope-overlay.tsx`), the model for camera keyframes.
- `noise3D` from `@remotion/noise` (already a dependency, used by the Noise background) gives seeded, frame-driven drift for the floating shapes.

Missing:

- `three`, `@react-three/fiber`, `@remotion/three`, `@types/three` aren't installed.
- Core elements are imported eagerly in `src/video/registry/index.ts`; 3D elements need the lazy path.
- The asset picker (`importAssets`, `src/main/projects.ts`) doesn't list 3D model files and copies one file per asset.
- Three's 3D text reads fonts only in its own format (typeface JSON: the letter outlines as JSON). Nothing turns a project font into it.
- Nothing stops `useFrame` (React Three Fiber's own animation loop) from being used.

Peer ranges checked on npm: `@remotion/three@4.0.535` wants `remotion` exactly `4.0.535`, `three >=0.137`, `@react-three/fiber >=8`. `@react-three/fiber` 9.8.2 (latest) wants `react >=19 <19.4` and `three >=0.156`; the project has React `^19.3.1` and the latest `three` is 0.186.1, so they fit today.

### Work

Build it in five phases. Each phase ends with the checks in item 20 that apply to it and the licence entries in item 19 for the packages it adds. Don't start the next phase until those pass:

- **Phase 1, stage and first elements:** items 1–6 and 17. Check that the Preview matches the render (see **Risks**) on the first element before building the others.
- **Phase 2, 3D text:** items 7–8.
- **Phase 3, 3D models:** items 9–14.
- **Phase 4, 3D screen:** items 15–16.
- **Phase 5, camera keyframes:** item 18.

1. **Dependencies.** `bun add --exact @remotion/three@4.0.535`, `bun add three @react-three/fiber`, `bun add -d @types/three`. Not `@react-three/drei`: its model loader and lighting presets fetch decoders and images from CDNs by default.
2. **Lazy loading for core elements.** Add a map of lazy core loaders in `src/video/registry/index.ts` (key → `() => import("../three/…")`), used by `loadElement` and `useElements` next to `generatedElementLoaders`, and keep `isCore` correct. Check in the Vite and Remotion builds that `three` lands in its own chunk.
3. **Lint.** In `oxlint.config.ts`, ban `useFrame` from `@react-three/fiber` in `src/video` (`no-restricted-imports` with `importNames`). If simple, add a rule to `scripts/oxlint-plugin-demoflow.ts` against `setAnimationLoop`, `THREE.Clock` and `THREE.Timer`.
4. **Stage.** `src/video/three/stage.tsx`, shared by every 3D element: `<ThreeCanvas>` sized from `useVideoConfig()` (width and height are required), transparent background so the element sits over footage, antialiasing on, a perspective camera (fixed until the camera path in item 18 drives it), and three lighting presets built from plain lights (Studio, Soft, Dramatic); no environment images. Any `<Sequence>` inside the canvas uses `layout="none"`.
5. **Motion presets.** `src/video/three/motion.ts`: pure functions `(frame, fps, durationInFrames, params) → { rotation, position, scale }` for Spin, Tilt in, Float, Orbit, Swing, with Remotion's `interpolate`/`spring` and `random(seed)`. Unit tests in `motion.test.ts`: values at chosen frames, and scrubbing backwards gives the same values.
6. **3D elements** in `src/shared/elements/core.ts` (category `scene`), components in `src/video/three/`:
   - **3D logo** (`core/logo-3d`): an SVG from the project extruded (Three's `SVGLoader` + `ExtrudeGeometry`), or a PNG/JPEG on a rounded card when the file isn't an SVG. Params: file (`asset`, accept `.svg,.png,.jpg,.jpeg,.webp`), depth, colour, metalness, motion, lighting.
   - **3D card** (`core/card-3d`): an image or a still on a card with rounded corners and a soft shadow that tilts in and settles. Params: file, corner radius, tilt angle, motion, lighting.
   - **Floating shapes** (`core/shapes-3d`, background): seeded spheres, rings and rounded boxes drifting with `noise3D`. Params: count, palette, seed, speed.
   - Descriptions say what they look like in plain words; no "Three.js", "WebGL", "SVG" in labels ("Logo file", not "SVG file").
7. **Font outlines for 3D text.** Woff2 is fine as a source: it holds every letter's outline, only in a format Three can't read. Convert once, automatically:
   - `src/fonts/outlines.ts`, a new Node-only folder shared by main and the worker like `src/traces` (add it to the layout in `AGENTS.md`): read the font with `fontkit`; for a variable font, take the instance at the asked weight (`font.getVariation`); write Three's typeface JSON (`glyphs` with `o` path commands, `ha`, `resolution`, `boundingBox`, `underlinePosition`, `underlineThickness`, `familyName`) for the font's Latin glyphs.
   - The file is found the way the video finds fonts: bundled families from `src/shared/fonts/bundled.generated.ts` (`public/fonts/bundled`), project fonts from `settings.fonts.families` (`assets/fonts/<id>/`), the roles from `settings.fonts`.
   - Cache in the project: `.demoflow/cache/outlines/<sha256 of font file + weight + italic>.json`, so a changed font file gets new outlines.
   - The editor asks main for the outlines of every 3D text node's font when it compiles (an IPC call like `requestStill`), and the render executor makes any that are missing in the worker before rendering, with the same module. The element loads `project:.demoflow/cache/outlines/<key>.json` inside `delayRender` and fails with "Couldn't prepare this font for 3D text." when it's missing.
   - Unit-test the conversion with a bundled font: "A" has path commands, advances are positive, two weights of a variable font differ.
8. **3D text** (`core/text-3d`): Three's `FontLoader` + `TextGeometry` with the converted outlines. Params: text, font (`font` widget, empty = the heading role), weight, italic, depth, bevel, colour, metalness, alignment, motion, lighting. Long text wraps on spaces at a width param.
9. **3D model element** `core/model-3d` (category `scene`). Params: model (`asset`, accept `.glb,.gltf`), motion (Turntable, Orbit, Swing, Still), animation (which one: name text, empty = first), animation speed, loop, zoom, lighting preset, floor shadow on/off; seed and loader details under `advanced`.
10. **Model import copies its files.** Add `glb` and `gltf` to the file filter in `importAssets` (`src/main/projects.ts`). A model goes into its own folder, `assets/models/<name>/`. For a `.gltf`, read its JSON and copy every relative `buffers[].uri` and `images[].uri` alongside it, keeping their relative paths; `data:` URIs stay inside the file. For a `.glb`, read the JSON chunk and do the same for any external image. Refuse absolute paths, `..` outside the model's folder and any `http(s):` URI. Missing files fail the import with their names ("This model needs these files next to it: …"). Unit-test the URI collection and the refusals.
11. **Loading.** `src/video/three/load-model.ts`: Three's `GLTFLoader` (`three/addons/loaders/GLTFLoader.js`) inside `delayRender`, `cancelRender` on failure, one parse per URL per tab (module-level cache). Relative files load next to the model through the same asset server. Compressed meshes:
    - Draco: `DRACOLoader` with its decoder files copied from `three/examples/jsm/libs/draco/gltf/` into `public/three/draco/` by a new script `scripts/three-decoders.ts` (`bun run three:decoders`), referenced through `staticFile()`. Add the folder and script to the generated-code table in `AGENTS.md`.
    - Meshopt: import `MeshoptDecoder` from `three/addons/libs/meshopt_decoder.module.js`.
    - KTX2 textures: not supported; fail with "This model uses a texture format that isn't supported yet."
12. **Framing.** Measure the model (`Box3`), centre it and scale it so any model fills the frame the same way without tuning; the zoom param scales from there.
13. **Animation.** One `AnimationMixer` per model; on every frame, `mixer.setTime((frame / fps) * speed)` with the chosen clip's loop mode, before the frame is drawn. Never `mixer.update(delta)`. Check that React Three Fiber draws after the mutation (it may need `invalidate()`), by scrubbing backwards in the Preview and by rendering frames twice.
14. **Model card.** `elementCard` covers the params; add a row with the model's file name.
15. **3D screen** `core/screen-3d`, `kind: "wrapper"`: takes a clip (the recording or anything else) and shows it on a device in 3D. Params: device (Laptop, Monitor, Phone, Floating screen), device colour, camera move preset (Push in, Orbit, Tilt reveal, Rise), **End face-on** (on by default: the camera's last pose has the screen facing it and filling the frame), lighting, background. Devices are built from Three's shapes in `src/video/three/devices.tsx` (rounded boxes for body, bezel and stand), not model files, so they need no assets and stay sharp at any size.
16. **The screen is a DOM layer, not a texture.** Render `children` in a `div` laid over the canvas and place it with a CSS `matrix3d` computed every frame from the camera and the screen plane's world matrix (the same projection `drei`'s `<Html transform>` uses; write it in `src/video/three/project.ts`, don't add drei). This keeps Demoflow's whole clip (camera zooms, cursor, captions, overlays) pixel-sharp with no video texture and no second render. Add a subtle glare as a DOM gradient on top. Unit-test `project.ts`: the four screen corners map to the expected pixels for a known camera. A texture through `@remotion/media`'s `onVideoFrame` would only be needed for curved screens or objects in front of the screen; don't build it.
17. **Other shapes.** The canvas follows `useVideoConfig()`, and the camera fits the subject to the frame's aspect; test in 16:9, 9:16, 1:1 and 4:5.
18. **Camera keyframes.** One camera path for every 3D element:
    - Param `camera` on every 3D element: a preset (the element's moves, plus Keyframes) and a list of keys `{ frame, yaw, pitch, distance, height, ease }`, frames local to the item like `envelope`. Schema next to the envelope's in `src/shared/schema/timeline.ts`; add it to `src/shared/nodes/params.ts` as `cameraPath()`.
    - Pure `cameraAt(frame, path)` in `src/video/three/camera-path.ts`: a preset expands to keys, keys interpolate with their ease; End face-on replaces the last key's pose. Unit tests: values at keys, between keys, before the first and after the last, scrubbing backwards.
    - Inspector: a `camera` widget in `zod-form.tsx`: list of keys with sliders, "Add key at playhead", and the preset picker. Add the widget name to the widget list in `AGENTS.md`.
    - Compile onto the item (`TimelineItem.camera`); `flattenSpec` shows the keys as marks on the bar like envelope keys, and dragging a key moves its frame through a handle in `src/shared/timeline/edits.ts`, with `beginGesture`/`endGesture` so a drag is one undo step. Add its round-trip test in `edits.test.ts`.
19. **Licences.** `THIRD_PARTY_NOTICES.md`: three (MIT), @react-three/fiber (MIT), @remotion/three (MIT), Draco decoder (Apache-2.0), meshoptimizer decoder (MIT).
20. **Check.**
    - Each 3D element exported for 3 s at 16:9 and 9:16 matches the Preview. Frames 0, 30 and 60 rendered twice give identical images. One export with **Draw with** set to software still renders correctly.
    - 3D text in a bundled font, a variable weight and a font from the user's library; changing the project's heading font changes the text.
    - Models: static, animated, Draco, Meshopt, and a `.gltf` with separate textures, imported and exported. Samples are small CC0 files kept under `~/.cache/demoflow-three-samples/`, never in the repo.
    - A recording through **3D screen** into a transition to the same recording flat: with End face-on, the cut must not jump. At every shape.
    - Camera keyframes: add, drag on the Timeline, undo in one step, scrub backwards; the export matches the Preview.

### Licence

`@remotion/three`, `three` and `@react-three/fiber` are MIT. The Draco decoder files copied into `public/three/draco/` are Apache-2.0. `fontkit` is already a dependency (MIT).

### Risks

- **Preview may not match render.** The editor draws with Electron's GPU, exports with the headless shell's ANGLE. Antialiasing, colour and tone mapping can differ slightly. Check with the first element before building the rest.
- **WebGL context limit.** Browsers cap live WebGL contexts (about 16). Live card thumbnails (up to six), the Preview and pop-out windows each add one per visible 3D element. Saved card stills avoid it once the engine is set up; React Three Fiber releases the context on unmount, which the agent should confirm.
- **Long renders.** Remotion warns that ANGLE can leak memory and kill a long render. Export a multi-minute film with 3D in several places.
- **React upgrades.** `@react-three/fiber` 9.8.2 allows React below 19.4 only. A React 19.4 upgrade waits for a matching release.
- **First-frame time.** Parsing a large model and compiling shaders happens inside `delayRender` (30 s timeout, then the frame is retried). Very large models may need a longer timeout or a size warning.
- **Model files from outside.** A model's file references are untrusted input. The import refuses paths that leave the model's folder and anything on the internet; never follow them at render time either.
- **Font coverage.** Outlines cover the font's Latin glyphs; letters the font doesn't have are left out of the 3D text. Converting a large font takes a moment once, in the background.
- **No GPU.** On machines or VMs without a usable GPU, ANGLE falls back to software and 3D renders get slow, as they do when **Draw with** is set to software; measure on CI's runners before adding a render test there.
- **WebGPU.** `<ThreeWebGPUCanvas>` exists, but WebGPU in the headless shell isn't something the app relies on anywhere yet; stay on WebGL.

### Sources

- [Remotion: @remotion/three](https://www.remotion.dev/docs/three)
- [Remotion: `<ThreeCanvas>`](https://remotion.dev/docs/three-canvas)
- [Remotion: GPU and the ANGLE renderer](https://www.remotion.dev/docs/gpu)
- [Remotion: delayRender()](https://remotion.dev/docs/delay-render)
- [Remotion: useOffthreadVideoTexture()](https://remotion.dev/docs/use-offthread-video-texture)
- [Remotion: using a video as a Three.js texture](https://www.remotion.dev/docs/videos/as-threejs-texture)
- [three.js: AnimationMixer](https://threejs.org/docs/pages/AnimationMixer.html)
- [three.js: GLTFLoader](https://threejs.org/docs/pages/GLTFLoader.html)
- [three.js: FontLoader](https://threejs.org/docs/pages/FontLoader.html), [TextGeometry](https://threejs.org/docs/pages/TextGeometry.html)
- [fontkit](https://github.com/foliojs/fontkit)
- [HyperFrames three skill (seek-driven Three.js pattern)](https://skills.sh/heygen-com/hyperframes/three)
- npm metadata for `@remotion/three@4.0.535`, `@react-three/fiber@9.8.2`, `three@0.186.1` (licences and peer ranges, checked 2026-10-16)

## @remotion/effects

### What it adds

Pixel effects for Remotion's own picture components (`<Video>` from `@remotion/media`, `<Img>`, `<Solid>`, `<HtmlInCanvas>`, the `@remotion/shapes` components), passed as an `effects` prop. Each effect is a function that returns a descriptor, for example `regionBlur({ topLeft, bottomRight, blurRadius, feather, roundness, disabled })`. Version 4.0.535 exports about 70 of them:

- blurs: `blur`, `regionBlur`, `zoomBlur`, linear and radial progressive blur and pixelate
- colour: `colorCorrection`, `levels`, `lut`, `whiteBalance`, `vibrance`, `duotone`, `tint`
- looks: `vignette`, `noise`, `scanlines`, `halftone`, `paper`, `lightLeak`
- distortions: `barrelDistortion`, `fisheye`, `wave`, `tear`, `cornerPin`, and more

On a `<Video>`, the chain runs on each decoded frame at the file's own resolution, before the frame is drawn. This happens in the editor Player and in renders: `@remotion/media` calls `runEffectChain` in both paths. `regionBlur`, `blur` and `pixelate` are WebGL2 shaders (checked in the package source).

For users:

- **Blur areas**: blurred areas on a recording that stay on the same part of the browser window while the camera zooms and pans, to hide emails, keys and personal details. Each Blur areas node has its own start and end times, and its areas can be drawn on any frame of the take. Areas that need different times go in separate nodes.
- **Picture look**: vignette, colour and grain on a recording or a video or image clip.

### Fit with Demoflow

Already in place:

- `@remotion/effects@4.0.535` is in `node_modules` because `@remotion/light-leaks` depends on it (and `@remotion/web-renderer`, through `@remotion/studio`). Demoflow doesn't import it.
- Recordings play through `<Video>` from `@remotion/media` in `src/video/camera/demo-clip.tsx`, in three places: the window shot, the full-page shot and the narrow-shape shot with its blurred copy behind it. An effect there sits inside the camera, so it zooms and pans with the page.
- The `boxes` widget (`BoxesField` in `src/renderer/editor/zod-form.tsx`) already draws areas over the upstream recording's first frame (`spotBackground`, `src/renderer/editor/inspector.tsx`) and stores them as fractions of that frame. Those are the UV coordinates `regionBlur` takes.
- Clip modifiers change recording props with `patchRecordings` (`src/shared/compile/clip.ts`), as `cursor.style` and the annotation nodes do (`src/shared/nodes/cursor.ts`).

Missing:

- The existing **Mosaic** element (`core/mosaic`, `src/video/core/mosaic.tsx`, built on remocn's `PixelateRegion`) wraps the finished frame. Its areas are drawn over the raw recording but applied to the composed frame, so they only line up while the camera is fully out and the recording fills the frame. Its help text says the areas don't follow the camera. A blur on the `<Video>` follows the camera, the window frame and other shapes with no extra work. Mosaic stays as it is.
- `@remotion/effects` isn't a direct dependency.
- The `boxes` widget always shows the take's first frame (`spotBackground` passes `timeMs: 0`).

### Work

1. **Dependency.** `bun add --exact @remotion/effects@4.0.535`.
2. **Spec.** In `src/shared/compile/recording.ts`, add `blurAreas?: BlurArea[]` and `look?: PictureLook` to `RecordingProps`.
   - `BlurArea = { x, y, width, height, strength, softness, roundness, fromMs?, untilMs? }`. Position and size are fractions of the recording frame, as the `boxes` widget stores them. `fromMs` / `untilMs` are on the take's clock, like annotations; missing means the whole clip.
   - `PictureLook = { vignette, brightness, contrast, saturation, warmth, grain }`, all optional.
3. **Pure helpers.** `src/video/camera/lib/blur-areas.ts`:
   - `blurEffects(areas, timeMs, fileWidth)` turns areas into `regionBlur` descriptors (`topLeft: [x, y]`, `bottomRight: [x + width, y + height]`). It turns `strength` and `softness`, stored as fractions of the frame width, into `blurRadius` and `feather` in pixels of the file being played.
   - The strength eases in over 0.2 s after `fromMs` and out over 0.2 s before `untilMs`; outside the times the area is `disabled`.
   - Test it in `blur-areas.test.ts`: corners, clamping to the frame, the same blur relative to frame width for a 960 px file and a full-size file, strength at the start, middle and end of the ease, and disabled outside the times.
4. **Frame slider for the `boxes` widget.** Add a time slider under the picture in `BoxesField`, over the take's length. `spotBackground` takes the chosen `timeMs` instead of 0. Store the time in the node as a hidden param so reopening shows the same frame. Mosaic uses the same widget and gets the slider too.
5. **Blur areas node.** `effect.blurAreas`, titled "Blur areas", category `Effects`, icon `eye-off`, in `src/shared/nodes/camera.ts` next to the other recording modifiers (`motionBlurNode`). Register it in `src/shared/nodes/registry.ts`.
   - Description: "Blurs parts of the browser window to hide private details. The blur stays on the page as the camera moves."
   - Params: `areas` (`boxes` widget, at most 8, same shape and help style as Mosaic's), `strength` (slider), `softness` (edge), `roundness`, `from` and `until` (`unit: "ms"`, shown in seconds, empty = the whole clip); anything else under `advanced`.
   - Card: the number of areas, the strength, and the times when set.
   - Compile: `passthrough` + `patchRecordings(clip, (p) => ({ ...p, blurAreas: [...(p.blurAreas ?? []), ...areas] }))`, with the node's `from` and `until` copied onto each area as `fromMs` / `untilMs`, so two Blur areas nodes add up and keep their own times.
6. **Picture look node.** `effect.look`, titled "Picture look", for recordings and for `core/video` and `core/image` clips. Choices: vignette; colour (brightness, contrast, saturation, warmth) through `colorCorrection` / `whiteBalance`; grain through `noise`.
   - Recordings: sets `look` on `RecordingProps`, drawn with the blur areas.
   - Video and image clips: an `effects` prop built in their components.
   - Offer a few looks with plain names. Never expose the catalogue as it is: names like "LUT" or "thermal vision" don't belong in the UI.
7. **Drawing.** In `demo-clip.tsx`, build the effects from the current frame's time with `useMemo` and pass them to all three `<Video>`s, including the blurred copy in the narrow-shape shot, so nothing private shows through it. Blur areas come before the look in the chain. The props arrive through `RecordingClip` (`src/video/core/recording-clip.tsx`). Fast preview plays the 960 px `recording.preview.mp4` and exports play the full-size take. Pick `fileWidth` from `useQuality()`, or the preview blurs harder than the export.
8. **Mosaic copy.** Add a sentence to Mosaic's `help` in `src/shared/elements/core.ts` pointing to Blur areas for things that must stay hidden while the camera moves.
9. **Check.** A take with Auto zoom and an email field under Blur areas. The blur must stay on the field through zoom in and out:
   - in the Preview, Fast and Exact
   - in an export
   - at 16:9 and 9:16
   - with motion blur on and off
   - with start and end times: it eases in and out at the right moments
   - with an area drawn on a later frame of the take
   - with **Draw with** set to software: the blur still shows, or the export fails (see **Risks**)

   Compare export time with and without the blur (see **Risks**). Check each picture look on a recording, a video clip and an image clip.

### Licence

Remotion licence (npm shows `UNLICENSED`). The existing Remotion licence notice covers it, so it needs no `THIRD_PARTY_NOTICES.md` entry.

### Risks

- **A blur that fails must fail the export.** If the chain can't get a WebGL2 context, which can happen with a software renderer, it throws (`createWebGL2ContextError` in `remotion`). Never catch that and draw the frame without the blur: the export would show what the user asked to hide. Let the render fail and give the error plain wording.
- **Motion blur multiplies the work.** `CameraMotionBlur` draws the window 8 times per frame (`samples={8}`). Each copy is a `<Video>` with its own effect chain and canvas pool, so a frame runs the blur 8 times and holds more WebGL contexts. Measure export time and watch for context loss with Auto zoom on. Report before changing the sample count, since that changes the look.
- **Preview and render take different paths.** The Player redraws through `@remotion/media`'s `MediaPlayer`, while renders extract the frame and then run the chain. Compare exported frames with the Preview.
- **It hides a place in the window, not a page element.** Content that scrolls moves out from under the blur. Following a page element is out of scope.

### Sources

- [Remotion: @remotion/effects](https://www.remotion.dev/docs/effects)
- [Remotion: @remotion/effects API](https://www.remotion.dev/docs/effects/api)
- [Remotion: regionBlur()](https://www.remotion.dev/docs/effects/region-blur)
- [Remotion: `<Video>` from @remotion/media](https://www.remotion.dev/docs/media/video)
- Package source and npm metadata of `@remotion/effects@4.0.535`, `@remotion/media@4.0.535` and `remotion@4.0.535` (exports, WebGL2 use, effect chain; checked 2026-10-16)

## @remotion/rough-notation

### What it adds

Hand-drawn marks drawn with Rough.js around any element: `<Box>`, `<Circle>`, `<Underline>`, `<Highlight>` (marker), `<StrikeThrough>`, `<CrossedOff>`, `<Bracket>`. Each one wraps its children and draws an SVG over them.

- Shared props: `progress` (0–1, how much of the stroke is drawn), `seed`, `color`, `strokeWidth`, `roughness`, `bowing`, `padding`, and the `<Sequence>` timing props.
- It measures the wrapped element with `offsetLeft` / `offsetWidth` (a `ResizeObserver` keeps it current). It waits with `delayRender` until it has a size.
- It redraws on every frame from `progress` and `seed`, so the drawing is a pure function of the frame when Demoflow drives `progress` from `useCurrentFrame()`.

For users:

- **Hand-drawn Highlight box**: the existing Highlight box node gets a look: Clean (today's box), Hand-drawn box, Circle, Underline or Marker. The mark draws itself in when the action starts.
- **Marked words in titles**: circle, underline, box, marker or crossed out on a chosen word of a Title.

### Fit with Demoflow

Already in place:

- `cursor.highlightBox` (`src/shared/nodes/cursor.ts`, built by `annotationNode`) is the right home. `resolveAnnotations` (`src/video/camera/action-log.ts`) turns each spec into rects with `fromS` / `toS` on the take's clock. `Annotations` (`src/video/camera/annotations.tsx`) draws them inside the camera, so they follow zooms. It already counter-scales strokes by `1 / cameraScale` and the design scale `k`.
- `@remotion/paths`, its other Remotion dependency, is already a direct dependency.

Missing:

- `@remotion/rough-notation` and `roughjs` aren't installed. `roughjs` comes in as its dependency; Demoflow never imports it directly.
- `annotationNode` gives every annotation node the same params, so the look needs a way to add params to one node.

### Work

1. **Dependency.** `bun add --exact @remotion/rough-notation@4.0.535`.
2. **Spec.** Add `look?: "clean" | "box" | "circle" | "underline" | "marker"` to `AnnotationSpec` (`src/shared/compile/recording.ts`) and to `Annotation` (`annotations.tsx`), and pass it through `resolveAnnotations`. A missing value means `clean`, so saved projects look the same.
3. **Highlight box node.** Let `annotationNode` take extra params for one node, and give `cursor.highlightBox` a `look` choice: "Clean", "Hand-drawn box", "Circle", "Underline", "Marker". Add a `sketchiness` slider under `advanced`. The card adds the look to its row.
4. **Drawing on the take.** In `annotations.tsx`, the `highlight` kind with a hand-drawn look renders an absolutely positioned block at the padded rect and wraps it in the matching component, with these props:
   - `progress`: an eased 0→1 over the first 0.45 s after `fromS`, computed from `timeS`; the existing envelope fades it out
   - `color`: the annotation's colour
   - `strokeWidth`: `3 * k * counter`, as the clean box uses
   - `seed`: `index + 1`
   - `iterations`: 2

   For Marker, set `mixBlendMode: "multiply"` and a low opacity so the page under it stays readable.

5. **Marked words in titles.** `core/title` (`src/shared/elements/core.ts`, component under `src/video/core/`): params `mark` (text to find in the title, empty = none), `markLook` (Underline, Circle, Box, Marker, Crossed out) and `markColor`. Wrap the matching word in a `<span>` inside the component's text, so measuring stays within Demoflow's own markup. Wrap that span in the mark, with `progress` starting after the title's entrance.
6. **Tests.** Unit-test the progress curve (pure, frame in, value out). Keep `card.test.ts` passing.
7. **Licences.** `THIRD_PARTY_NOTICES.md`: `@remotion/rough-notation` (MIT) and `roughjs` (MIT).
8. **Check.**
   - A take with Auto zoom and a hand-drawn Highlight box, in each of the four looks: the mark lands on the element at every zoom.
   - Titles with a marked word, including long titles that wrap onto two lines and the project's heading font: the mark follows the word after the font loads, since it measures after `delayRender`.
   - Scrubbing backwards in the Preview redraws the same strokes.
   - Rendering the same frame twice gives identical images.
   - The export matches the Preview.

### Licence

MIT (`@remotion/rough-notation`, `roughjs`). Both get entries in `THIRD_PARTY_NOTICES.md`.

### Risks

- **Seed 0 is random.** Rough.js's generator falls back to `Math.random()` when the seed is falsy (`if (this.seed)` in its `Random` class), and the package passes `seed ?? 1`. Never pass 0: computed seeds start at 1, and any seed param has `.min(1)`.
- **Size comes from layout, not transforms.** `offsetWidth` ignores CSS transforms. Give the wrapped block its real size in layout and never size it with `scale()`, or the mark draws at the unscaled size.
- **Strokes inside the camera.** The SVG zooms with the camera like everything in `Annotations`. Keep the counter-scaled stroke width or the line gets thick when zoomed in.

### Sources

- [Remotion: @remotion/rough-notation API](https://www.remotion.dev/docs/rough-notation/api)
- [Remotion: `<Highlight>`](https://www.remotion.dev/docs/rough-notation/highlight), [`<Box>`](https://www.remotion.dev/docs/rough-notation/box), [`<Circle>`](https://www.remotion.dev/docs/rough-notation/circle), [`<Underline>`](https://www.remotion.dev/docs/rough-notation/underline)
- [Remotion: Text highlights](https://www.remotion.dev/docs/text-highlights)
- Package source and npm metadata of `@remotion/rough-notation@4.0.535` (measuring, `delayRender`, seed handling, `roughjs@4.6.6`; checked 2026-10-16)

## @remotion/mac-cursors

### What it adds

macOS cursor artwork for Remotion, available since 4.0.513:

- `<MacOSCursor cursor="pointer" />`: draws one cursor.
- `resolveCursor(cssValue)`: returns `{ src, hotspot, width, height }`, or `null` for `none`.
- `macOSCursorNames`: every CSS cursor value it knows.

It maps CSS cursor values to 40 SVGs: arrow, pointing hand, open and grabbing hand, text cursor (horizontal and vertical), busy, not allowed, copy, help, zoom in and out, the resize arrows, and others. Each one is 32×32 with its own hotspot. They're inline `data:image/svg+xml` URIs in one 72 KB module, so nothing loads from the internet. It depends only on `remotion`.

For users: a **Mac** look for the drawn cursor, chosen on the Cursor node. The cursor turns into a pointing hand over links and buttons and a text cursor over fields, as it would on a Mac. "Simple arrow" stays the default everywhere.

### Fit with Demoflow

Already in place:

- The cursor is drawn in the video, not filmed: in `synthetic` mode (`src/worker/capture/driver.ts`), capture hides the page's own cursor and records a track. `Cursor` (`src/video/camera/cursor.tsx`) draws one SVG arrow on that track with the click ripple, scaled by the design scale and the camera (`CURSOR_SCALE_EXP`).
- `cursor.style` (`src/shared/nodes/cursor.ts`) already sets visibility and size through `patchRecordings`; the look is one more param.

Missing:

- `CursorSample` (`src/video/camera/lib/click-log.ts`) is `{ t, x, y }`: capture doesn't record which cursor the page would show.
- The injected style in `src/worker/capture/recorder.ts` sets `* { cursor: none !important; }`, so `getComputedStyle(el).cursor` reads `none` everywhere while it's active.

### Work

1. **Sample.** Add `c?: string` (a CSS cursor keyword) to `CursorSample`. Old takes have none and draw the arrow.
2. **Read it in the page.** Add an in-page helper next to the cursor mount script in `recorder.ts`. For a point it:
   - finds `document.elementFromPoint`
   - switches the injected `cursor: none` style off for the read (`style.disabled = true`, read, restore), so the page's own cursor rules apply
   - reads `getComputedStyle(el).cursor`
   - keeps only the last keyword of the value, dropping any `url(...)` part (see **Risks**)
   - turns `auto` into `text` for text inputs, `textarea` and `contenteditable`, and `default` otherwise

   Check which coordinates `mark(x, y)` uses (it works with `captureScaleOf(page)`) and convert to CSS pixels for `elementFromPoint`.

3. **When to read.** Read once when a glide settles on its target and at each click or hover action, then carry the value forward. A glide starts with the arrow (`default`) and takes the read value when it settles, so a hand doesn't stay on all the way across the page after the cursor leaves a link. Shapes the cursor would show while passing over things mid-glide aren't recorded; that's accepted. Don't read on every travel sample: one page round trip per sample would slow the glide. Capture timing comes from frames, so make sure the extra reads don't change pacing noticeably.
4. **Cache.** Bump `ENGINE_VERSION` in `src/worker/engine/cache.ts`: the Record executor's output changes. Takes are recorded again on their next run.
5. **Dependency.** `bun add --exact @remotion/mac-cursors@4.0.535`. A static import is fine: 72 KB and no side effects.
6. **Spec and node.** Add `cursorLook?: "arrow" | "mac"` to `RecordingProps` (`src/shared/compile/recording.ts`), defaulting to `arrow`. Give `cursor.style` a `look` choice, "Simple arrow" (default) or "Mac", and show it on the card row.
7. **Drawing.** In `cursor.tsx`, for `mac`, draw `resolveCursor(state.c ?? "default")` with `<Img>`. Size it 32 design px × `k` × the existing counter-scale, and offset it by the hotspot so the click point stays where the arrow tip was. The shape changes at the sample's time, without a crossfade. `null` (`none`) hides the cursor. The ripple stays centred on the hotspot.
8. **Tests.** Unit-test the keyword clean-up (`"url(a.png) 4 4, pointer"` → `pointer`, `auto` on an input → `text`). Extend `src/video/camera/lib/cursor.test.ts` so `cursorAt` returns the latest sample's cursor.
9. **Check.** A take that moves over a link, a button and a text field: hand, hand, text cursor, back to the arrow. The ripple sits on the click point. Check at camera zoom and in a narrow shape, and compare the export with the Preview.

### Licence

npm shows `Remotion License`; the existing Remotion licence notice covers it, so it needs no `THIRD_PARTY_NOTICES.md` entry. The package doesn't say where its cursor artwork comes from (see **Open questions**).

### Risks

- **Custom cursors would load from the internet.** `resolveCursor` accepts `url(...)` values and returns that URL as the image. A page's custom cursor would then load from its site at preview and render time. Always pass only the keyword, never the raw CSS value.
- **Baked mode isn't covered.** In `baked` mode the arrow is part of the filmed page (`#pw-cursor`), so the look only applies to `synthetic` takes. The node card should say so when the take is baked.
- **Iframes and canvases.** `elementFromPoint` stops at an iframe, and canvas apps set cursors in script. Both read as the iframe's or canvas's own cursor, usually the arrow. That's acceptable; don't follow into frames in this plan.

### Sources

- [Remotion: @remotion/mac-cursors](https://www.remotion.dev/docs/mac-cursors)
- Package source and npm metadata of `@remotion/mac-cursors@4.0.535` (cursor list, hotspots, inline SVGs, `resolveCursor`'s `url()` handling; checked 2026-10-16)

## Decided

- **Cursor artwork (decided): ship it,** relying on the package's Remotion licence. Checked 2026-10-17: the package (`@remotion/mac-cursors@4.0.535`, 39 cursors embedded as SVG data) credits no source, and the PR that added it ([remotion-dev/remotion#10264](https://github.com/remotion-dev/remotion/pull/10264)) doesn't say where the artwork came from. The file names match the folder names of macOS's own system cursors, so they are redrawings of Apple's cursor designs; they aren't copies of [cursor.in](https://github.com/sawyerh/cursor.in) (no licence; 7 files compared, all differ) or the GPL-3.0 `apple_cursor` set. Revisit if Remotion documents a source or licence for them.
