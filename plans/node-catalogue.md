# Node catalogue

The node catalogue cut to one node per job: nodes that are one dropdown apart become one node, and nodes that do two jobs give one away. The catalogue goes from 272 nodes to 171. It starts with the rules every node follows from now on, then each area's changes.

Status: plan only. Nothing here is built yet.

## How to read this plan

- Section 1 is the node rules; they go into `agent-docs/nodes.md` and decide every change after them. Section 2 is the one mechanism most merges need. Sections 3 to 10 are the changes by area and can be built in any order once section 2 is in.
- Each section has the same parts where they apply: **In place**, **Work** (numbered), **Decided**, **Risks**. Rules from `AGENTS.md` and `agent-docs/` hold throughout; when they disagree with this plan, they win.
- Node types are written as in the catalogue (`editor-data catalog`): `fx.*` nodes are elements (metadata in `crates/compile/src/elements/<group>/`, components keyed in `packages/video/src/elements/index.ts`), the others are Rust nodes in `crates/compile/src/nodes/`.

## Conventions

- **No old formats.** Removed and renamed nodes are deleted outright, with no migration of saved projects; `examples/` is written again (`bun run examples`) and `crates/project/src/templates.json` is rebuilt from the new nodes in the same change. Local test projects are deleted.
- A new or merged node has a `description`, a `card`, a look on its definition (`.look(…)`, then `bun run glyphs:bundle`) and passes `crates/compile/tests/unit/nodes_c.rs`.
- Any change to the film's types runs `bun run types:export`; any change under `packages/video` runs `bun run film:bundle` before render tests.
- Cache version constants are not bumped; clear test caches instead.
- Copy follows `AGENTS.md` §7: sentence case, durations in seconds, no library or format names.
- Checks for every section: `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, then the app itself (`check-app`).
- Each section updates the domain docs it makes wrong in the same change.

## Summary

| Section                     | What users get                                                                      | Nodes   | Size   |
| --------------------------- | ----------------------------------------------------------------------------------- | ------- | ------ |
| 1. Node rules               | One rule set every node follows                                                     | none    | small  |
| 2. Kind groups              | One node over several elements, its settings switched by Kind                       | none    | medium |
| 3. Transitions              | One Transition node; Sequence only joins clips; a clip carries its own transition   | 27 → 1  | large  |
| 4. Text                     | Animated text, a fuller Text highlight, Rolling number, Arrow                       | 23 → 4  | medium |
| 5. Elements                 | 3D object, Device frame, Background, Logo set, Code, Chart, Metric cards, Film look | 51 → 11 | large  |
| 6. Files into the film      | One way in: Media file → Use as clip                                                | 5 → 1   | small  |
| 7. Takes, camera and frames | Frame for recordings, Camera move, Camera follows the cursor, a plain Screenshot    | 7 → 3   | medium |
| 8. Images                   | Frame for images with window and device kinds; colour fixes through Apply grade     | 5 → 1   | medium |
| 9. Sound and files          | One Sound effect; each sound fix its own node                                       | 8 → 6   | medium |
| 10. Templates               | Data story, Launch anything, Release teaser as starting projects; a Steps timeline  | 3 → 1   | medium |

## 1. Node rules

**Work.**

1. Add these rules to the Rules section of `agent-docs/nodes.md`, each with its reason, and a line in `AGENTS.md` §7 pointing to them:
   1. **A node does one thing: one action on one kind of input.** Its settings tune that thing and never add a second one.
   2. **The same thing on the same kind of input is one node.** Variants are a dropdown (Kind, Style, Type), and each variant shows only its own settings.
   3. **Different kinds of input may each have their own node for the same thing** (Trim and Trim file, Frame for recordings and Frame for images). Nodes are never merged across port types, because conversions are lossless and settings-free only.
   4. **What must happen while recording stays on the recording node or the action.** Drawn into the page during the take, it can't move to a later node: the "As on screen" cursor and quick-mode labels on Record take, the Chapter card, HTML overlay, and "Zoom in on this action" with its zoom group.
   5. **Something that belongs to one clip or one moment rides on it.** A node with the same type in and out adds it (the style chain, the grade chain, a clip's `enter` transition); nodes that join (Sequence, Layers) only join.
   6. **A node is not a film.** Several elements in one are a project template built from nodes.
   7. **Elements don't make sound and don't carry another element's job** (a title inside a device, a mark inside a title); combine them with Layers.
   8. **A file reaches the film one way:** Media file → Use as clip.
   9. **No bundles of fixes.** Each fix is its own node and they chain.
2. A test in `crates/compile/tests/unit/` that fails when two visible nodes have the same input and output port types and the same title stem, to catch a new near-duplicate early. It names the pair; a real exception is listed with its reason.

## 2. Kind groups

Most merges in sections 3 to 5 put several elements under one node whose Kind picks the element. Each element keeps its own component and metadata; only the node over them is shared.

**In place.**

- `element_node` (`crates/compile/src/nodes/elements.rs`) makes one node per element from its `ElementMeta`, and `element_props` builds the element's props from its full defaults with the node's values on top.
- Device frame already draws other elements from its own settings (`camera.deviceFrame`, `crates/compile/src/nodes/camera.rs`; the full-defaults rule in `agent-docs/elements.md`).
- Nested forms open two levels deep and `x-show-if` hides fields by another setting's value (`agent-docs/inspector.md`).

**Work.**

1. **`group_node`** beside `element_node`: a node over a list of `ElementMeta`, with a `kind` choice and one nested object per kind (`params.<kind>.<prop>`) holding that element's settings, shown with `x-show-if` on `kind`. Its compile builds props with `element_props` for the picked kind only.
2. **The Inspector shows the picked kind's object as the node's own fields**, with Basic and Advanced tabs as for any node, rather than as a nested form (`crates/ui/src/inspector.rs`).
3. **Paths into the kind's object** for everything that names a setting: `ParamRef` handles and keys the Timeline edits, card rows (`shown` in `nodes/elements.rs`), the length cases (`packages/video/src/registry/length-cases.generated.json`), and the Local API's node schemas.
4. **Element sounds and lengths** read the picked kind's element (`agent-docs/elements.md`).
5. Docs: a Kind groups part in `agent-docs/nodes.md` and the settings paths in `agent-docs/inspector.md`.

**Decided.**

- **One object per kind**, not one flat set of settings: the elements' props collide (`speed`, `color`, `fontSize` in most of them), and switching Kind and back keeps each kind's values.
- Only the picked kind's object is kept when the project is saved, so a file holds no settings for kinds it doesn't use.

**Risks.**

- Every place that reads a node's settings by a flat key (handles, card rows, the AI and API schemas, the length cases) must learn the kind path; one that doesn't reads nothing and falls back silently. The test in section 1 item 2 doesn't catch this; a test that compiles every group at every kind does.

## 3. Transitions

**In place.**

- Sequence has a Transition dropdown, a length, easing, Keep every clip whole and Different transitions (`cuts`), and builds one `Option<TransitionRef>` per clip from `cuts_for`, with the first always empty (`crates/compile/src/nodes/timeline.rs`). It sends each transition's default settings only (`default_transition_params`).
- The 25 transition nodes (`crates/compile/src/nodes/transitions.rs`, metadata in `crates/compile/src/elements/transition/`) are each a Sequence of two clips with one transition; Light leak cut (`transition.lightLeak`) is another. Spring settle (`fx.spring-settle`, Timeline category) plays clips as scenes that shrink away and land.
- `TransitionRef` (`crates/types/src/timeline.rs`) holds a whole transition: kind, length, params, hold, easing. `sequence_clips` and `resolve_enters` (`crates/compile/src/clip.rs`) place clips from that list.
- `ClipValue` (`crates/types/src/values.rs`) already carries a rider through clip nodes: `recording`, copied by `wrap_clip` and kept by `..clip` builders.
- Timeline Split inside a Sequence writes a "none" cut so the halves join with a plain cut (`crates/compile/src/edits/timeline.rs`).

**Work.**

1. **The `transition` port type** in `crates/types` and `crates/compile/src/ports.rs`, with `CompileValue::Transition(TransitionRef)`. Nothing converts into it.
2. **`ClipValue.enter: Option<TransitionRef>`**: "this clip comes in with this transition". `wrap_clip` copies it like `recording`; every node that builds a new `ClipValue` from a clip input copies it by hand.
3. **The Transition node** (`timeline.transition`), a kind group (section 2) over the 25 transition elements, the light leak cut and spring settle: Kind, length, easing, Keep every clip whole, and the kind's own settings. Input `clip`; outputs `clip` (the input with `enter` set) and `transition` (the `TransitionRef` alone).
4. **Sequence only joins clips.** Remove `transition`, `transitionDuration`, `easing`, `keepWhole` and `cuts`; add an optional `transition` input. Join _i_ uses clip _i_'s `enter`, then the input, then a plain cut. Its output takes its first clip's `enter`, so a nested Sequence passes it up. Card rows show each join's transition and keep the squeezed and covered warnings.
5. **Repeat takes an optional `transition` input** for the seams between repeats; the clip's own `enter` stays on its output.
6. **Layers and Choose by variable.** Layers clears `enter` on its output and warns on a marked layer ("Layers stack clips; this transition is ignored"). Choose by variable passes the picked clip's `enter`.
7. **Warnings.** A marked first clip of the top Sequence, or a marked clip that reaches no Sequence, shows "Nothing comes before this clip" on the Transition node's card.
8. **Split** stops writing a cut entry: the new half has no `enter`.
9. **Remove** the 25 `transition.*` nodes, `transition.lightLeak` and `fx.spring-settle`.
10. **Test**: every node with a clip input and a clip output keeps a marked clip's `enter` (fed a marked clip, its output has the same `enter`), so a new clip node can't drop it.
11. Docs: rewrite Transitions in `agent-docs/film.md` (the `enter` rider, the order at each join) and the Split bullet in `agent-docs/timeline.md`.

**Decided.**

- The transition rides on the clip it brings in, not on a port between clips: the transition stays next to its clip and Sequence stays plain.
- Transition sounds keep timing to every transition through the compiled items, which still get `enter` from `sequence_clips`.

**Risks.**

- A clip node that builds a fresh `ClipValue` and forgets `enter` drops the transition silently; item 10 is the guard.
- Spring settle lands the layers of a Layers scene one after another and has its own backgrounds and loop (`crates/compile/src/elements/scene/spring_settle.rs`); as a transition kind it joins two clips, so its layer stagger has to read the incoming clip's layers, and its backgrounds become Background clips.

## 4. Text

**Work.**

1. **Animated text** (`fx.animated-text`), a kind group over Soft blur in, Typewriter, Staggered fade up, Kinetic center build, Word push, Mask reveal up, Tracking in, RGB glitch text, Matrix decode, Line by line slide, Animated headline, Handwriting, Neon sign and Gradient scale cut text. Kinetic center build and Word push describe the same effect: keep the one that reads better and drop the other kind.
2. **Title card** (`fx.title`): its Entrance lists Animated text's kinds. Remove "Mark a word" and its mark settings (`mark`, `markColor`, `markDrawTime`, `markLook`, `markSeed`, `markThickness`, `markerOpacity`); a mark is Ink underline or Scribble circle in Layers.
3. **Text highlight** (`fx.text-highlight`) takes over Marker highlight, Inline highlight, Strikethrough replace and Shimmer sweep, whose presets it already has (marker, color, strikethrough, shimmer); fold each removed element's extra settings into its preset. Its logo-wipe preset and its logo settings (`logoSrc`, `logoScale`, `spinTurns`) move to Logo reveal (section 5).
4. **Rolling number** (`fx.rolling-number`) takes over Slot machine roll as a reel style that rolls characters as well as digits.
5. **Arrow** (`fx.arrow`), a kind group over Callout and Ink arrow, with a clean or brush look.
6. **Remove** the 14 elements of item 1, the four of item 3, Slot machine roll, Callout and Ink arrow.

## 5. Elements

**Work.**

1. **3D object** (`fx.object-3d`), a kind group over 3D logo, 3D card, 3D text and 3D model (What: logo, image card, text, model). Their shared camera, lens, lighting, motion, end facing, grow in, tilt and zoom settings sit outside the kind objects.
2. **Device frame** (`camera.deviceFrame`, clips) takes over Laptop frame, Phone frame, Device mockup, Device lineup and 3D screen. Device: laptop, phone, tablet, monitor, browser, floating; Angle: flat, tilt, 3D, pull back, lineup. Device mockup's eyebrow, title and subtitle go (rule 7).
3. **Captured video** is removed: Use as clip → Crop → Device frame → Camera move does the same.
4. **Background** (`fx.background`), a kind group over Solid colour, Gradient, Paper, Studio backdrop, Noise, Starburst, Aurora, Particle field, Floating shapes and the seven shader backgrounds.
5. **Logo set** (`fx.logo-set`), a kind group over Logo wall, Logo drift and Logo enter (Layout: grid, drift, stack).
6. **Logo reveal** (`fx.logo-reveal`) becomes a kind group over itself, Logo assemble and Text highlight's logo wipe (Style: outline, orbit, wipe).
7. **Code reveal** takes over Glass code block (Look: plain, glass); **Code diff** (`fx.code-diff-wipe`) takes over Code morph (Style: line wipe, token morph).
8. **Chart** (`fx.chart`), a kind group over Bar chart, Line chart, Funnel, Donut, Before and after bars, Bar chart race and Gauge; each type keeps its own data field.
9. **Metric cards** (`fx.metric-ticker`) takes over Stat card as a ring style for a single metric.
10. **Film look** (`fx.film-look`), a kind group over CRT screen, VHS filter, Halftone print and Camera lens.
11. **Redact** for clips: Mosaic (`fx.mosaic`) becomes `fx.redact`, titled Redact, with Redact take's styles (large pixels, blur, solid colour).
12. **Remove** every element absorbed above.

**Decided.**

- Agent run and Check list stay separate: one shows an assistant's run, the other a handwritten list.
- Polaroid, TV power off, Speed lines, Light leak, Confetti, Radial burst, Lower third, Keystroke, Notification stack and the UI scenes (Signup flow, AI prompt flow, Command menu, Search reveal, Kanban board, File tree, File upload, Commit graph) stay as they are: each shows one thing.

**Risks.**

- Device frame's kinds draw different elements with different inputs (`screenSrc` files on Laptop frame and Phone frame, a clip on the others); every kind takes the clip, so the file-based screens go.

## 6. Files into the film

**Work.**

1. **Use as clip** (`media.toClip`) takes over Image, Video, Looping video and Animation, choosing by the connected file's kind: fit and background for all; Ken Burns for images; loop, speed, trim, volume, mute and dim for video; loop, speed and direction for animations (`x-show-if-file`).
2. **Media file** (`media.import`) loses its `clip` output and the settings that came with it (`background`, `envelope`, `fit`, `stillMs`).
3. **Remove** `fx.image`, `fx.video`, `fx.video-loop` and `fx.lottie`.

**Risks.**

- `x-show-if-file` only works on top-level asset settings (`agent-docs/inspector.md`); Use as clip's file comes through a port, so the file's kind has to reach the Inspector from the connected value (`InspectorContext`).

## 7. Takes, camera and frames

**In place.** A take's frame is drawn inside the camera: the camera zooms inside the window and the title bar shows the address at each frame (`agent-docs/film.md`, Window frames). A take inside Device frame draws as the bare screen (`shot.inDevice`).

**Work.**

1. **Frame for recordings** (`camera.frame`, on the take's style) replaces Window frame, Shot and Backdrop. Kind: window, whole page, device (laptop, phone). Window: window style, title bar, title bar text, what the address shows, font. Shape: camera style, window size, corners, shadow. Background: colour, gradient, studio image, ground colour, blur and dim. Shot's ground colour and Backdrop become one background.
2. **Camera move** (`camera.move`, recording → recording) replaces Zoom here and Pan: a time, an optional start spot, an end spot, a zoom, a hold.
3. **Camera** (`camera.camera`) takes over Follow cursor as "Follows: actions or cursor", with Follow cursor's settings shown for the cursor.
4. **Clip from take** loses its playback speed setting; the take's usual speed (1.25× for browser takes, 1× for terminal takes) stays built in, and Speed changes it.
5. **Screenshot** (`capture.still`) outputs only its image. Remove its `clip` output and its style, title bar, backdrop, window size, cut-out size, shadow, alignment, push-in, length and keyframes; Frame for images (section 8) → Use as clip does the rest, a push-in being a scale keyframe.
6. **Remove** `camera.windowFrame`, `camera.shot`, `camera.backdrop`, `camera.manualZoom`, `camera.pan` and `camera.followCursor`.
7. Docs: Window frames in `agent-docs/film.md`, Screenshots in `agent-docs/capture.md`.

**Decided.**

- Record take keeps its "As on screen" cursor with its settings and its quick-mode action labels, and Chapter keeps its card: all three are drawn into the page while recording (`crates/capture/src/overlays.rs`), so no later node can reach them (rule 4).
- Click, Type, Hover, Choose option and Drag keep "Zoom in on this action" and Zoom group: they tell the automatic Camera about that action, which only the action knows.
- Device frame stays for video and image clips; a take gets a device from Frame for recordings.

**Risks.**

- Screenshot's layouts (`core/still-layout.ts`, mirrored in `crates/capture/src/still_layout.rs`) size the capture to the shot; once the framing moves to Frame for images, the capture still has to take enough pixels for the largest frame it may get (`capture_scale`).

## 8. Images

**Work.**

1. **Frame for images** (`image.frame`) replaces Frame screenshot and Tilt. Kind: plain, window, device (laptop, phone). Angle: flat, tilted (Tilt's lean, turn and perspective). Shape, corners, shadow, padding and background as Frame screenshot has them.
2. **It draws through the film page** for every kind, as a still render with the same `WindowFrame` and device elements the film uses, instead of in `crates/image` (`paint_of`).
3. **Remove** Adjust image, Auto levels and Blur and sharpen: the grade nodes (Colour and light, Auto fix, Texture, Black and white and tint) into Apply grade, which takes image files, do the same. Invert moves to Colour and light.
4. **Remove** `image.tilt` and the Rust frame drawing it no longer uses.
5. Docs: `agent-docs/images.md`.

**Decided.**

- One drawing for frames: an image framed with a window or a device matches the same frame in the film.

**Risks.**

- Framing an image needs the recording browser set up, where Frame screenshot today needs nothing; the node says so when the engine is missing.
- A still render costs a page load per image; batch the images of one run through one page if it is slow.

## 9. Sound and files

**Work.**

1. **Sound effect** (`audio.sfx`) takes over Generated sound, Tones and countdown beeps and Riser to a mark. Source: file (the `media` input) or generated. Sound: whoosh, riser, swell, pop, ding, tap, click, tone, beep, countdown. Timing: at a time, at an action, or ending on a mark (the riser's end hit and peak).
2. **Clean speech splits**: **Remove noise** (`media.removeNoise`: learned or spectral, and the rumble cut) and **Soften "s" sounds** (`media.softenS`); its level step is Even out volume.
3. **Fix clicks and clipping splits** into **Fix clicks** (`media.fixClicks`) and **Fix clipping** (`media.fixClipping`).
4. **Apply grade** loses `codec`, `crf`, `proresProfile` and `soundQuality` and keeps the file's own format as Trim file and the other file nodes do; Convert chooses a format.
5. **Convert** takes over Web-ready as a setting for video files (Export and Convert already make H.264 and H.265 files web-ready, `codec_info` in `crates/media/src/codecs.rs`).
6. **Remove** `audio.generate`, `audio.tones`, `audio.riser`, `media.cleanSpeech`, `media.repair` and `media.faststart`.
7. Docs: `agent-docs/audio.md`, `agent-docs/footage.md` for Apply grade.

**Decided.**

- Music and Fit music to the film, Make GIF and Export video, Grab frame and Export images stay apart: each pair works on different port types (rule 3).

## 10. Templates

**Work.**

1. **Steps timeline** (`fx.steps-timeline`): the timeline of steps Data story draws, as its own element, the one part of Data story no node draws today.
2. **Three templates** in `crates/project/src/templates.json` built from nodes: Data story (Title card, Chart, Metric cards, Steps timeline, Title card, End card in a Sequence), Launch anything and Release teaser (their scenes from existing elements, their music as a Music node).
3. **Remove** `fx.data-story`, `fx.launch-anything` and `fx.release-teaser`, with their components and the photos and marks only they use.

**Risks.**

- Launch anything's seven demos and Release teaser's dimensional mark have no element of their own; a template that can't match a scene with existing elements leaves that scene out rather than keeping a one-off element.
