# Elements

How an element is written, which node draws it, how its settings reach its drawing, how its length and size are worked out, and the 3D elements. Paths are relative to `crates/compile/src/elements/` unless they name another crate. Settings declarations are in [inspector.md](inspector.md), the scene it returns in [scene.md](scene.md), the renderer in [film.md](film.md), prepared files in [assets.md](assets.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                        | Where                                                                                                                                                                                              |
| ------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| One element                    | `<category>/<element>.rs`, one folder per `ElementCategory` (`mod.rs`): its settings struct, `definition()` and its drawing (`draw`, `wrap`, a presentation or `scenes`)                           |
| The builder                    | `entry.rs` → `element`, `meta`, `design`, `exact_design`, `optional_clip`; what an element is: `mod.rs` → `ElementKind`                                                                            |
| Key → drawing                  | `draw.rs` → `element`, `wrapper`, `presentation`, `scenes` (one arm per key), `Cx`; the walker's side `library.rs` → `FilmLibrary`                                                                 |
| Node list order                | `mod.rs` → `elements()`; which elements get a node: `crates/compile/src/nodes/elements.rs` → `nodes`                                                                                               |
| Choices several elements share | `shared.rs`                                                                                                                                                                                        |
| Parts several elements draw    | `parts/` (buttons, inputs, spinners, toasts, hand-drawn marks, line charts, device screens); UI kits `ui_kit.rs`, `warm_theme.rs` → `warm_theme`, `neutral_theme.rs` → `neutral_theme`             |
| Theme preset tables            | `theme.rs` (the `*_themes` tables and `with_themes`)                                                                                                                                               |
| Settings walking and defaults  | `introspect.rs` → `map_leaves`, `frames_from_seconds`, `widget_keys`                                                                                                                               |
| Props built from settings      | `crates/compile/src/nodes/elements.rs` → `element_props`; `crates/compile/src/nodes/image_frame.rs` → `FramedImageProps`; `crates/compile/src/clip.rs` → `PlaceholderProps`                        |
| Lengths                        | `lengths.rs` → `element_timing`, `animation_length`, `fits_its_animation`; the Length field `crates/compile/src/nodes/elements.rs` → `clip_duration_field`; `crates/compile/tests/unit/lengths.rs` |
| 30 fps clock                   | `draw.rs` → `Cx::authored_frame`, `authored_duration`, `real_frames`; `crates/scene/src/clock.rs` → `Clock::authored`                                                                              |
| Design box                     | `crates/scene/src/timeline.rs` → `design_box`                                                                                                                                                      |
| Colour literal allow-list      | `scripts/element-colors.test.ts` (`ALLOWED`, `ELEMENT_FILES`, `DIRS`)                                                                                                                              |
| Element stills                 | `bun run elements:render` → `crates/render/src/bin/render-elements.rs`, specs `crates/render/src/element_spec.rs`, into `~/.cache/opsis-dev/element-stills`                                        |
| Goldens                        | `crates/render/tests/goldens.rs`: every element's middle frame (`fixtures/elements`), chosen elements' scenes (`fixtures/scene/elements`)                                                          |
| 3D                             | `three/mod.rs` → `staged`, `standard`, `textured`, `two_sided`, `unlit`; `three/motion.rs` → `pose_at`, `element_pose`; `crates/stage3d`; `crates/geometry`                                        |
| 3D stills check                | `crates/render/tests/three_stills.rs` against `fixtures/prepare/stills`; `OPSIS_STILLS_OUT` names where it writes its stills (default `~/.cache/opsis-dev/three-stills`)                           |

## Rules

### One shape for every element

- **Every element is one Rust file holding its settings struct, the rows and choices only it uses, `definition()`, which registers it with the one builder (`entry.rs`), and the function that draws it**, returning scene nodes ([scene.md](scene.md)).
- **An element's key is its plain name (`title`, `soft-blur-in`), and an element with a node of its own has the node type `fx.<key>` (`element_node_type`).** No name, type, path or field says where an element came from; only `THIRD_PARTY_NOTICES.md` records that.
- **An element has a node of its own unless another node draws it** (`nodes` in `crates/compile/src/nodes/elements.rs`):
  - an element in a kind group is drawn by the group's node, its Kind picking it ([nodes.md](nodes.md), Kind groups);
  - transitions (`ElementKind::Presentation`) and spring settle (`ElementKind::Sequencer`) are kinds of the Transition node ([film.md](film.md), Transitions);
  - the laptop, phone, 3D screen, pull-back and lineup elements are Device frame's angles (`drawn_elements`, `crates/compile/src/nodes/device_frame.rs`);
  - image, video and animation are drawn by Use as clip (`CLIP_ELEMENTS`, `crates/compile/src/nodes/media.rs`);
  - captions by Show captions and Captions from script.
- **Every element is Opsis's own code under the same rules, checks and formatting as the rest**, whatever it started as.
- **Theme settings reach the preset tables through a `schemars` transform**; a family of elements with a palette of its own has its own table built with `with_themes`.
- **`draw.rs` has one arm per key**, for elements, wrappers, transitions and sequencers; a key with no arm draws "Unknown element" ([scene.md](scene.md)).
- **`placeholder` and `framed-image` are drawn there too with no element entry** (`placeholder.rs`, `framed_image.rs`); their props are `PlaceholderProps` and `FramedImageProps` (Frame for images, [images.md](images.md)).

### Settings to props

- **Every element's drawing reads its settings struct as compile wrote it** (`drawn` in `draw.rs` deserialises the props into it) and does in its own file whatever its drawing needs: its background, centring and placement.
  - Compile turns seconds into frames, fills kind props and font stacks and drops empty labelled values (`element_props`); `hosted` turns `app:` refs into addresses ([assets.md](assets.md)).
- **Nothing else reshapes an element's props.**
- **A prop is a setting, a fixed prop (`.fixed`), a kind prop (`.kind_prop`), a wrapper's output, or left at the drawing's own value** when it is timing or internal.
- **A prop that follows the kind of a file setting is declared with `.kind_prop`** (`media` → `src` on the text mask): compile sets it from `kind_from_path`, so a drawing never guesses a file's kind from its name.
- **A drawing has no defaults of its own, so anything drawing an element for another node builds its props from that element's full defaults with its own values on top** (`meta.default_props`, then `element_props`), never a partial map (Device frame's screen, the light-leak cut).
- **Elements make no sound**: no element has a sound setting; sound comes from the sound nodes, Sound effect timed to the clip ([audio.md](audio.md)).

