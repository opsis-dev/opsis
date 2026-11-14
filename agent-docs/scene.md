# The scene

What every element returns and how a frame is worked out: the scene nodes, the clock and its maps, the timeline walker, measuring, content hashes and the caches. Paths are relative to `crates/scene/src/` unless they name another crate. Laying out, painting and the renderer are in [film.md](film.md), elements in [elements.md](elements.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                      | Where                                                                                                                                                                                               |
| ---------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Nodes                        | `node.rs` → `Node` (an element or text), `Element`, `Kind` (`div`, `span`, image, inline SVG, footage, Lottie, effect, 3D stage, drawing, slot), `div`, `span`, `svg`                               |
| Style                        | `style.rs` → `Style` (one `Prop` per CSS property set), `without_placement`; values `values.rs` (`Length`, `Calc`, `Px`, `Alpha`, `Num`); CSS text `css.rs`; colour `color.rs`                      |
| Clock and its maps           | `clock.rs` → `Clock`, `sequence`, `authored`, `real_frames`, `AUTHORED_FPS`, `js_round`; motion blur samples `shutter.rs`                                                                           |
| Timeline walker              | `timeline.rs` → `draw_film`, `tracks`, `Library`, `SceneTracks`, `At`, `Stage`, `Presented`, `transition_wrap`, `design_box`                                                                        |
| The film's library           | `crates/compile/src/elements/library.rs` → `FilmLibrary`; `crates/compile/src/elements/draw.rs` → `element`, `wrapper`, `presentation`, `scenes`                                                    |
| Measuring                    | `measure.rs` → `Measure`, `MeasuredBox`, `AnimationTiming`; served by `crates/film/src/lib.rs` → `FilmMeasure` over `crates/layout/src/measure.rs` → `Measurer`                                     |
| Animation maths              | `spring.rs` → `spring`, `spring_progress`, `measure_spring`; `interpolate.rs`; `easing.rs`; `keyed.rs`; `random.rs` → `random`; `noise.rs`                                                          |
| 2D drawing calls             | `drawing.rs` → `Drawing` (the Canvas 2D calls as data)                                                                                                                                              |
| 3D, typefaces, vector shapes | `node.rs` → `Stage3d`, `Shape3d`, `Model3d`; `typeface.rs`; `vector.rs`                                                                                                                             |
| Hashes                       | `hash.rs` → `content_hash`, `element_content_hash`, `full_hash`                                                                                                                                     |
| Tests                        | `crates/scene/tests/{clock,timeline,hash,css,shutter,noise,parity}.rs`; `crates/render/tests/goldens.rs` → `chosen_elements_build_their_golden_scenes`, `elements_hold_still_after_their_animation` |

## Rules

### Every frame is a function of its frame number

- **An element is a pure function of its settings, its clock and its stage**: no element reads a wall clock, a random source, a file or the network. Randomness is `random(seed)`, noise is `noise.rs`, and both give the same numbers for the same seed.
- **Time is a value passed down, never a global.** `Clock` carries the frame (fractional, because remaps are), the rate and the length; `sequence`, loops, group `remap`, start offsets and each motion-blur sample map one clock to another.
- **`spring` is closed form and `measure_spring` is memoised (`comemo`)**, so frame n needs no earlier frame and a spring's rest length is worked out once.
- **Elements registered with `.authored_clock` see a 30 fps clock** (`Clock::authored`, `AUTHORED_FPS`), and a nested sequence inside them counts in real frames (`real_frames`), so a 60 fps project plays them at their own speed ([elements.md](elements.md)).
- **The maths ports keep their rounding step by step** (`js_round`, the `#[expect(clippy::suboptimal_flops)]` on the curve and spring code), because the recorded goldens hold what that draws.

### The scene contract

- **A node carries the film's own CSS and SVG subset under the same names, with CSS meaning and the reset's defaults**: `box-sizing: border-box`, no margin or padding, `line-height: 1.5`. Porting a construct is a lookup, never a workaround.
- **Leaves are what CSS has no word for**: a picture or footage frame, a Lottie file at a frame, a 3D stage, a shader effect over child nodes, a 2D drawing, and a slot a transition fills.
- **Inline SVG keeps its tags and attributes**; it is drawn through `usvg` ([film.md](film.md)).
- **A missing component draws "Unknown element: <name>" in the error colour** (`missing`, `timeline.rs`) instead of failing the frame; a component that fails returns a `DrawError` naming it, which fails the render.

### The timeline walker

- **`draw_film` turns the `TimelineSpec` into one scene per frame**: tracks, items with `from` and length, envelopes, groups with `remap`, crop and design box, wrappers, sequencers and transitions, calling the `Library` for each element, wrapper, presentation, scene series and take.
- **A transition pairs its sides through slots**: the outgoing side wraps its scene in a `Kind::Slot` named by the cut, and the incoming side's effect holds an empty slot of that name; paint draws the named scene into that input and hides it where it stands ([film.md](film.md), Effects).
- **Elements that measure before they lay out ask `Measure`**: text widths and ink, laid-out boxes, a Lottie file's timing, a terminal's frames, a picture's size, a prepared typeface or vector shape. The film answers from the same fonts and files it draws with.

### Hashes and caches

- **Every node hashes by its content**; a group's own placement (position, transform, opacity) stays out of its content hash (`without_placement`), so moving or fading a group keeps what's inside cached.
- **Floats hash through `Px` (1/64 px steps), `Alpha` (8-bit steps) and `Num` (exact bits)**, so held frames hash equal and a cache key never misses a real change.
- **A frame whose whole scene hashes the same as the last one, at the same scale, reuses its picture** (`Film::scaled_frame`, `crates/film/src/lib.rs`).
- **Decoded pictures, SVG trees, typefaces, Lottie files and composited layers stay in `opsis_paint::Cache` for as long as each was used in the last two frames** (`KEPT_FRAMES`, `crates/paint/src/resources.rs`); a layer is keyed by its element, its laid-out subtree and its raster transform. Shaped paragraphs stay in `Fonts` the same way, keyed by text, styles and boxes (`Fonts::next_frame`, `crates/layout/src/fonts.rs`).
- **Picture sizes are looked up once per film** (`known_sizes`).
- **An element with `will-change: transform` is painted as a cached layer at its exact scale**, keyed by its whole subtree and raster transform (`paint_composited`, `crates/paint/src/lib.rs`); the recording's camera is one.
- **A layer marked `will-change: scale` is rastered at its scale stepped up to the next eighth of an octave (`stepped_scale`), over its visible region snapped to a 256 px grid (`LAYER_GRID`), and keyed by its content without its own placement (`stepped_layer_key`)**, so a zoom draws it once per step and a pan inside the region not at all; the picture is drawn down by at most an eighth of an octave.
  - The camera asks for it only on frames whose scale differs from the next frame's (`camera`, `crates/compile/src/elements/camera/recording.rs`), so a still camera keeps its exact raster.
  - The choice is a function of the frame alone, never of which frames a worker drew before: a layer drawn from the cache equals the same layer drawn fresh, which `crates/paint/tests/layers.rs` checks.

## How it works

### Checks

- **`clock.rs` and `timeline.rs` tests are property tests**: a frame evaluated twice is equal, a frame evaluated after another equals a fresh one, the clock maps compose.
- **`elements_hold_still_after_their_animation` draws every element after its animation length and checks the scene stays put within `SETTLED`**, skipping the elements that keep moving on purpose (`KEEPS_MOVING`, with the reason for each).
- **Twelve chosen elements' scenes are recorded at three frames** (`fixtures/scene/elements/<key>@<frame>.json`), so a change in what an element returns shows before it shows in a picture.

## Deliberate limits

- **The scene types are Rust only**; node packs will reach them through WIT ([plans/node-packs.md](../plans/node-packs.md)).
