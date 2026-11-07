# Settings and the Inspector

How node and element settings are declared in Rust, how their JSON Schema carries the Inspector's UI, what Rust sends the forms beyond the schema, and how the forms behave. Rust paths are relative to `crates/compile/src/` unless they name another crate; C++ paths to `crates/qt/cpp/`.

## Code map

| Concern                              | Where                                                                                                                                        |
| ------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------- |
| Settings and choice macros           | `crates/macros` → `opsis_macros::settings`, `opsis_macros::choice`                                                                           |
| Schema building, row defaults        | `ParamSchema::new`, `with_row_defaults`                                                                                                      |
| Widget names                         | `elements/introspect.rs` → `FieldWidget` (exported to `packages/video/src/generated/spec.ts`)                                                |
| Form context sent with every compile | `inspector.rs` → `InspectorContext`                                                                                                          |
| `x-show-if` value check              | `crates/compile/tests/unit/nodes_c.rs`                                                                                                       |
| Fields from the schema               | `crates/ui/src/inspector.rs` → `fields`; `inspector_panel.cc` → `InspectorPanel`, `list_fields.cc`, `visual_fields.cc`, `keyframe_fields.cc` |
| Typing as one undo step              | `var_text.cc` → `VarSurface::SetLive`; `inspector_panel.cc` with `FormHold`                                                                  |
| Variable chips                       | `var_text.cc` → `VarText`, `VarSurface` (tokens from the session's `text_tokens`)                                                            |
| Colour field                         | `color_picker.cc` → `ColorPicker`; `color.rs` (formats)                                                                                      |
| Marks field                          | `marks_field.cc`; geometry in `marks.rs`                                                                                                     |
| Gradient field                       | `gradient.rs`, `gradient_view`, `edit_gradient`                                                                                              |
| Empty values dropped from props      | `nodes/elements.rs` → `element_props`; `elements/introspect.rs` → `map_leaves`                                                               |
| Spot and target keys                 | `crates/ui/src/inspector.rs` → `spot_y_key`; the target's role in `<key>Role`                                                                |
| Frames-at-30 conversion              | `frames_from_seconds`                                                                                                                        |

## Rules

### Declaring settings

- **Node settings are Rust structs with `serde` defaults and `schemars` attributes, validated with `jsonschema`**; elements are settings structs registered in `elements/` the same way.
- **A settings struct is marked `#[settings]` and each field `#[setting(…)]`; never add a `derive`, and never write `#[schemars(…)]` or `#[default …]` on a settings field by hand.**
  - `#[settings]` writes `Debug`, `Clone`, `PartialEq`, `SmartDefault`, `Serialize`, `Deserialize`, `JsonSchema`, `#[serde(rename_all = "camelCase", default)]` and `impl Params`.
  - `#[settings(export)]` adds `specta::Type` (element settings and their rows); `#[settings(eq)]` adds `Eq` (settings with no floats); `#[settings(check)]` leaves `impl Params` to the struct, which writes `check`.
- **`#[setting(…)]` takes the label first, then in the order written:** `default = <expr>`, `range(…)`, `length(…)`, `transform = <fn>`, `check = colour` or `colour_or_empty` (or a rule object), a bare word for an `x-` keyword that is `true` (`advanced`, `hidden`), any other `word = <value>` for `x-<word>` with `_` as `-` (`help`, `zero_label`, `show_if`), and `"key" = <value>` for a schema keyword as it is (`"type"`).
- **Every enum a setting picks from is a `#[choice]`**: it writes the derives (`Default` when a variant is `#[default]`, `specta::Type` with `#[choice(export)]`) and `rename_all = "kebab-case"` (another with `#[choice(rename_all = "…")]`).
  - `#[label("…")]` on every variant gives the enum's `x-labels`, keyed by the wire name, and its `label()`, which cards use instead of matching the variants again.
  - A field's own `labels = {…}` is only for a field that names the choices differently from its enum.
- **The label is the schema's `description`, limits are ranges, and everything else is an `x-` keyword**; our own checks are `x-check` (`{ rule, message }`).
- **Every settings schema carries its defaults as its own `default`, and every list of objects its default row as the items' `default`** (`with_row_defaults`), so the form fills and adds rows without working defaults out.
- **Widget names are the `FieldWidget` enum; a test fails on any other.**

### What is a setting

- **Colours, copy, content lists, images, visible sizes and weights, light or dark, and the length of a still are settings.** Only timing the Timeline owns (`delay`, `holdSeconds`, `exitAtInFrames`…), animation tuning, functions and React nodes stay in code.
- **Lists of colours are palettes, and colour settings use the `color` widget.**
- **Technical settings are `advanced`.** The Inspector shows Basic (the default, opened again on every node) and Advanced with its count; a node with no advanced settings shows no tabs, and nested forms list all their fields in order (`SchemaForm`'s `tabs`).

### Keyword meanings the editor relies on

- **`x-unit: "ms"` and `"frames"` are stored as-is and shown in seconds.**
- **`x-frames-at: 30` stores a frame count as seconds**, and compile turns it back into frames (`frames_from_seconds`, also inside rows and comma lists) ([elements.md](elements.md)).
- **`x-zero-label` on a colour, file or text field means empty is unset**: compile drops the empty value (`map_leaves` in `element_props`, also inside rows and nested objects) and the field shows the label ("Theme", "Built-in image"). On an optional number (no default) it is the placeholder, and clearing the field unsets it.
- **`x-unconnected-zero-label` takes its place while none of the node's inputs is connected** (a wrapper whose clip is optional plays its usual length alone).
- **`x-show-if` values are text (`["true"]` for a switch, never `[true]`)**, because the form compares the setting's value as text; `nodes_c.rs` fails on any other.
- **`x-show-if-file: { path: ["image"] }` shows a field only when that setting holds a file of that kind**; Rust sends each file setting's kind (`InspectorContext.file_kinds`, `kind_from_path`), so the editor never reads extensions.
- **`x-show-if-take: ["browser"]` shows a field only for that kind of take**; Rust sends the kind of take the node reaches on its own inputs or a later node's (`InspectorContext.take_kind`), and the field shows while no take is connected.

### Widgets with a contract

- **`spot` keeps its y in `spot_y_key(name)` and shows the frame at the time in its `x-time-key` setting**; `target` keeps the picked element's role in `<name>Role`.
- **`boxes` are areas drawn over a frame of the upstream recording, stored as fractions of the frame.**
- **`marks` draws on the inspected picture, and each drag is written once when it ends**; the selected mark's other settings are its nested form.
- **`gradient` only draws what Rust sends**: Rust parses the CSS gradient and applies each edit (`gradient_view`, `edit_gradient`).
- **`theme` is `{ mode, colors }` over the preset in `x-themes`, storing only changed tokens**; `terminal-theme` names a theme from the catalogue's `terminalThemes`, and empty keeps the take's own.
- **`cut` and `port` pick one of the node's connected numbered inputs over `InspectorContext.ports`**; `cut` leaves out the first ([film.md](film.md), Transitions).
- **`font` lists project, built-in and library fonts; picking a library font copies it into the project** ([fonts.md](fonts.md)).
- **`sound` and `music` offer built-in sounds or songs or a project file, with a play button** ([audio.md](audio.md)).
- **`envelope` and `camera` edit keys that Rust applies** ([timeline.md](timeline.md)).

### Form behaviour

- **Rust sends what a form needs beyond the schema with every compile** (`InspectorContext`: variables, layers, actions, the spot's frame, the picker's page, the project's colours, connected inputs, file kinds, ports, take kind).
- **Plain text fields render variables as chips (`VarText`); never show `{{…}}` to a user.**
- **Fields write on every keystroke, so a focused text field is wrapped in a gesture** (`FormHold` around `VarSurface`'s live typing): typing until it loses focus is one undo step.
- **Lists pick their input from the item schema**: text items a text list (Enter adds, pasting lines adds one per line), numbers a number list (pasting a spreadsheet column fills it), colours a palette, objects rows (each a nested form, so any widget works in a column).
  - All honour the array's `minItems`/`maxItems`; rows reorder by dragging, one undo step per drop, and a list of four or more starts collapsed.
- **The colour field (our own; formats in `color.rs`) keeps the format a value is written in** (`#hex`, `rgb()`, `hsl()`, `oklch()`, with alpha) and rewrites it only when the user picks; its popover is one gesture and offers the project's colours as swatches.