### Themes

- **On themed elements, a colour the drawing falls back to a theme token for is empty ("Theme") by default** unless its default differs from that token, so the theme switch reaches it.
- **A themed element turns its theme value into its UI kit's colours**:
  - the warm kit takes the mode, with the changed colours on top (`warm_theme`);
  - the neutral kit takes the whole light or dark preset with the changed colours on top (`neutral_theme`);
  - an element with its own palette takes its preset for the mode with the changed colours on top.
- **The film's own element code keeps colour literals only for internal fallbacks a finished film never shows** (`PAGE_BG`/`BLEED_BG`, empty-clip placeholders), shadows and window frames; `scripts/element-colors.test.ts` lists each with its reason (`ALLOWED`) and fails on a new one and on a listed one that is gone. A setting's default (`#[setting(…, default = …)]`) is not a literal it counts.
  - The test reads only the folders in `DIRS` and the files in `ELEMENT_FILES`, so an element file with fallback colours has to be added to that list to be checked, and each of its literals to `ALLOWED` with its reason.

### Time and size

- **Elements whose entry has `.authored_clock` count time on a 30 fps clock**: their drawing reads `Cx::authored_frame` and `authored_duration` instead of the frame and length, and a nested sequence inside them takes `real_frames`, so a 60 fps project plays them at their own speed.
- **Their timing settings counted in frames are stored as seconds with `x-frames-at: 30`**, and compile turns them back into frames: `map_leaves` walks the settings, rows and nested settings included, and `frames_from_seconds` turns each such leaf, a number or a comma list (`settings_in_frames`, `element_props`).
- **An element's Length defaults to 0 when it can work its length out** (`clip_duration_field`).
  - It reads "Fit the animation" when `lengths.rs` knows how its animation ends or it runs on the 30 fps clock (`fits_its_animation`), a wrapper included; any other wrapper reads "Match the clip".
  - At 0, a wrapper with a clip connected takes the clip's length whichever label it shows, and any other element its animation's length (`compile_element`), where `animation_length` keeps the designer's hold after the last animation.
  - An element built with `optional_clip` carries "Usual length" for while no clip is connected (`x-unconnected-zero-label`).
  - An element on the 30 fps clock with no `element_timing` entry keeps its designed length, scaled to the project's frame rate and its speed.
- **A transition's length in its entry is the overlap it is made for, never the length of a demo around it.** The Transition node plays it at Length 0 and its card warns when a Length squeezes it below 40% (`speed_row`, `crates/compile/src/nodes/transitions.rs`); for spring settle it is the pause between one clip leaving and the next landing.
  - For a transition taken from Remocn, that is the `linearTiming` of its docs example (`components/docs/examples/<key>-example.tsx`), not its `config.ts` `durationInFrames`, which is the whole preview: two scenes and the overlap.
