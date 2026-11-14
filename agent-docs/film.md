# The renderer and rendering

How a film is drawn and encoded: layout, paint, the shader effects and 3D in the renderer program, the drawing choice, raw frames into FFmpeg, chunks, stills and previews, then transitions between clips, other shapes and window frames. Paths are relative to the repository root. The scene nodes, clock and caches are in [scene.md](scene.md), elements in [elements.md](elements.md), footage in [footage.md](footage.md), asset addresses in [assets.md](assets.md), saved pictures and keys in [runs.md](runs.md), the Skia and Filament builds in [toolchain.md](toolchain.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                       | Where                                                                                                                                                                                                                                                                                                       |
| ----------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| One film, frame by frame      | `crates/film/src/lib.rs` → `Film` (`scene`, `frame`, `scaled_frame`, `with_drawing`, `with_frames`), `FilmMeasure`, `FALLBACK_FAMILIES`, `Drawing`                                                                                                                                                          |
| Layout                        | `crates/layout/src/lib.rs` → `lay_out`; `tree.rs` (Taffy), `inline.rs` (Parley), `computed.rs` (inheritance and units), `transform.rs`, `output.rs` → `Laid`; fonts `fonts.rs` → `Fonts`                                                                                                                    |
| Paint                         | `crates/paint/src/lib.rs` → `paint`, `Painter::collect`, `paint_effect`, `paint_offscreen`, `Slots`; `boxes.rs`, `text.rs`, `gradients.rs`, `svg.rs`, `clips.rs`, `filters.rs`, `canvas.rs`, `color.rs`; `resources.rs` → `Resources`, `Cache`                                                              |
| Skia                          | `crates/skia` (`opsis-skia`): `cpp/*.cc` behind `src/bridge.rs`; surfaces, paints, paths, shaders, image filters, typefaces, pictures, Lottie                                                                                                                                                               |
| Shader effects and 3D         | `crates/effects/materials/*.mat`, `crates/effects/src/child.rs` → `serve`, the program `crates/effects/src/bin/opsis-renderer.rs`; `crates/stage3d/src/lib.rs` → `render`, `model.rs`, `glb.rs`; `crates/geometry`; `crates/filament` (`opsis-filament`)                                                    |
| The renderer program's client | `crates/renderer/src/client.rs` → `Renderer` (`start`, `run`, `stage`), `Driver`, `renderer_program`; `protocol.rs`                                                                                                                                                                                         |
| Drawing choice                | `crates/compile/src/nodes/output.rs` → `Gl`; `crates/render/src/executors.rs` → `drawing_for`, `SOFTWARE_VULKAN`                                                                                                                                                                                            |
| Exports and stills            | `crates/render/src/native.rs` → `NativeFiles` (`resolve`, `bytes`, `size`, `frame`, `stalled`), `render_native`, `render_native_still`, `NATIVE_BASE`, `CHUNK_FRAMES`, `HELD_FRAMES`; `crates/render/src/chunks.rs` → `render_in_order`, `Schedule`; `crates/render/src/frames.rs` → `FilmError`, `FilmRun` |
| Encoding                      | `crates/media/src/codecs.rs` → `film_encode_args`, `colour_tags`; tests `crates/media/tests/{raw_frames,lossless}.rs`                                                                                                                                                                                       |
| Take previews                 | `crates/render/src/preview.rs` → `TakePreviews`                                                                                                                                                                                                                                                             |
| Element stills                | `bun run elements:render` → `crates/render/src/bin/render-elements.rs`, specs `crates/render/src/element_spec.rs`                                                                                                                                                                                           |
| Goldens                       | `crates/render/tests/goldens.rs` (`fixtures/elements`, `fixtures/scene/elements`), `crates/paint/tests/cases.rs` (`fixtures/cases`), `crates/layout/tests/cases.rs` (`fixtures/layout`), the cases themselves `crates/cases`; the picture helper `crates/golden/src/pictures.rs`                            |
| Transitions and joins         | `crates/compile/src/nodes/transitions.rs` → `transition_of`, `joins_for`, `join_clips`, `leak_track`; `crates/compile/src/clip.rs` → `sequence_clips`, `sequence_cuts`; `crates/types/src/values.rs` → `ClipValue.enter`; `crates/compile/src/graph.rs` → `transition_issues`                               |
| Transition warnings           | `crates/compile/src/card_rows.rs` → `cover`, `cover_row`, `squeezed_too_much`, `TRANSITION_WARN_RATIO`; `crates/compile/src/nodes/elements.rs` → `SHORT_ELEMENT_RATIO`; `crates/compile/src/timeline/flatten.rs` → `TransitionMark.covers`                                                                  |
| Other shapes                  | `crates/compile/src/elements/overlay_frame.rs` → `overlay_frame`, `shape_insets`; `crates/compile/src/elements/camera/track.rs` → `narrow_pose`, `frame_for`; `crates/compile/src/elements/camera/recording.rs` → `contain_box`, `narrow_bleed`, `screen_clip`                                              |
| Export sizes, shapes          | `crates/compile/src/nodes/output.rs` → `variant_size`, `variant_canvas`, `CANVAS_SHORT_SIDE`; the canvas picker's shapes `crates/ui/src/project.rs` → `shapes`                                                                                                                                              |
| Keep this layout              | `crates/compile/src/nodes/timeline.rs` → `keep_layout`; the design box `crates/scene/src/timeline.rs` → `design_box`                                                                                                                                                                                        |
| Window frames                 | `crates/compile/src/elements/camera/window.rs` → `window_frame`; `crates/compile/src/elements/camera/recording.rs` → `frame_address`, `terminal_title`                                                                                                                                                      |

## Rules

### The renderer

- **Nothing draws a film in a page.** `Film` walks the spec into a scene ([scene.md](scene.md)), Taffy and Parley lay it out from the same font bytes Skia draws, and Skia paints it on the CPU; the shader effects and 3D run on Filament in the renderer program.
- **Skia is reached only through `opsis-skia` and Filament only through `opsis-filament`**, the two crates allowed `unsafe` besides `opsis-vt` and `opsis-qt`. Filament's wrapper exposes plain C functions ([toolchain.md](toolchain.md)).
- **Fonts come only from font bytes**: the spec's families from `public/` and the project, then the bundled fallbacks in order (`FALLBACK_FAMILIES`: Noto Sans subsets, Noto Sans Symbols 2, DejaVu Sans), never the computer's fonts, so glyphs don't change by machine. Text is greyscale anti-aliased and glyph advances stay fractional.
- **Paint follows CSS stacking order** (`Painter::collect`): negative, zero and positive `z-index` lists, positioned boxes as their own entries, and the properties that make a stacking context; effects run in CSS order (backdrop filter, clip path, opacity and blend, filter, mask).
- **A filtered element sees its content beyond the frame**: it is painted unclipped on a surface grown by the filter's reach, then drawn under its ancestors' clips (`paint_offscreen`), as Chrome feeds a filter the element's whole content.
- **An anti-aliased stroke no wider than a pixel is drawn as its outline** (`Surface::draw_path`, `crates/skia/cpp/surface.cc`), because Skia would draw it as a hairline, lighter than its width; terminal sprites and thin SVG lines keep their weight.
- **3D-transformed content is drawn as vectors through the perspective**, never rastered first and projected.

### Effects and 3D

- **Filament runs only in the renderer program `opsis-renderer`**, one per render worker, started the first time a frame has an effect or a 3D stage (`Renderer::start`); passes and stages go over its standard input and output as a JSON header and the layer pixels (`protocol.rs`), models and the room's map once per child. The app links no Filament.
- **The drawing choice (`Gl`: Graphics card or Software) picks the program's driver** (`drawing_for`): the graphics card through Vulkan or Metal, or the SwiftShader that comes with the recording browser, named in a driver file written next to the run (`VK_DRIVER_FILES`), since choosing the driver inside the running app would need `set_var`.
- **`renderer_program` finds the program beside the app or up to five folders above a test**; the installers place it beside `opsis` and `opsis-ffmpeg`, and `bun start` and the e2e runner build it. A program that can't start leaves the frames without their effects and logs why.
- **An effect paints its children into layers, runs its material and draws the result back** (`paint_effect`); a two-scene transition reads the outgoing scene from its slot ([scene.md](scene.md)).
- **Every hash in the materials is integer (PCG)**, so passes give the same pixels on every driver; grain and Perlin noise differ from the browser's pixel for pixel on purpose.
- **3D is a Filament scene** (`opsis-stage3d`): `standard`, its two-sided copy and the unlit `basic` material, directional lights, the room's prefiltered reflections and irradiance (`bun run room:environment`), four samples and three.js's ACES curve; the geometry is three.js's, ported in `opsis-geometry`. Models load prepared GLB through gltfio and are fitted from their bounds sampled over their clip.

### Output

- **Every file a spec names is checked before the first frame** (`NativeFiles::resolve`): prepared refs are prepared, project paths (`.opsis/cache` takes included) must exist, and a missing file fails the render naming it.
- **Frames render in contiguous chunks on every core** (`render_in_order`, `CHUNK_FRAMES`): each worker keeps its own `Film`, decoders and renderer program across a chunk, and a reorder buffer holds at most `HELD_FRAMES` (or two per worker) ahead of the one encoder.
- **Footage reaches the film as decoded frames** (`NativeFiles::frame`, `Footage::shown`); a decoder that sends nothing for 30 s fails the render (`FootageError::Stalled`, [footage.md](footage.md)).
- **Finished frames go to FFmpeg as raw RGBA** (`film_encode_args`): converted to BT.709 limited range for the YUV codecs and tagged with their colour on every output (`colour_tags`); the lossless formats (MP4 lossless `libx264rgb` at `-qp 0` in `bgr24` with ALAC, MKV lossless FFV1 in `bgr0` with FLAC) keep RGB, and `crates/media/tests/lossless.rs` checks every pixel comes back.
- **Stills, take previews and `elements:render` draw through the same `Film`** (`render_native_still`, `TakePreviews`); a still is PNG, JPEG or WebP through the `image` crate.

### Goldens

- **Picture goldens are recorded and compared exactly on x86 Linux with AVX2**, the app's own Skia path; elsewhere FLIP's mean must stay under 0.05 (`crates/golden/src/pictures.rs`).
- **Elements drawn through Filament may differ in 0.05% of their pixels** (`FILAMENT_SHARE`, `goldens.rs`), since SwiftShader's multisampling moves a few single highlight pixels between runs.
- **Record them with `OPSIS_WRITE_GOLDEN=1`** (`goldens.rs` and `three_stills.rs` with `--ignored`, since they need the dev engine's SwiftShader and the renderer program) and review the pictures.

### Transitions

- **A transition overlaps the clips by its length, capped by what each neighbouring clip can give (`resolve_enters`, `crates/compile/src/clip.rs`), unless Keep every clip whole is on**; the cap is what the squeezed warning reports. With it on (`TransitionRef.hold`), `sequence_clips` holds the outgoing clip's last frame and the incoming clip's first frame for the transition (group `remap`) and delays the incoming sound, so no clip loses screen time.
- **A transition rides on the clip it brings in (`ClipValue.enter`), set by the Transition node (`timeline.transition`)**, so it stays next to its clip and Sequence only joins.
  - The Transition node is a kind over every transition element, the light leak cut and spring settle, with its length, easing and Keep every clip whole (`transition_of`); it outputs the clip with `enter` set and the transition alone on its `transition` port.
  - Every node that makes a clip from a clip keeps `enter`: `wrap_clip` copies it, and a node that builds a new `ClipValue` copies it by hand. Trim drops it when it cuts the start, Layers clears it (its card warns on a marked layer) and Choose by variable passes the picked clip's. `crates/compile/tests/unit/catalogue.rs` → `every_clip_node_keeps_a_marked_clips_transition` feeds a marked clip to every clip-to-clip node.
- **Each join in a Sequence uses the incoming clip's `enter`, then the Sequence's `transition` input, then a plain cut** (`joins_for`); the first clip's `enter` is not a join, so the Sequence's output carries it up to the next Sequence (`join_clips`).
  - Repeat takes the same `transition` input for the seams between repeats.
  - A light leak cut is a plain cut with a leak drawn over it on a track of its own (`leak_track`); spring settle plays the joined clips as scenes that shrink away and land.
  - A marked clip with nothing before it (the first clip of the top Sequence, or a clip that reaches no Sequence) puts "Nothing comes before this clip" on the Transition node (`transition_issues`).
- **Timeline Split inside a Sequence leaves the new half with no `enter`**, so the halves join with a plain cut ([timeline.md](timeline.md)).
- **Cards warn where a clip loses more than its transitions were meant to take:**
  - a transition that covers too much of a clip or is squeezed well below its designed length (`cover`, `cover_row`, `squeezed_too_much`, both at `TRANSITION_WARN_RATIO`);
  - Fit length cutting or speeding a lot;
  - an element much shorter than its usual length (`SHORT_ELEMENT_RATIO`).
- **The Timeline draws every transition the same**: how much of a clip a transition covers is recorded on its flattened mark (`TransitionMark.covers`) but not shown.

### Other shapes

- **Exports size the spec with `variant_size`**, and the project Inspector's canvas picker builds its shapes and sizes from `variant_canvas` at `CANVAS_SHORT_SIDE` (`crates/ui/src/project.rs` → `shapes`), the same source as the catalogue's `shapes`.
- **A recording in a shape narrower than itself keeps its own shape inside the frame (`contain_box`) and the camera zooms harder** (`Tuning.narrow`, `narrow_pose`): it never rests fully zoomed out and aims at the cursor between actions.
- **In a narrower shape the camera plans for that frame** (`Tuning.narrow`): `frame_for` fits each target to the frame's width, centred, up to the boosted maximum, and moves only as far as it must to keep the click in shot.
- **The full-page styles (Proof, Ledger) show the whole page in any other shape (`narrow_bleed`) over a blurred copy of the footage**, with that camera only when narrower. Window styles in wider shapes and the `center`/`fit` choices keep the crop or bars.
- **Overlays size by the long side (`overlay_frame`) and keep clear of `shape_insets`; outside landscape, lower thirds move to the top** so captions own the bottom.
- **A Layers node with Keep this layout in other shapes wraps its layers in a group with `design` (the project's size, `keep_layout`)**, drawn scaled to fit in its design box (`design_box`, [elements.md](elements.md)), for marks placed over a picture.
- **`contain_box` also sizes screenshot windows**; the capture's copy of the sizing is in [capture.md](capture.md).
- **A take inside a Device frame, or framed with Frame's device kind, draws as the bare screen (`screen_clip`, `shot.in_device`)**, with no window frame or reframing; Frame's device kind wraps the clip in the device after the take (`in_device`, `ShotProps.device`).

### Window frames

- **Without a Frame of kind window a take has no frame.** Frame (`camera.frame`, on the take's style) sets the take's `window` (`framed` in `crates/compile/src/nodes/camera.rs`: `window.chrome` on unless its style is None, with its colour, font and title bar), its shot and its background, and `window_frame` draws it.
- **Frame for images draws through the same `window_frame` and device elements** (the `framed-image` element, `crates/compile/src/elements/framed_image.rs`, rendered as a still, [images.md](images.md)), so an image framed with a window or a device matches the same frame in the film.
- **With Title bar off (`window.bar`) the bar shows no text, and any Title bar text is dropped (`Bar::typed`).**
- **With it on, a browser take's title bar shows its first page address unless the frame has Title bar text** (`window.text`, `window.mode`), and a terminal take's shows the window title the take logged (`terminal_title`): "This address" keeps the typed text; "This host with the page's path" puts the typed host before the path on screen at each frame (`frame_address`, from the take's address log, [capture.md](capture.md)).
  - The address mode shows only for browser takes (`x-show-if-take`).

## Deliberate limits

- **HTML inside SVG (`foreignObject`) is not drawn**; the one place the film used it, the terminal under the lens, is an element with a filter instead.
- **No `text-wrap: balance` and no automatic hyphenation**; soft hyphens are drawn with `-`.
- **`vertical-align: top` and `bottom` on text spans align as `baseline`.**
- **Lottie plays without expressions**: a file that uses them draws its keyframed values, and the Media file card warns (`Measured::Expressions`, `crates/probe/src/lottie.rs`).
