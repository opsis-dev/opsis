# Nodes

How a node is defined, what its ports carry, how values convert between ports, how executors attach, how numbered inputs grow, and how cards and looks are made. Paths are relative to `crates/compile/src/` unless they name another folder. Settings and the Inspector are in [inspector.md](inspector.md), elements in [elements.md](elements.md), the run in [runs.md](runs.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                                  | Where                                                                                                                                   |
| ---------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| Node definition builder                  | `node.rs` → `NodeDef`, `PortFamily`, `Claim`, `CardInputs`, `style_modifier`, `recording_modifier`, `grade_modifier`, `Category::label` |
| Node definitions                         | `nodes/` (one file per category)                                                                                                        |
| Port types                               | `crates/types/src/workflow.rs` → `PortType`                                                                                             |
| Conversions and connect checks           | `ports.rs` → `conversion`, `can_connect`, `value_converts`, `refused_conversion`                                                        |
| Gathering inputs                         | `compile.rs` → `gather_inputs`, `convert_input`, `film_of`; the runner's copy in `crates/runner/src/context.rs`                         |
| Validation                               | `graph.rs` → `claim_clashes`, `file_need_issues`, `value_issues`                                                                        |
| File needs                               | `node.rs` → `FileNeed`, `FileNeed::refuses`                                                                                             |
| Executors by node type                   | `crates/media`, `crates/render`, `crates/capture`, `crates/image`, `crates/terminal`; check: `crates/render/tests/executors.rs`         |
| Card names of picked actions and markers | `cards.rs` → `picked_names`                                                                                                             |
| Card icons                               | `crates/types/src/cards.rs` → `Icon`; bundled from `lucide` by `scripts/node-glyphs.ts` (`ICONS`)                                       |
| Node looks                               | `crates/types/src/looks.rs` → `NodeLook`, `NodeMotion`, `GLYPH_SETS`                                                                    |
| Glyph data and drawing                   | `crates/ui/glyphs.generated.json`, `crates/ui/src/look.rs` → `icon_svg`, `scripts/lib/glyph-sets.ts`                                    |
| Look checks                              | `crates/compile/tests/unit/looks.rs`, `scripts/node-glyphs.test.ts`, `crates/ui/src/look.rs` → `every_card_icon_is_bundled`             |
| Glyph motions                            | `motions.rs` → `motion_spec`, `sample`; `crates/ui/src/motion.rs`; `crates/qt/cpp/graph_canvas.cc` → `PlayMotion`                       |
| Ports before the first compile           | `catalog.rs` → `node_inputs`                                                                                                            |

## Rules

### Definitions

- **A node definition has ports, settings and an optional pure `compile`; it never generates film code.** Compile produces a `TimelineSpec`; every timeline item has a `kind` (`group`, `recording` with typed `GroupProps`/`RecordingProps`, `element` naming a registry `component`), read by narrowing on `kind`, never by component name.
- **Every node with `executes: true` has an executor**, registered by node type in the crate that runs it; `crates/render/tests/executors.rs` checks it.
- **A node that saves something only one node may save (a sign-in name, an export file) says so with `claims`**, and validation reports every later node claiming the same (`claim_clashes`).
- **Export nodes take a `clip`, and `film_of` makes their film from that clip** when they run and when the Timeline picks its film (`picked_film`, `FilmTarget`); the compile result carries no films.
- **A skipped node passes its input through**, so it never enters a session's steps, a style or a grade.
- **New nodes take the computer's defaults where a definition has `host_defaults`** (Device frame takes the OS window style), applied when a node is added (`new_node_params`).

### Ports and values

- **Every port carries a resource (`PortType`), and the value on it is that resource's typed value** (`crates/types/src/values.rs`).
- **An output connects to an input of the same type or one the conversion table turns it into**, and conversions are lossless and settings-free only (an image or a recording is a file). Anything that needs choices (fit, length, volume, start) is a node.
- **`gather_inputs` converts every value with `convert_input`, so a node always receives the type its port declares.**
- **A value can refuse a conversion its port type allows** (a terminal take has no video file). `value_converts` and `refused_conversion` decide it, and every layer asks them:
  - every compile sends each output's conversions (`EditorCompile.converts`, `output_conversions`), which the canvas's `connects` checks;
  - `edit_workflow` refuses such an edge (`connection_problem_now`);
  - validation puts an error on one already there (`value_issues`).
- **A node says which files its file inputs take with `.needs(FileNeed::…)` after its ports** (an image, a picture, something that plays, sound, a sound file); validation warns when the connected file, converted, is another kind (`file_need_issues`), before anything runs.
- **Nothing walks the graph backwards to find a chain.** Values carry forward what later nodes need (a browser session carries the address the last Go to page opened, [capture.md](capture.md)).

### Growing inputs and many connections

- **Numbered inputs (`in1`, `in2`, …) are a `PortFamily` declared with `.family(…)`**: how many are required, a label for each that knows the last connected one, and `refs`, the list settings that name its ports. Sequence, Layers, the scene sequencers and Join files use it.
- **A family node shows one empty input after the last connected one** (`input_ports`); `NodeDef::input` accepts any member and the catalogue describes the family so the editor and the Local API know any member.
- **Each compile sends every family node's inputs (`EditorCompile.inputs`) and the canvas draws those**; before the first compile it draws the connected members plus one empty one from the edges (`node_inputs`), so an edge into `in3` never hangs loose.
- **Renumbering a family moves the edges and the `refs` settings together** (`renumber_family`), for closing gaps after a removal and shifting ports on a split.
- **A `many()` input accepts any number of edges and `gather_inputs` merges their values** (Add sound's Sounds: their sounds, concatenated).

### Cards

- **Every visible node has a `description` and a `card` of up to four `row(icon, text)` lines**, computed in Rust from interpolated params, run results, saved sign-ins, connected inputs (lengths, takes, files, captions, session step types, style entry types) and the names of picked actions and markers (`CardInputs.names`, so a card says “Launch”, never an id). The editor shows the rows sent with every compile (`cards`).
- **Every `Icon` variant is bundled**: `glyphs:bundle` takes each from `lucide` by its name and fails on a name lucide lacks, and `every_card_icon_is_bundled` fails when a variant has no bundled icon, since a missing one would draw the fallback box.
- **Cards show no picture**: no stills, poster frames or live thumbnails; a node's output is seen in the Run panel.

### Looks

- **A node's icon is its look (`NodeLook`), set with `.look(…)`**; an element's node takes its element's look, sent with the catalogue.
- **A look is a stroke-only glyph from one of the six sets in `GLYPH_SETS`** (`lucide:`, `tabler:`, `lucide-lab:`, `hugeicons:`, `iconoir:`, `mynaui:`), never a filled icon. Pick the glyph that says the most across all six, not the first set that has something close.
- **A look has at most one hover animation: a `motion` (`NodeMotion`) or a `morph` into another glyph.** Animate only when the motion shows what the node does (every transition and text effect, click, drag, morphs between two states); most nodes stay still.
- **Every node has a look, and two nodes share a glyph only if they move differently** (`looks.rs`). Two glyph names whose drawings lie within half a stroke of each other everywhere count as the same glyph (`scripts/node-glyphs.test.ts`, traced with morphicons' `resampleIcon`).
- **Look-alikes drawn differently pass that check, so compare a new glyph with its neighbours by eye.**
- **Run `bun run glyphs:bundle` after changing a look.** It reads the looks through `editor-data` and each set from its own package, keeps only each shape's geometry so every set draws with the badge's 2 px stroke, and fails on aliases, typos, fills, groups, any other attribute and a set whose licence isn't in `THIRD_PARTY_NOTICES.md`.
- **Motions are keyframes in `motions.rs`, sampled in Rust**, played once when a canvas card is hovered and skipped under reduced motion.

## Deliberate limits

- **Nothing converts into a `grade`**; grades come only from grade nodes ([footage.md](footage.md)).
- **Hugeicons, Iconoir and MynaUI are drawn at 1.5 px upstream**; the badge redraws them at 2 px, so their shapes can sit closer to the edge than Lucide's.
