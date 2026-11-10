# Elements

How an element is written in Rust and on the page, which node draws it, how its settings reach its component, how its length and size are worked out, and the 3D elements. Rust paths are relative to `crates/compile/src/elements/`, page paths to `packages/video/src/`. Settings declarations are in [inspector.md](inspector.md), the film page in [film.md](film.md), prepared files in [assets.md](assets.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                        | Where                                                                                                                                                                                                                                                               |
| ------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| One element's definition       | `<category>/<element>.rs`, one folder per `ElementCategory` (`mod.rs`)                                                                                                                                                                                              |
| The builder                    | `entry.rs` → `element`, `meta`, `design`, `exact_design`, `optional_clip`; what an element is: `mod.rs` → `ElementKind`                                                                                                                                             |
| Node list order                | `mod.rs` → `elements()`; which elements get a node: `crates/compile/src/nodes/elements.rs` → `nodes`                                                                                                                                                                |
| Choices several elements share | `shared.rs`                                                                                                                                                                                                                                                         |
| Theme preset tables            | `theme.rs` (the `*_themes` tables and `with_themes`)                                                                                                                                                                                                                |
| Settings walking and defaults  | `introspect.rs` → `map_leaves`, `frames_from_seconds`, `widget_keys`; `default_props.rs` and `length_cases.rs` write the generated JSON (`bun run types:export`)                                                                                                    |
| Props built from settings      | `crates/compile/src/nodes/elements.rs` → `element_props`; `crates/compile/src/nodes/image_frame.rs` → `FramedImageProps`; `crates/compile/src/clip.rs` → `PlaceholderProps`; the page's settings types from `props.rs` → `ElementSettings` (`bun run types:export`) |
| Lengths                        | `lengths.rs` → `element_timing`, `animation_length`, `fits_its_animation`; the Length field `crates/compile/src/nodes/elements.rs` → `clip_duration_field`; checked by `registry/lengths.test.ts` through `length-cases.generated.json`                             |
| Components                     | `elements/<category>/<key>.tsx` or `elements/<category>/<key>/index.tsx`                                                                                                                                                                                            |
| Key → implementation           | `elements/index.ts` → `ELEMENTS`; lookup `registry/index.ts` → `getComponent`, `getPresentation`, `getSequencer` (`resolveElement` wraps a lookup's result)                                                                                                         |
| What an implementation is      | `registry/framing.tsx` (`component`, `presentation`, `sequencer`)                                                                                                                                                                                                   |
| Settings shape helpers         | `lib/settings/shape.ts` → `nestedFrom`, `listsFrom`, `ticksFromWords`, `treeFromPaths`                                                                                                                                                                              |
| UI kits                        | `lib/neutral-ui`, `lib/warm-ui` (`theme.ts`, `fonts.ts`, `item.tsx`), `lib/ui-kit` (colour, motion, step and timeline helpers)                                                                                                                                      |
| 30 fps clock                   | `core/authored-clock.ts` → `useAuthoredFrame`, `useAuthoredConfig`, `useRealFrames`                                                                                                                                                                                 |
| Design box                     | `demo-timeline.tsx` → `DesignBox`                                                                                                                                                                                                                                   |
| Colour literal allow-list      | `colors.test.ts` (`ALLOWED`, `ELEMENT_FILES`, `DIRS`)                                                                                                                                                                                                               |
| Element stills                 | `bun run elements:render` → `crates/render/src/bin/render-elements.rs`, into `~/.cache/opsis-dev/element-stills`                                                                                                                                                    |
| 3D stage and loaders           | `three/` → `Stage3D`, `useDrawnScene`; `three/project.ts`                                                                                                                                                                                                           |
| 3D stills check                | `crates/render/tests/three_stills.rs` against `fixtures/prepare/stills`; `OPSIS_STILLS_OUT` names where it writes its stills (default `~/.cache/opsis-dev/three-stills`)                                                                                            |

## Rules

### One shape for every element

- **Every element is one Rust file holding its settings struct, the rows and choices only it uses, and `definition()`**, which registers it with the one builder (`entry.rs`).
- **An element's key is its plain name (`title`, `soft-blur-in`), and an element with a node of its own has the node type `fx.<key>` (`element_node_type`).** No name, type, path or field says where an element came from; only `THIRD_PARTY_NOTICES.md` records that.
- **An element has a node of its own unless another node draws it** (`nodes` in `crates/compile/src/nodes/elements.rs`):
  - an element in a kind group is drawn by the group's node, its Kind picking it ([nodes.md](nodes.md), Kind groups);
  - transitions (`ElementKind::Presentation`) and spring settle (`ElementKind::Sequencer`) are kinds of the Transition node ([film.md](film.md), Transitions);
  - the laptop, phone, 3D screen, pull-back and lineup elements are Device frame's angles (`drawn_elements`, `crates/compile/src/nodes/device_frame.rs`);
  - image, video and animation are drawn by Use as clip (`CLIP_ELEMENTS`, `crates/compile/src/nodes/media.rs`);
  - captions by Show captions and Captions from script.
- **Every element is Opsis's own code under the same rules, checks and formatting as the rest**, whatever it started as.
- **Theme settings reach the preset tables through a `schemars` transform**; a family of elements with a palette of its own has its own table built with `with_themes`.
- **`ELEMENTS` imports every implementation and `DemoTimeline` looks each one up synchronously**, so a page that opens loads them all once.
- **`ELEMENTS` also holds `placeholder` and `framed-image`, page-only implementations with no element entry in Rust**; their props are `PlaceholderProps` and `FramedImageProps` (Frame for images, [images.md](images.md)).

### Settings to props

- **Every element's exported component takes its settings as compile wrote them** and does in its own file whatever its drawing needs: its background, centring and placement, and the conversions in `lib/settings/shape.ts` (`listsFrom` leaves out an all-empty column).
  - Compile turns seconds into frames, fills kind props and font stacks and drops empty labelled values (`element_props`); on the page `DemoTimeline` only turns `app:` refs into addresses (`withAppFiles`, [assets.md](assets.md)).
- **Nothing else reshapes an element's props**; `registry/framing.tsx` only marks what an implementation is.
- **A component prop is a setting, a fixed prop (`.fixed`), a kind prop (`.kind_prop`), a wrapper's output, or left at the component's own value** when it is timing, a function, a React node or internal.
- **A prop that follows the kind of a file setting is declared with `.kind_prop`** (`media` → `src` on the text mask): compile sets it from `kind_from_path`, so a component never guesses a file's kind from its name.
- **A component has no defaults of its own, so anything drawing an element for another node builds its props from that element's full defaults with its own values on top** (`meta.default_props`, then `element_props`), never a partial map (Device frame's screen, the light-leak cut).
- **Elements make no sound**: no element has a sound setting and a component's own audio props are never settings; sound comes from the sound nodes, Sound effect timed to the clip ([audio.md](audio.md)).

### Themes

- **On themed elements, a colour the component falls back to a theme token for (`prop ?? t.token`) is empty ("Theme") by default** unless its default differs from that token, so the theme switch reaches it.
- **A themed element turns its theme value into its UI kit's provider**:
  - the warm kit's provider takes the mode, and the changed colours go to the element's view (`useWarmTheme`);
  - the neutral kit takes the whole light or dark preset with the changed colours on top, since its components force light mode;
  - an element with its own palette takes its preset for the mode with the changed colours on top (`{ ...PALETTES[theme.mode], ...theme.colors }`).
- **Opsis's own video code keeps colour literals only for internal fallbacks a finished film never shows** (`PAGE_BG`/`BLEED_BG`, empty-clip placeholders), shadows and window frames; `colors.test.ts` lists each with its reason (`ALLOWED`) and fails on a new one and on a listed one that is gone.
  - The test reads only the folders in `DIRS` and the files in `ELEMENT_FILES`, so an element file with fallback colours has to be added to that list to be checked, and each of its literals to `ALLOWED` with its reason.

### Time and size

- **Elements whose entry has `.authored_clock` count time on a 30 fps clock**: their code reads `useAuthoredFrame()`/`useAuthoredConfig()` instead of `useFrame()`/`useFilm()`, and a `<Sequence from>` inside them takes `useRealFrames()`, so a 60 fps project plays them at their own speed.
- **Their timing settings counted in frames are stored as seconds with `x-frames-at: 30`**, and compile turns them back into frames: `map_leaves` walks the settings, rows and nested settings included, and `frames_from_seconds` turns each such leaf, a number or a comma list (`settings_in_frames`, `element_props`).
- **An element's Length defaults to 0 when it can work its length out** (`clip_duration_field`).
  - It reads "Fit the animation" when `lengths.rs` knows how its animation ends or it runs on the 30 fps clock (`fits_its_animation`), a wrapper included; any other wrapper reads "Match the clip".
  - At 0, a wrapper with a clip connected takes the clip's length whichever label it shows, and any other element its animation's length (`compile_element`), where `animation_length` keeps the designer's hold after the last animation.
  - An element built with `optional_clip` carries "Usual length" for while no clip is connected (`x-unconnected-zero-label`).
  - An element on the 30 fps clock with no `element_timing` entry keeps its designed length, scaled to the project's frame rate and its speed.
- **A transition's length in its entry is the overlap it is made for, never the length of a demo around it.** The Transition node plays it at Length 0 and its card warns when a Length squeezes it below 40% (`speed_row`, `crates/compile/src/nodes/transitions.rs`); for spring settle it is the pause between one clip leaving and the next landing.
  - For a transition taken from Remocn, that is the `linearTiming` of its docs example (`components/docs/examples/<key>-example.tsx`), not its `config.ts` `durationInFrames`, which is the whole preview: two scenes and the overlap.
- **Elements laid out in fixed pixels carry a design size and draw in a `DesignBox` scaled to fit the frame**, so other shapes and 4K keep them whole.
  - With `design` the box grows to the frame's shape at that scale; with `exact_design` it keeps exactly the design size, centred.

### Fonts and files

- **An element whose component draws with a family its settings don't name declares it (`.fonts(&["Caveat"])`)**, and a film that uses the element loads it first (`used_font_names`); a test fails on a declared family that isn't bundled.
- **Elements never load fonts themselves**: no font loader packages, no `@fontsource` stylesheets, no `@font-face` of their own, because with every element loaded on every page such a rule applies to all of them ([fonts.md](fonts.md)).
- **Element defaults never name a library and never load from the internet**: neutral copy and `app:` placeholders.

### Checking elements

- **After changing elements, run `bun run elements:render`** (optionally `-- <part of a key> <position>`, the position a share of the length, 0.5 by default): it compiles every element with its defaults in Rust (captions through Captions from script, transitions and spring settle through the Transition node between two Background clips, grouped elements through their group at their kind, Device frame's elements through its angle, wrappers around one Background clip) and renders that frame as a small JPEG (`STILL_WIDTH`) through the frame driver, on a file server that answers prepared files as an export's does.
  - It needs the engine in the dev data folder and `bun run film:bundle`; elements whose default is no file (image, video, animation, 3D model) render empty.
- **Tests check every element's defaults pass its own schema and neither load from the internet nor name a library (`crates/compile/tests/unit/nodes_e.rs`), and that every declared font is bundled (`nodes_c.rs`).**

### 3D elements

- **3D elements draw on `@react-three/fiber`'s `Canvas` through `Stage3D`, which never runs fiber's frame loop**: after each commit it draws once at the film's size and holds the render until that frame is drawn (`useDrawnScene`).
- **Every pose is a pure function of the frame.** oxlint bans fiber's `useFrame` and `opsis/no-three-clock` bans three's clocks and animation loops; model animations call `mixer.setTime`, never `update`.
- **Assets load through `useHeldLoad`**, which releases its hold only after the loaded value has rendered, so a still is never captured between a load and the canvas it mounts.
- **3D text, models, SVG logos and textures load prepared files**; the element only fetches an address ([assets.md](assets.md)). 3D text picks its face with the same `family_for` as the page.
- **The 3D screen (Device frame's 3D angle) shows its clip as a page layer placed with `matrix3d` from `three/project.ts`**; with **End facing the camera** its last pose fills the frame exactly, so it cuts pixel for pixel to the same clip.
- **Camera keys are the `camera` setting, active when the motion or move is Keyframes or the element has neither setting** (`active_camera`, `crates/compile/src/nodes/elements.rs`): compile puts them on the item's `camera` (`ItemBase`, `crates/types/src/timeline.rs`) and `flatten_spec` turns them into `camera_keys` marks (`camera_key_marks`) the Timeline moves ([timeline.md](timeline.md)).

## How it works

### Adding an element

1. Write `<category>/<key>.rs` with its settings struct and `definition()`, add it to the category's `mod.rs` and to the list in `elements()` (`mod.rs`). When it is a variant of something a kind group already draws, add it to that group as a kind (`crates/compile/src/nodes/groups.rs`) rather than giving it a node ([nodes.md](nodes.md), What a node is).
2. Add its settings to `ElementSettings` (`props.rs`), renamed to its key.
3. Write the component and add it to `ELEMENTS` (`elements/index.ts`) as a `component`, `presentation` or `sequencer`.
4. Run `bun run types:export` for `generated/spec.ts`, `default-props.generated.json` and `length-cases.generated.json`.
5. When its animation ends at a known time, add its arm to `element_timing` (`lengths.rs`) and its cases to `length_cases.rs`.
6. When it keeps colour literals, list its file in `ELEMENT_FILES` and each literal with its reason in `ALLOWED` (`colors.test.ts`).
7. Run `bun run elements:render` and look at its still.

## Deliberate limits

- **`elements:render` isn't part of `bun run test`**, because shaders need WebGL; the 3D stills check (`three_stills.rs`) is `#[ignore]`d because it needs the dev engine and the film page, and runs with `--ignored` after `bun run film:bundle`.
- **The shared grain gradient field mounts its shader with the blob shape's `clamp` bounds in order** (`parts/shader-fields.tsx` → `GrainGradientField`, `ORDERED_CLAMP_SHADER`), for the grain gradient background and the Grain dissolve, Ripple zoom and Wave wipe transitions. The library's `clamp(0., 1., length(…))` is undefined: graphics cards read it as `min(1., …)` and Software drawing (export `gl: "swangle"`) as 1, where Grain dissolve would draw no blobs.
