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
- **SVG trees, typefaces and Lottie files stay in each film's `opsis_paint::Cache` for as long as each was used in the last two frames that film drew** (`KEPT_FRAMES`, `crates/paint/src/resources.rs`). Shaped paragraphs stay in `Fonts` the same way, keyed by text, styles and boxes (`Fonts::next_frame`, `crates/layout/src/fonts.rs`).
- **Decoded pictures and composited layers stay in a `SharedCache` that every worker of a render shares** (`Film::with_shared`): each entry is stamped with the latest output frame that used it, and the render drops those not used by the last two written frames or any frame still being drawn (`SharedCache::sweep`). A film without one keeps its own and sweeps it by its own frames. A picture is keyed by its address, so a footage frame is wrapped once however many workers and motion blur samples show it ([footage.md](footage.md)).
- **Picture sizes are looked up once per film** (`known_sizes`); a footage frame's size is its source's, read without decoding.
- **An element with `will-change: transform` is painted as a cached layer at its exact scale**, and a layer marked `will-change: scale` at its scale stepped up to the next eighth of an octave (`stepped_scale`); the picture is drawn down by at most an eighth of an octave (`paint_composited`, `crates/paint/src/lib.rs`).
  - A `transform` layer covers its whole ink whenever that is at most `WHOLE_LAYER_SHARE` times the surface, so a pan at one zoom, even zoomed in past the picture, places one raster; a `scale` layer, and a `transform` layer larger than that, covers its visible ink, so a zoom draws it once per step and a pan inside what it holds not at all.
  - **Either is cached as fixed 256 px tiles on one grid in layer space** (`LAYER_GRID`, `tiles_of`, `raster_tile`), each keyed by the layer's key and its corner (`tile_key`) and drawn at its own corner with a 2 px overlap all round (`TILE_GUTTER`), which the draw back reads past its core, so neighbouring tiles join without seams. A tile is always rastered at the same origin, so it comes out the same whichever frame drew it: Skia works in 32-bit floats, and a picture drawn at another origin (a larger region that once held it) rounds its edges a level differently. `layers_drawn` counts the passes that rastered any tile.
  - **Both are keyed by the layer's content without its own placement** (`layer_key`): its content hash and opacity, its raster scale, and each box under it by its path below the layer, its laid-out boxes, styles and text and its transform relative to the layer, so a pan or a zoom inside one step draws it once and a moving parent never draws it again.
  - **The raster places each box with the same transform the key holds**: its transform relative to the layer, computed in 64 bits and rounded to 1/64 px in its translation and 1/65536 in the rest (`quantized`, `LayerSpace`, `Placement::place`), so the same key always means the same picture. Every matrix the painter sets is multiplied out in 64 bits first (`Painter::set_matrix`), not by Skia in 32 bits on top of the surface's base, and SVG drawing takes the same placement (`SvgPainter::device`).
  - The recording's window (title bar and take picture) is one, its camera's shadow another; the camera asks for the stepped kind only on frames whose scale differs from the next frame's (`camera`, `crates/compile/src/elements/camera/recording.rs`), so a still camera keeps its exact raster.
  - The choice is a function of the frame alone, never of which frames a worker drew before: a layer drawn from the cache equals the same layer drawn fresh, which `crates/paint/tests/layers.rs` checks.
- **A colour-only filter layer, a filtered element's offscreen surface and a cached layer's region cover only the element's ink** (`Painter::ink`, `crates/paint/src/ink.rs`): its boxes and their shadows, outlines, text overhang and filter reach, mapped through their transforms, so a cursor's lift or drop shadow is a small pass. A subtree whose ink can't be bounded (an SVG with visible overflow, a canvas drawing, a leaf such as Lottie or 3D, a backdrop filter) keeps the whole surface.
  - The bounds are widened to a 32 px grid from the clip's corner (`DITHER_GRID`), so Skia's ordered dithering lines up with what a whole-surface layer draws; sampling still lands a rounding step apart on a few pixels, which the recorded goldens hold.
  - Opacity layers keep the whole clip.

### Motion blur

- **Motion blur stacks every sample in its own layer, opacity `1/(i+1)` for sample `i`** (`shutter_layers`, `crates/scene/src/shutter.rs`): opaque pixels end up the equal-weight average, while a partly covered pixel grows denser with every copy (`crates/scene/tests/shutter.rs`). Every frame stacks all its samples, so an anti-aliased edge has the same weight whether the samples match or not.
- **Matching samples share their rasters instead of merging**: a recording's samples differ only where something moves within the shutter (the camera, the cursor, an annotation, a footage frame, since footage times are snapped to the source's frames, [footage.md](footage.md)), so the window's layer is rastered once per distinct take frame and placed once per sample, which `samples_that_share_one_raster_stack_to_the_same_edges_as_eight_rasters` (`crates/paint/tests/layers.rs`) checks against eight separate rasters.
- **The cursor and the annotations sit outside the window's cached layer**, in a box over the take's area with the window's clip (`overlay_box`, `recording.rs`), so a sample that only moves them places the cached window again and draws them on top. Lift colours filters the window inside its layer, the cursor, and each shown annotation mark on its own (`lift_filters`, `lifted`, `Shown.lift`), each in a layer the size of its ink, which gives the same colours as filtering the window with them inside wherever no colour clamps at black or white, since the filters are affine per pixel; a translucent mark over the take can differ there by a few levels. A spotlight covers the window, so its pass does too; annotations that are all hidden at a frame add nothing.

## How it works

### Checks

- **`clock.rs` and `timeline.rs` tests are property tests**: a frame evaluated twice is equal, a frame evaluated after another equals a fresh one, the clock maps compose.
- **`elements_hold_still_after_their_animation` draws every element after its animation length and checks the scene stays put within `SETTLED`**, skipping the elements that keep moving on purpose (`KEEPS_MOVING`, with the reason for each).
- **Twelve chosen elements' scenes are recorded at three frames** (`fixtures/scene/elements/<key>@<frame>.json`), so a change in what an element returns shows before it shows in a picture.

## Deliberate limits

- **A curve that crosses a layer tile's edge is anti-aliased per tile**: Skia cuts a path where it leaves a surface, so each tile flattens its part of the curve on its own, and a stroke across a seam differs from the same stroke on one surface on a few of its edge pixels, by up to about a quarter of the colour range; it is the same in every frame, which is what the tiles are for (`raster_tile`, `crates/paint/src/lib.rs`).
- **The scene types are Rust only**; node packs will reach them through WIT ([plans/node-packs.md](../plans/node-packs.md)).
