# Nodes

What a node may be, how a node is defined, how kind groups put several elements under one node, what its ports carry, how values convert between ports, how executors attach, how numbered inputs grow, and how cards and looks are made. Paths are relative to `crates/compile/src/` unless they name another folder. Settings and the Inspector are in [inspector.md](inspector.md), elements in [elements.md](elements.md), the run in [runs.md](runs.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                                  | Where                                                                                                                                                                                                      |
| ---------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Node definition builder                  | `node.rs` → `NodeDef`, `PortFamily`, `Claim`, `CardInputs`, `MAX_CARD_ROWS`, `style_modifier`, `recording_modifier`, `grade_modifier`, `clip_modifier`, `with_clip`, `Category::label`                     |
| Kind groups                              | `nodes/groups.rs` → `KindGroup`, `GroupKind`, `group_node`, `kind_values`, `kind_properties`, `saved_workflow`, `groups`, `grouped_elements`                                                               |
| Groups and notes                         | `graph.rs` → `GROUP_TYPE`, `NOTE_TYPE`, `is_structural`, `topo_sort`                                                                                                                                       |
| Node definitions                         | `nodes/<area>.rs`, each with its `nodes()`, gathered in `nodes/mod.rs` → `all()`; `registry.rs` (`Registry::new` refuses a duplicate type, `is_output_type`)                                               |
| Card row helpers                         | `card_rows.rs`                                                                                                                                                                                             |
| Port types                               | `crates/types/src/workflow.rs` → `PortType`                                                                                                                                                                |
| Conversions and connect checks           | `ports.rs` → `conversion`, `can_connect`, `value_converts`, `refused_conversion`                                                                                                                           |
| Gathering inputs                         | `compile.rs` → `gather_inputs`, `merge_values`, `film_of`, `bypass`; `ports.rs` → `convert_input`; the runner calls the same `gather_inputs` (`crates/runner/src/context.rs`)                              |
| Validation                               | `graph.rs` → `claim_clashes`, `file_need_issues`, `value_issues`                                                                                                                                           |
| File needs                               | `node.rs` → `FileNeed`, `FileNeed::refuses`                                                                                                                                                                |
| Executors by node type                   | `crates/media`, `crates/render`, `crates/capture`, `crates/image`, `crates/terminal`; registered in `crates/session/src/run.rs` → `executors`; check: `crates/render/tests/executors.rs` → `all_executors` |
| Card names of picked actions and markers | `cards.rs` → `picked_names`                                                                                                                                                                                |
| Card icons                               | `crates/types/src/cards.rs` → `Icon`; bundled from `lucide` by `scripts/node-glyphs.ts` (`ICONS`)                                                                                                          |
| Node looks                               | `crates/types/src/looks.rs` → `NodeLook`, `NodeMotion`, `GLYPH_SETS`                                                                                                                                       |
| Glyph data and drawing                   | `crates/ui/glyphs.generated.json`, `crates/ui/src/look.rs` → `icon_svg`, `scripts/lib/glyph-sets.ts`                                                                                                       |
| Look checks                              | `crates/compile/tests/unit/looks.rs`, `scripts/node-glyphs.test.ts`, `crates/ui/src/look.rs` → `every_card_icon_is_bundled`                                                                                |
| Glyph motions                            | `motions.rs` → `motion_spec`, `sample`; `crates/ui/src/motion.rs`; `crates/qt/cpp/look_glyph.cc` → `LookPlayer`, `PaintLookGlyph`                                                                          |
| Glyph morphs                             | `scripts/node-glyphs.ts` → `bakeMorphs`; `crates/ui/src/morph.rs` → `frame`, `step`; `crates/qt/cpp/look_glyph.cc` → `PaintMorph`                                                                          |
| Family inputs                            | `catalog.rs` → `node_inputs`; `node.rs` → `NodeDef::input_ports`, `PortFamily::shown_count`                                                                                                                |

## Rules

### What a node is

- **A node does one thing: one action on one kind of input.** Its settings tune that thing and never add a second one, so a node's title says all it does and two jobs never share one set of settings.
- **The same thing on the same kind of input is one node.** Variants are a dropdown (Kind, Style, Type, Animation) and each variant shows only its own settings, so the catalogue never holds two nodes one dropdown apart.
  - `crates/compile/tests/unit/catalogue.rs` → `no_two_nodes_do_the_same_thing` fails when two visible nodes have the same input and output port types and the same first title word; a real exception is listed there with its reason.
- **Different kinds of input may each have their own node for the same thing** (Trim and Trim file, Frame for recordings and Frame for images). Nodes are never merged across port types, because conversions are lossless and settings-free only.
- **What must happen while recording stays on the recording node or the action.** What is drawn into the page during the take can't move to a later node: Record take's "As on screen" cursor and quick-mode labels, the Chapter card, HTML overlay, and an action's "Zoom in on this action" with its zoom group, which tells the automatic Camera about that action ([capture.md](capture.md)).
- **Something that belongs to one clip or one moment rides on it.** A node with the same type in and out adds it (the style chain, the grade chain, a clip's `enter` transition, [film.md](film.md)); nodes that join (Sequence, Layers) only join, so the setting stays next to what it changes.
- **A node is not a film.** Several elements in one are a project template built from nodes (`crates/project/src/templates.json`), so every part of it can be changed and reused.
- **Elements make no sound and never carry another element's job** (a title inside a device, a mark inside a title); Layers combines them, and sound comes from sound nodes ([audio.md](audio.md)).
- **A file reaches the film one way: Media file → Use as clip** (`media.toClip`, [assets.md](assets.md)), whose settings follow the connected file's kind.
- **No bundles of fixes.** Each fix is its own node and they chain (Remove noise, Soften "s" sounds, Fix clicks, Fix clipping), so a fix the file doesn't need is never part of the run.

### Definitions

- **A node definition has ports, settings and an optional pure `compile`; it never generates film code.** Compile produces a `TimelineSpec` ([film.md](film.md) has how its items are read).
- **Every node built with `.executes()` (`executes: true`) has an executor**, registered by node type in the crate that runs it.
  - Each executor crate's list is joined in two places that must name the same crates: `executors` in `crates/session/src/run.rs`, which the app runs, and `all_executors` in `crates/render/tests/executors.rs`, which checks every executing node has one and none has two. A crate missing from the first never runs; one missing from the second isn't checked.
- **A node's `Phase` picks the runner lane it runs in and whether its key takes the project settings** (render-phase nodes, [runs.md](runs.md)); an Output-category node is an export (`is_output_type`).
- **A node that saves something only one node may save (a sign-in name, an export file) says so with `claims`**, and validation reports every later node, skipped ones aside, claiming the same (`claim_clashes`).
- **Export nodes take a `clip`, and `film_of` makes their film from that clip** when they run and when the Timeline picks its film (`picked_film`, `FilmTarget`); the compile result carries no films.
- **Groups and notes (`core.group`, `core.note`) are structural**: they have no ports and never compile or run (`is_structural`; `topo_sort` and the runner skip them).
- **A skipped node passes its input through** (`bypass`), so it never enters a session's steps, a style or a grade.
  - Each output takes the first input of its own type; an output with no input of that type gives nothing.
- **Nodes that change a chain or a clip are built with one of the modifier builders:**
  - style and grade nodes append `{ nodeId, type, params }` to the chain on their optional input (`style_modifier`, `grade_modifier`);
  - take edits (`recording_modifier`) need a connected take and append to its `edits`, giving nothing without one;
  - clip nodes (`clip_modifier`) get only the clip ports and change the clip through `with_clip`.
- **New nodes take the computer's defaults where a definition has `host_defaults`** (Frame and Frame for images take the OS window style), applied when a node is added (`new_node_params`).

### Kind groups

- **A kind group is one node over several elements, its Kind picking the element** (`KindGroup`, built by `group_node`; every group is in `groups`). Each element keeps its own metadata and component; only the node over them is shared.
- **Each kind's settings live in their own object, `params.<kind>.<prop>`**, marked `x-kind` and shown only for its kind (`x-show-if` on `kind`).
  - One object per kind, not one flat set, because the elements' props collide (`speed`, `color`, `fontSize`) and switching Kind and back keeps each kind's values.
  - Settings every kind shares (`KindGroup.shared`, such as 3D object's camera and lens) sit at the top level, shown only for the kinds that have them.
  - **A setting is shared only when every kind that has it gives it the same default.** A shared field takes one kind's default for all of them, so a setting whose defaults differ (3D object's motion, tilt, lighting and background) stays in each kind's object.
  - The Inspector judges a top-level field's `x-show-if` against the top level and the picked kind's object together (`with_picked_kind`, `crates/ui/src/inspector.rs`), so a shared field can follow a kind's setting (3D object's camera keys follow its motion).
- **A group's Length defaults to 0, each kind's usual length** (`duration_field`): the label is the kinds' own zero label when they all have the same one ("Fit the animation", "Match the clip"), "Its usual length" otherwise.
  - A kind can fix props (`GroupKind::fixed`, such as Code's glass look) and hide others (`GroupKind::hide`).
- **A group compiles and draws its card from the picked kind only**: `kind_values` lays the element's full defaults, then the fixed props, the shared settings the element takes and the kind's own object (`compile_element`, `element_card`).
- **Saving keeps only the picked kind's object** (`saved_workflow`, called by `workflow_text` in `crates/project/src/files.rs`), so a file holds no settings for kinds it doesn't use.
- **Everything that names a setting takes a path into the kind's object**: the Inspector shows the picked object's fields as the node's own ([inspector.md](inspector.md)), and Timeline handles and edits read and write `kind/prop` paths (`param_value`, `put_param` in `edits/timeline.rs`).
- **An element under a group has no node of its own** (`grouped_elements`, which also lists the elements Device frame and Use as clip draw).
- **Device frame and the Transition node keep each drawn element's settings in the same objects** (`nodes/device_frame.rs`, `nodes/transitions.rs`); Frame (`camera.frame`) and Frame for images are plain settings switched by `x-show-if`, since they draw no element per kind.
- `crates/compile/tests/unit/catalogue.rs` → `every_group_compiles_at_every_kind` compiles each group at each of its kinds and checks the picked element is drawn, so a kind path that reads nothing fails there; `a_group_at_its_defaults_draws_each_kind_as_its_element_does` fails when a group's own defaults (Length, a shared setting) change what a kind draws.

### Ports and values

- **Every port carries a resource (`PortType`), and the value on it is that resource's typed value** (`crates/types/src/values.rs`).
- **An output connects to an input of the same type or one the conversion table turns it into**, and conversions are lossless and settings-free only (an image or a recording is a file). Anything that needs choices (fit, length, volume, start) is a node.
- **`gather_inputs` converts every value with `convert_input`, so a node always receives the type its port declares.**
- **A value can refuse a conversion its port type allows** (a terminal take has no video file). `value_converts` and `refused_conversion` decide it:
  - the canvas asks before it connects (`connection_problem` in `crates/ui/src/ui.rs`, which compiles and calls `connection_problem_now`);
  - every edit that adds an edge, from the editor or the Local API, goes through the session's `document::edit`, which refuses it (`refused_edge`, `connection_problem_now`); the Local API route itself checks only the port types (`connection_problem`, [api.md](api.md));
  - validation puts an error on one already there (`value_issues`).
- **A node says which files its media inputs take with `.needs(…)` after its ports** (`FileNeed`); validation warns when the connected file, converted, is another kind (`file_need_issues`, skipping turned-off nodes and repeated warnings), before anything runs.
  - `.needs` applies only to `media` ports, and every need refuses a terminal take's `.cast` (`FileNeed::refuses`, `TERMINAL_TAKE_FILE`), which has no picture or sound of its own.
- **Nothing walks the graph backwards to find a chain.** Values carry forward what later nodes need (a browser session carries the address the last Go to page opened, [capture.md](capture.md)).

### Growing inputs and many connections

- **Numbered inputs (`in1`, `in2`, …) are a `PortFamily` declared with `.family(…)`**: how many are required and a label for each that knows the last connected one. No setting names a family's ports, so renumbering moves only edges. The nodes that use it call `.family(` in `nodes/`.
- **A family node shows one empty input after the last connected one, and never fewer than `PortFamily::shown`, its required count but at least two** (`NodeDef::input_ports`, `PortFamily::shown_count`); `NodeDef::input` accepts any member and the catalogue describes the family so the editor and the Local API know any member.
- **Every family node's inputs come from the edges through one function, `node_inputs`**: each compile sends them (`EditorCompile.inputs`) and before the first compile the canvas works them out itself (`crates/ui/src/graph.rs`), so an edge into `in3` never hangs loose.
- **Renumbering a family moves its edges** (`renumber_family`), for closing gaps after a removal and shifting ports on a split.
- **A `many()` input accepts any number of edges and `gather_inputs` merges their values** (`merge_values`): sounds are concatenated (Add sound's Sounds); for any other type the last edge's value wins.

### Cards

- **Every visible node has a `description` and a `card` of at most `MAX_CARD_ROWS` `row(icon, text)` lines**, computed in Rust from interpolated params, run results, saved sign-ins, connected inputs (lengths, takes, files, captions, session step types, style entry types) and the names of picked actions and markers (`CardInputs.names`, so a card says “Launch”, never an id). The editor shows the rows sent with every compile (`cards`); before the first compile it works them out itself (`crates/ui/src/graph.rs`, calling `cards.rs` → `workflow_cards`).
- **Every `Icon` variant is bundled**, since a missing one would draw the fallback box.
  - A new variant is added by hand to `ICONS` in `scripts/node-glyphs.ts`; `glyphs:bundle` takes each from `lucide` by its name and fails on a name lucide lacks.
  - `every_card_icon_is_bundled` fails when a variant has no bundled icon.
- **Cards show no picture**: no stills, poster frames or live thumbnails; a node's output is seen in the Run panel.

### Looks

- **A node's icon is its look (`NodeLook`), set with `.look(…)`**; an element's node takes its element's look, sent with the catalogue.
- **A look is a stroke-only glyph from one of the sets in `GLYPH_SETS`, written `<set>:<name>`**, never a filled icon. Pick the glyph that says the most across every set, not the first set that has something close.
- **A look has at most one hover animation: a `motion` (`NodeMotion`) or a `morph` into another glyph.** Animate only when the motion shows what the node does (every transition and text effect, click, drag, morphs between two states); most nodes stay still.
- **Every node has a look, and two nodes share a glyph only if they move differently** (`crates/compile/tests/unit/looks.rs`, for the same glyph name).
- **Two glyph names whose drawings lie within half a stroke of each other everywhere count as the same glyph** (`scripts/node-glyphs.test.ts`, traced with morphicons' `resampleIcon`).
- **Look-alikes drawn differently pass that check, so compare a new glyph with its neighbours by eye.**
- **Run `bun run glyphs:bundle` after changing a look.** It reads the looks through `editor-data` and each set from its own package, keeps only each shape's geometry so every set draws with the badge's 2 px stroke, and fails on aliases, typos, fills, groups, any other attribute and a set whose licence isn't in `THIRD_PARTY_NOTICES.md`.
- **Canvas cards and node list rows play their look on hover through one `LookPlayer` and one painter (`PaintLookGlyph`)**, so a row's smaller badge shows the same animation as its card.
- **Motions are keyframes in `motions.rs`, sampled in Rust**, played once per hover and skipped under reduced motion.
- **Morphs are baked when glyphs are bundled, not computed in the app**: `glyphs:bundle` runs morphicons' plan and polar interpolation for each glyph pair and stores frames past both ends (`MORPH_FROM`, `MORPH_STEP`, `MORPH_STEPS`), so the spring's overshoot has frames too. `morph::frame` blends the two frames around a position.
- **A morph follows morphicons' default spring** (`morph::step`: its snappy preset, constants in `crates/ui/src/morph.rs`): towards the second glyph while hovered and back when the pointer leaves, from wherever it is. Under reduced motion it switches glyphs at once. At rest the badge draws the glyph itself, and in flight the baked lines.

## How it works

### Adding a node

1. Write its settings struct and its `NodeDef` (`NodeDef::of::<P>(type, title, Category, Phase)` or a modifier builder) in the `nodes/` file for its area, with a description, card and look, and add it to that file's `nodes()`.
2. If it executes, mark it `.executes()`, write its executor in the crate that runs that kind of work and register it there; a new executor crate is added to both lists named in Definitions.
3. Unless it is a film node with only `clip` ports, which the profile project places by itself (`isFilmNode`), add it to the profile project in `scripts/lib/profile/project.ts`; `project.test.ts` fails on a node type the project leaves out.
4. Run `bun run glyphs:bundle` for its look, `bun run types:export` when it changes exported types, and the tests (`crates/compile/tests/unit/nodes_*.rs` check descriptions, banned words, card rows and settings; `looks.rs` its look).

## Deliberate limits

- **Nothing converts into a `grade`**; grades come only from grade nodes ([footage.md](footage.md)).
- **Hugeicons, Iconoir and MynaUI are drawn at 1.5 px upstream**; the badge redraws them at 2 px, so their shapes can sit closer to the edge than Lucide's.
