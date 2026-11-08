# Elements

How an element is written in Rust and on the page, how its settings reach its component, how its length and size are worked out, and the 3D elements. Rust paths are relative to `crates/compile/src/elements/`, page paths to `packages/video/src/`. Settings declarations are in [inspector.md](inspector.md), the film page in [film.md](film.md), prepared files in [assets.md](assets.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                        | Where                                                                                                                                                                                             |
| ------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| One element's definition       | `<category>/<element>.rs` (categories `media`, `text`, `overlay`, `transition`, `background`, `effect`, `ui`, `data`, `scene`, `device`)                                                          |
| The builder                    | `entry.rs` → `element`, `meta`                                                                                                                                                                    |
| Node list order                | `mod.rs` → `elements()`                                                                                                                                                                           |
| Choices several elements share | `shared.rs`                                                                                                                                                                                       |
| Theme preset tables            | `theme.rs` → `neutral_themes`, `warm_themes`, `code_themes`, `keystroke_themes`, `stat_card_themes`, `data_story_themes`…, `with_themes`                                                          |
| Props built from settings      | `props.rs` → `ElementSettings`; `crates/compile/src/nodes/elements.rs` → `element_props`; `crates/compile/src/nodes/capture.rs` → `StillProps`; `crates/compile/src/clip.rs` → `PlaceholderProps` |
| Element sounds                 | `element_sounds`                                                                                                                                                                                  |
| Lengths                        | `lengths.rs` → `element_timing`, `animation_length`; checked by `registry/lengths.test.ts` through `length-cases.generated.json`                                                                  |
| Components                     | `elements/<category>/<key>.tsx` or `elements/<category>/<key>/index.tsx`                                                                                                                          |
| Key → implementation           | `elements/index.ts` → `ELEMENTS`; lookup `registry/index.ts` → `resolveElement`                                                                                                                   |
| What an implementation is      | `registry/framing.tsx` (`component`, `presentation`, `sequencer`)                                                                                                                                 |
| Settings shape helpers         | `lib/settings/shape.ts` → `nestedFrom`, `listsFrom`, `ticksFromWords`, `treeFromPaths`                                                                                                            |
| UI kits                        | `lib/neutral-ui`, `lib/warm-ui` (themes only), `lib/ui-kit` (colour, motion, step and timeline helpers)                                                                                           |
| 30 fps clock                   | `core/authored-clock.ts` → `useAuthoredFrame`, `useAuthoredConfig`, `useRealFrames`                                                                                                               |
| Design box                     | `DesignBox`                                                                                                                                                                                       |
| Colour literal allow-list      | `colors.test.ts`                                                                                                                                                                                  |
| Element stills                 | `bun run elements:render` → `crates/render/src/bin/render-elements.rs`, into `~/.cache/opsis-dev/element-stills`                                                                                  |
| 3D stage and loaders           | `three/` → `Stage3D`, `useDrawnScene`; `project.ts`                                                                                                                                               |
| 3D stills check                | `crates/render/tests/three_stills.rs` against `fixtures/prepare/stills`                                                                                                                           |

## Rules

### One shape for every element

- **Every element is one Rust file holding its settings struct, the rows and choices only it uses, and `definition()`**, which registers it with the one builder (kind, key, title, description, category, length, design size, fixed props, kind props, the 30 fps clock, look).
- **An element's key is its plain name (`title`, `soft-blur-in`) and its node type `fx.<key>` or, for a transition, `transition.<key>`.** No name, type, path or field says where an element came from; only `THIRD_PARTY_NOTICES.md` records that.
- **Every element is Opsis's own code under the same rules, checks and formatting as the rest**, whatever it started as.
- **Theme settings reach the preset tables through a `schemars` transform**; a family of elements with a palette of its own has its own table built with `with_themes`.
- **`ELEMENTS` imports every implementation and `DemoTimeline` looks each one up synchronously**, so a page that opens loads them all once.

### Settings to props

- **Every element's exported component takes its settings exactly as stored** and does in its own file whatever its drawing needs: its background, centring and placement, and the conversions in `lib/settings/shape.ts` (`listsFrom` leaves out an all-empty column).
- **Nothing outside an element reshapes its props**; `registry/framing.tsx` only marks what an implementation is.
- **A component prop is a setting, a fixed prop (`.fixed`), a kind prop (`.kind_prop`), a wrapper's output, or left at the component's own value** when it is timing, a function, a React node, internal or a sound.
- **A prop that follows the kind of a file setting is declared with `.kind_prop`** (`media` → `src` on the text mask): compile sets it from `kind_from_path`, so a component never guesses a file's kind from its name.
- **A component has no defaults of its own, so anything drawing an element for another node builds its props from that element's full defaults with its own values on top** (`meta.default_props`, then `element_props`), never a partial map (Device frame's screen, the light-leak cut).
- **A `sound` setting (with `<key>Volume`) never reaches the component**: compile turns it into a music item on the node's own Mixer channel (`element_sounds`), and a component's own audio props are never settings.