- **Elements laid out in fixed pixels carry a design size and draw in a design box scaled to fit the frame** (`design_box`), so other shapes and 4K keep them whole.
  - With `design` the box grows to the frame's shape at that scale; with `exact_design` it keeps exactly the design size, centred.

### Fonts and files

- **An element whose drawing uses a family its settings don't name declares it (`.fonts(&["Caveat"])`)**, and a film that uses the element loads it first (`used_font_names`); a test fails on a declared family that isn't bundled.
- **Elements never load fonts themselves**: the film registers the spec's families and the bundled fallbacks once ([fonts.md](fonts.md), [film.md](film.md)).
- **Element defaults never name a library and never load from the internet**: neutral copy and `app:` placeholders.

### Checking elements

- **After changing elements, run `bun run elements:render`** (optionally `-- <part of a key> <positions>`, each position a share of the length, several separated by commas, 0.5 by default): it compiles every element with its defaults (captions through Captions from script, transitions and spring settle through the Transition node between two Background clips, grouped elements through their group at their kind, Device frame's elements through its angle, wrappers around one Background clip) and draws each frame as a small JPEG (`STILL_WIDTH`; `<key>.jpg`, or `<key>@<position>.jpg` for several positions) through the renderer.
  - Shader effects and 3D need the renderer program and a graphics card or the dev engine's SwiftShader; elements whose default is no file (image, video, animation, 3D model) render empty.
- **Every element's middle frame is a golden picture** (`goldens.rs`); a change that moves one is recorded again with `OPSIS_WRITE_GOLDEN=1` and reviewed ([film.md](film.md), Goldens).
- **Tests check every element's defaults pass its own schema and neither load from the internet nor name a library (`crates/compile/tests/unit/nodes_e.rs`), and that every declared font is bundled (`nodes_c.rs`).**

### 3D elements

- **3D elements return a `Stage3d` node** (`staged`, `three/mod.rs`): meshes, models, textures, lights and the camera, drawn by Filament in the renderer program ([film.md](film.md), Effects and 3D).
- **Every pose is a pure function of the frame** (`pose_at`, `element_pose`); a model's clip time comes from the frame, and a looping clip wraps as three.js does.
- **Geometry is measured in compile** (`opsis-geometry`), so an element places what it draws by its real bounds.
- **3D text, models, SVG logos and textures read prepared files** through `Measure::typeface`, `Measure::vector` and the prepared refs ([assets.md](assets.md)). 3D text picks its face with the same `family_for` as flat text.
- **The 3D screen (Device frame's 3D angle) draws the device in Filament and lays its clip on the projected quad with a `matrix3d`**; with **End facing the camera** its last pose fills the frame exactly, so it cuts pixel for pixel to the same clip.
- **Camera keys are the `camera` setting, active when the motion or move is Keyframes or the element has neither setting** (`active_camera`, `crates/compile/src/nodes/elements.rs`): compile puts them on the item's `camera` (`ItemBase`, `crates/types/src/timeline.rs`) and `flatten_spec` turns them into `camera_keys` marks (`camera_key_marks`) the Timeline moves ([timeline.md](timeline.md)).

## How it works

### Adding an element

1. Write `<category>/<key>.rs` with its settings struct, `definition()` and its drawing, add it to the category's `mod.rs` and to the list in `elements()` (`mod.rs`). When it is a variant of something a kind group already draws, add it to that group as a kind (`crates/compile/src/nodes/groups.rs`) rather than giving it a node ([nodes.md](nodes.md), What a node is).
2. Add its settings to `ElementSettings` (`props.rs`), renamed to its key.
3. Add its arm to `draw.rs` (`element`, `wrapper`, `presentation` or `scenes`).
4. When its animation ends at a known time, add its arm to `element_timing` (`lengths.rs`).
5. When it keeps colour literals, list its file in `ELEMENT_FILES` and each literal with its reason in `ALLOWED` (`scripts/element-colors.test.ts`).
6. Run `bun run elements:render` and look at its still, then record its golden (`OPSIS_WRITE_GOLDEN=1`, `goldens.rs`).

## Deliberate limits

- **The goldens and the 3D stills are `#[ignore]`d**, because they need the dev engine's SwiftShader and the renderer program; they run with `--ignored`.
- **The grain gradient material writes each blob's bound as `min(1.0, length(…))`** (`shader-grain-gradient.mat`), for the grain gradient background and the Grain dissolve, Ripple zoom and Wave wipe transitions that share it (`grain_field`). Paper Shaders' `clamp(0., 1., length(…))` is undefined: graphics cards read it as `min(1., …)` and SwiftShader as 1, where Grain dissolve would draw no blobs.
