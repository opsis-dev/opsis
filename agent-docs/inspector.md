# Settings and the Inspector

How node and element settings are declared in Rust, how their JSON Schema carries the Inspector's UI, what Rust sends the forms beyond the schema, and how the forms behave. Rust paths are relative to `crates/compile/src/` unless they name another crate; C++ paths to `crates/qt/cpp/`. Elements' settings are in [elements.md](elements.md), node definitions in [nodes.md](nodes.md), the bridge the forms use in [ui.md](ui.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                             | Where                                                                                                                                                                                                                                                       |
| ----------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Settings and choice macros          | `crates/macros` → `opsis_macros::settings`, `opsis_macros::choice`                                                                                                                                                                                          |
| Schema building, row defaults       | `params.rs` → `ParamSchema::new`, `with_row_defaults`, `CHECK_KEYWORD`                                                                                                                                                                                      |
| Widget names                        | `elements/introspect.rs` → `FieldWidget` (exported to `packages/video/src/generated/spec.ts`)                                                                                                                                                               |
| Form context for the inspected node | `inspector.rs` → `InspectorContext`                                                                                                                                                                                                                         |
| `x-show-if`                         | the form: `crates/ui/src/inspector.rs` → `shown`, `ANY_VALUE`; card rows: `nodes/elements.rs` → `shown`; the value check `crates/compile/tests/unit/nodes_c.rs`                                                                                             |
| Kind objects (`x-kind`)             | `crates/compile/src/nodes/groups.rs` → `kind_properties`, `kind_object_shown`; the form `crates/ui/src/inspector.rs` → `form`; paths `crates/compile/src/edits/timeline.rs` → `param_value`, `put_param`                                                    |
| Fields from the schema              | `crates/ui/src/inspector.rs` → `fields`; `inspector_panel.cc` → `InspectorPanel`, `list_fields.cc`, `visual_fields.cc`, `keyframe_fields.cc`                                                                                                                |
| Typing as one undo step             | `var_text.h` → `VarSurface::SetLive`; `inspector_panel.cc` with `FormHold`                                                                                                                                                                                  |
| Variable chips                      | `var_text.cc` → `VarText`, `VarSurface`; tokens from `crates/ui/src/project.rs` → `text_tokens`, `join_tokens` (over `opsis_compile::vars`)                                                                                                                 |
| Colour field                        | `color_picker.cc` → `ColorPicker`; `color.rs` (formats)                                                                                                                                                                                                     |
| Marks field                         | `marks_field.cc`; geometry in `marks.rs`                                                                                                                                                                                                                    |
| Gradient field                      | `crates/compile/src/gradient.rs` → `gradient_view`, `edit_gradient`, called through `crates/ui/src/colors.rs`                                                                                                                                               |
| Empty values dropped from props     | `nodes/elements.rs` → `element_props`, `setting_value`; `elements/introspect.rs` → `map_leaves`                                                                                                                                                             |
| Spot and target keys                | `crates/ui/src/inspector.rs` → `spot_y_key`; the target's role in `<key>Role`, written on a pick by `crates/ui/src/ui.rs` and cleared by typing (`inspector_panel.cc`, `FieldKind::Target`); the hidden-role test in `crates/compile/tests/unit/nodes_a.rs` |
| Frames-at-30 conversion             | `elements/introspect.rs` → `frames_from_seconds`                                                                                                                                                                                                            |
| Basic and Advanced tabs             | `inspector_panel.cc` → `InspectorPanel::AddTabs`                                                                                                                                                                                                            |

## Rules

### Declaring settings

- **Node settings are Rust structs with `serde` defaults and `schemars` attributes, validated with `jsonschema`**; elements are settings structs registered in `elements/` the same way.
- **A settings struct is marked `#[settings]` and each field `#[setting(…)]`; never add a `derive`, and never write `#[schemars(…)]` or `#[default …]` on a settings field by hand.**
  - `#[settings]` writes the derives, the serde attributes (camelCase, defaults) and `impl Params` (`crates/macros/src/settings.rs`).
  - `#[settings(export)]` adds `specta::Type` (element settings and their rows); `#[settings(eq)]` adds `Eq` (settings with no floats); `#[settings(check)]` leaves `impl Params` to the struct, which writes `check`.
- **`#[setting(…)]` takes an optional label first, then its parts in any order** (`crates/macros/src/setting.rs`): `default`, `range`, `length`, `transform`, `check`, a bare word for an `x-` keyword that is `true`, `word = <value>` for `x-<word>` with `_` as `-`, and `"key" = <value>` for a schema keyword as it is. A default given both here and by hand is an error.
- **Every enum a setting picks from is a `#[choice]`**: it writes the derives (`Default` when a variant is `#[default]`, `specta::Type` with `#[choice(export)]`) and `rename_all = "kebab-case"` (another with `#[choice(rename_all = "…")]`).
  - `#[label("…")]` goes on every variant or on none (`crates/macros/src/choice.rs` refuses a mix). Labelled, the enum gets `x-labels`, keyed by the wire name, and a `label()` that cards use instead of matching the variants again; unlabelled, it gets neither and the form shows the wire names.
  - A field's own `labels = {…}` is only for a field that names the choices differently from its enum.
- **The label is the schema's `description`, limits are ranges, and everything else is an `x-` keyword**; our own checks are `x-check` (`{ rule, message }`).
- **Every settings schema carries its defaults as its own `default`, and every list of objects its default row as the items' `default`** (`with_row_defaults`), so the form fills and adds rows without working defaults out.
- **Widget names are the `FieldWidget` enum; a test fails on any other** (`crates/compile/tests/unit/nodes_a.rs`).

### What is a setting

- **Colours, copy, content lists, images, visible sizes and weights, light or dark, and the length of a still are settings.** Only timing the Timeline owns (`delay`, `holdSeconds`, `exitAtInFrames`…), animation tuning, functions and React nodes stay in code.
- **Lists of colours are palettes, and colour settings use the `color` widget.**
  - The form also treats a string whose `x-check` rule is `colour` as a colour field without the widget (`is_color`, `crates/ui/src/inspector.rs`); `colour-or-empty` makes one only with the widget, though compile's own test (`is_color_schema`, `elements/introspect.rs`) counts both rules.
- **Technical settings are `advanced`.** The Inspector shows Basic (the default, shown again when another node is inspected) and Advanced with its count; a node with no advanced settings shows no tabs (`InspectorPanel::AddTabs`), and nested forms list all their fields in order without tabs.

### Keyword meanings the editor relies on

- **`x-unit: "ms"` is stored in milliseconds and shown in seconds** (`number_field`, `crates/ui/src/inspector.rs`); any other unit, `"frames"` included, is shown as stored, with the unit beside the field.
- **`x-frames-at: 30` stores a frame count as seconds**; how compile turns it back into frames is in [elements.md](elements.md), Time and size.
- **`x-step` sets a number field's step and decimals.**
- **`x-help` (a setting's `help`) is the help shown with the field** (`blank`, `crates/ui/src/inspector.rs`).
- **`x-presets` (titled by `x-presets-label`) puts preset buttons above a field** (`InspectorPanel::WithPresets`).
- **`x-pauses` lets a text field hold pause marks** (passed to `VarText`).
- **`x-hidden` keeps a setting out of the form.**
- **`x-zero-label` on a colour, file, select or text field means empty is unset**: for an element, compile drops the empty string (`setting_value` through `map_leaves` in `element_props`; inside a list row the item becomes `null`), and other nodes read the empty value themselves.
  - Colour and text fields show the label in place of empty ("Theme"); on a number it is shown in place of 0, and a number whose schema allows `null` also gets a Clear button that unsets it.
- **`x-unconnected-zero-label` takes the zero label's place while none of the node's inputs is connected** (a wrapper whose clip is optional plays its usual length alone).
- **`x-show-if` values are text (`["true"]` for a switch, never `[true]`)**, because the form compares the setting's value as text; `nodes_c.rs` fails on any other.
  - `"*"` matches any value that isn't empty (`ANY_VALUE`).
- **`x-show-if-file: { path: ["image"] }` shows a field only when that setting, or the `media` input of that name, holds a file of that kind**; Rust sends the kinds (`InspectorContext.file_kinds`), so the editor never reads extensions.
  - A file setting's kind comes from its path (`kind_from_path`, `file_kinds`), a top-level `asset` setting or one in the picked kind's object; a connected input's from the value on it (`connected_file_kinds`), which is how Use as clip shows each kind of file its own settings.
  - List rows have no kind, so it doesn't work inside them.
- **A kind group's object (`x-kind`) is drawn as the node's own fields, not as a nested form** (`form`), each field keyed `kind/prop` and in Basic or Advanced by its own `x-advanced`; only the picked kind's object shows, through its `x-show-if` on `kind` ([nodes.md](nodes.md), Kind groups).
- **`x-show-if-take: ["browser"]` shows a field only for that kind of take**; Rust sends the kind of take the node reaches on its own inputs or a later node's (`InspectorContext.take_kind`), and the field shows while no take is connected.

### Widgets with a contract

- **`spot` keeps its y in `spot_y_key(name)` and shows the frame at the time in its `x-time-key` setting.**
- **`target` keeps the picked element's role in a hidden `<name>Role` setting that the node declares**; `nodes_a.rs` fails on a `target` without one.
- **`boxes` are areas drawn over the node's spot picture (a frame of the upstream recording, the graded file or the image input), stored as fractions of the frame.**
- **`marks` draws on the inspected picture, and each drag is written once when it ends**; the selected mark's other settings are its nested form.
- **`gradient` only draws what Rust sends**: Rust parses the CSS gradient and applies each edit (`gradient_view`, `edit_gradient`).
- **`theme` is `{ mode, colors }` over the preset in `x-themes`, storing only changed tokens**; `terminal-theme` names a theme from the catalogue's `terminalThemes`, and empty keeps the take's own.
- **`font` lists project, built-in and library fonts; picking a library font copies it into the project** ([fonts.md](fonts.md)).
- **`sound` and `music` are a list of the catalogue's built-in sounds or songs, and None** ([audio.md](audio.md)).
- **`envelope` and `camera` edit keys that Rust applies** ([timeline.md](timeline.md)).

### Form behaviour

- **Rust sends what the inspected node's form needs beyond the schema with every compile** (`InspectorContext`).
- **Plain text fields render variables as chips (`VarText`); never show `{{…}}` to a user.**
  - Rust splits the text into variables, pause marks and plain text and joins it back (`text_tokens`, `join_tokens`), so the field has no tokenizer of its own.
- **Fields write on every keystroke, so a focused text field is wrapped in a gesture** (`FormHold` around `VarSurface`'s live typing): typing until it loses focus is one undo step.
- **Lists pick their input from the item schema**: text items a text list, numbers a number list (each with an Add button and a remove button per row, `list_fields.cc`), colours a palette, choices a row of toggles, objects rows (each a nested form, so any widget works in a column).
  - All honour the array's `minItems`/`maxItems`; rows reorder by dragging, one undo step per drop.
  - An object row duplicates below itself, and a list of four or more object rows starts collapsed (`kCollapseFrom`).
- **Nested forms open two levels deep** (`NESTING`, `crates/ui/src/inspector.rs`); deeper rows show only their summary.
- **The colour field (our own; formats in `color.rs`) keeps the format a value is written in** and rewrites it only when the user picks; its popover is one gesture and offers the project's colours as swatches.