### Themes

- **On themed elements, a colour the component falls back to a theme token for (`prop ?? t.token`) is empty ("Theme") by default** unless its default differs from that token, so the theme switch reaches it.
- **A themed element turns its theme value into its UI kit's provider**:
  - the warm kit takes the mode and the changed colours;
  - the neutral kit takes the whole light or dark preset with the changed colours on top, since its components force light mode;
  - an element with its own palette takes its preset for the mode with the changed colours on top (`{ ...PALETTES[theme.mode], ...theme.colors }`).
- **Opsis's own video code keeps colour literals only for internal fallbacks a finished film never shows** (`PAGE_BG`/`BLEED_BG`, empty-clip placeholders), shadows and window frames; `colors.test.ts` lists each with its reason and fails on a new one.

### Time and size

- **Elements whose entry has `.authored_clock` count time on a 30 fps clock**: their code reads `useAuthoredFrame()`/`useAuthoredConfig()` instead of `useFrame()`/`useFilm()`, and a `<Sequence from>` inside them takes `useRealFrames()`, so a 60 fps project plays them at their own speed.
- **Their timing settings counted in frames are stored as seconds with `x-frames-at: 30`.**
- **An element's Length defaults to 0, "Fit the animation", when `lengths.rs` knows how its animation ends or when it runs on the 30 fps clock**; `animation_length` keeps the designer's hold after the last animation.
- **Elements laid out in fixed pixels carry a `design` size (1920×1080 in their entry) and draw in a `DesignBox` scaled to fit the frame**, so other shapes and 4K keep them whole.

### Fonts and files

- **An element whose component draws with a family its settings don't name declares it (`.fonts(&["Caveat"])`)**, and a film that uses the element loads it first (`used_font_names`); a test fails on a declared family that isn't bundled.
- **Elements never load fonts themselves**: no font loader packages, no `@fontsource` stylesheets, no `@font-face` of their own, because with every element loaded on every page such a rule applies to all of them ([fonts.md](fonts.md)).
- **Element defaults never name a library and never load from the internet**: neutral copy and `app:` placeholders.

### Checking elements

- **After changing elements, run `bun run elements:render`**: it compiles every element with its defaults in Rust (captions through Captions from script, transitions between two solid colours) and renders its middle frame as a 480 px JPEG through the frame driver, on a file server that answers prepared files as an export's does.
  - It needs the engine in the dev data folder and `bun run film:bundle`; elements whose default is no file (image, video, animation, 3D model) render empty.
- **A test checks every element's defaults pass its own schema.**

### 3D elements

- **3D elements draw on `@react-three/fiber`'s `Canvas` through `Stage3D`, which never runs fiber's frame loop**: after each commit it draws once at the film's size and holds the render until that frame is drawn (`useDrawnScene`).
- **Every pose is a pure function of the frame.** oxlint bans fiber's `useFrame` and `opsis/no-three-clock` bans three's clocks and animation loops; model animations call `mixer.setTime`, never `update`.
- **Assets load through `useHeldLoad`**, which releases its hold only after the loaded value has rendered, so a still is never captured between a load and the canvas it mounts.
- **3D text, models, SVG logos and textures load prepared files**; the element only fetches an address ([assets.md](assets.md)). 3D text picks its face with the same `family_for` as the page.
- **The 3D screen shows its clip as a page layer placed with `matrix3d` from `project.ts`**; with **End facing the camera** its last pose fills the frame exactly, so it cuts pixel for pixel to the same clip.
- **Camera keys are the `camera` setting, active when the motion or move is Keyframes**: compile puts them on `TimelineItem.camera` and `flatten_spec` turns them into `cameraKeys` marks the Timeline moves ([timeline.md](timeline.md)).

## Deliberate limits

- **`elements:render` isn't part of `bun run test`**, because shaders need WebGL.
- **The grain gradient element mounts its shader with the blob shape's `clamp` bounds in order.** The library's `clamp(0., 1., length(…))` is undefined: graphics cards read it as `min(1., …)` and Software drawing (export `gl: "swangle"`) as 1, where Grain dissolve would draw no blobs.
