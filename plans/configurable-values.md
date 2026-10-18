# Configurable values

Every value a user would reasonably want to change in a node must be a setting in the Inspector, not a constant in code. An audit on 2026-10-18 found about 45 nodes that hard-code colours, copy, content lists, images or sizes. This plan fixes all of them, adds the Inspector inputs the fixes need, and adds checks so new nodes can't regress.

Status: plan only. Nothing here is built yet.

## How to read this plan

- The work is split into six phases. Phases 1 and 2 build the inputs and the ingest support that phases 3 and 4 rely on, so they come first. Each phase is checked (see **Checks**) before the next starts.
- Phases 3 and 4 have one row per node: what is hard-coded today, the new settings, the input they use, and notes.
- **Shadcn inputs** at the end lists the components researched for the new inputs, with a recommendation for each.

## What counts as configurable

A value must be a setting when a user can see it in the video and would want it different for their own demo:

- **Colours:** text, background, accent, panel and border colours, and palettes (`colors` lists).
- **Copy:** titles, labels, button text, captions, sample sentences.
- **Content:** code, keys pressed, list items, chart data, steps of a scripted flow, logos and screen images.
- **Visible size and weight:** font size and weight, and the size of a cursor, ripple or keycap.
- **Light or dark:** when the element was designed for both.
- **Length:** how long a still image lasts, when nothing downstream sets it.

A value may stay in code when it's one of these:

- **Timing owned elsewhere:** `delay`, `holdSeconds`, `exitAtInFrames`, `delayInFrames` and similar. The Timeline, the clip length and the length handles already own when an element starts, holds and ends. Hide them with an `omit` entry that gives the reason.
- **Animation tuning:** easing curves, spring stiffness and damping, internal offsets and sampling constants, unless the effect is plainly visible (then expose it as an advanced setting).
- **Functions and React nodes:** `valueFormatter`, `className`, `children`, `ref`. Where a React node carries user content (logo marks), a wrapper turns a plain setting into it (phase 2).
- **Values the project already owns:** the project's fonts, size and frame rate.

## Conventions

These restate rules from `AGENTS.md`; when the two disagree, `AGENTS.md` wins.

**Where code goes.**

- Never hand-edit `src/video/elements/**`, `src/video/lib/**`, `src/video/hooks/**` or any `*.generated.ts`. Registry elements get settings through `scripts/registry-ingest.list.ts` (`controls`, `defaults`, `omit`, `NEUTRAL_DEFAULTS`, `NEUTRAL_FIXED_PROPS`) and new ingest features in `scripts/lib/registry-controls.ts` (reading configs, inferring controls) and `scripts/lib/registry-codegen.ts` (writing schemas), then `bun run registry:ingest -- --no-add`.
- Prop wrappers that turn flat settings into nested props live in `src/video/registry/framing.tsx` next to `placedAt` and `painted`, and are picked by the ingest's `WRAPPERS` list.
- Demoflow's own elements: schema in `src/shared/elements/core.ts`, component under `src/video/core/` or `src/video/camera/`.
- Demoflow's own nodes: params in `src/shared/nodes/*.ts`; values reach the video through `compile()` and the spec types in `src/shared/compile/`.
- New Inspector inputs: field components in `src/renderer/editor/form/`, dispatched from `FieldControl` in `src/renderer/editor/zod-form.tsx`. A new widget name goes into `FIELD_WIDGETS` (`src/shared/elements/introspect.ts`); the registry test fails on unknown widgets.
- Components from a shadcn registry are installed with `bunx --bun shadcn@latest add …` into `src/renderer/components/ui/` and never edited by hand. If one doesn't pass the type check as-is, write a small equivalent in `src/renderer/editor/form/` instead (the same rule `AGENTS.md` sets for Motion Primitives).

**Defaults.**

- Every new setting defaults to the value the component uses today, so existing projects and examples look the same. The exception is defaults that break the rules below; those change.
- Defaults never name a library, product or company and never load from the internet: neutral copy ("Acme") and `app:` placeholders (`public/placeholders`, `public/backdrops`). The ingest's `defaultProblems` check covers string defaults; phase 5 extends it to defaults hidden in React nodes (logo marks).

**User-facing copy.**

- Field labels in sentence case, durations in seconds, no library or format names. Technical settings go under `advanced`.
- Card rows (`card()`) show the most telling new setting when it changes the look, such as a palette swatch row for gradients.

**Behaviour.**

- Everything a setting changes in `src/video` stays a pure function of `useCurrentFrame()`.
- A setting that reaches the picture changes the node hash through its params, so exports re-render on their own. `PICTURE_VERSION` only needs a bump if something changes the picture without being in the spec.

**No old formats.** Nothing has been released, so there is no migration code. When a param's shape changes, write `examples/` again (`scripts/write-examples.ts`) and delete local test projects.

## Summary

| Phase | What                                | Nodes affected                                                  | Depends on |
| ----- | ----------------------------------- | --------------------------------------------------------------- | ---------- |
| 1     | New Inspector inputs                | All of phases 3 and 4, plus elements that show a JSON box today | none       |
| 2     | Ingest support for the new controls | All registry elements in phase 3                                | 1          |
| 3     | Registry elements                   | Every element in 3a–3e                                          | 1, 2       |
| 4     | Demoflow's own nodes                | 6 nodes and 2 core elements                                     | 1          |
| 5     | Checks that stop regressions        | All                                                             | 2          |
| 6     | Docs                                | n/a                                                             | 5          |

## Phase 1: Inspector inputs

**Today.** `schemaKind` (`src/renderer/editor/form/schema-kind.ts`) knows strings, numbers, booleans, enums, enum lists and lists of `z.object`. Everything else falls back to `JsonField`, a raw JSON textarea. That already affects shipped elements: Terminal simulator's `lines` (its help text asks users to type JSON), Donut chart's and Bar chart race's `colors`, the remotion-ui `metrics`, `stages`, `segments`, `rows` and `series`, and Comparison bars' `seriesLabels`. `ColorField` wraps a native `<input type="color">`, which only takes `#rrggbb`. Values like Lower third's panel `rgba(10,10,12,0.72)`, Glass code block's `rgba(10, 10, 10, 0.6)` or the remocn themes' `oklch(…)` show a black swatch, and picking a colour drops the transparency.

**New and upgraded inputs.** Build these before any node work. Each one gets a story in the Inspector (a node that uses it) and a unit test for its value handling.

| Input            | Schema it handles                               | Widget name | Replaces / used by                                                                                                                                                     | Built on                                            |
| ---------------- | ----------------------------------------------- | ----------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------- |
| Colour (upgrade) | `zColor()` and `color` widget strings           | `color`     | Every colour field. Adds a popover with a saturation area, hue and alpha sliders, eyedropper, hex/rgb/hsl/oklch formats and project swatches, and keeps the text input | Colour picker component (see **Shadcn inputs**)     |
| Palette          | `z.array(zColor()).min(n).max(m)`               | `colors`    | Shader backgrounds, shader transitions, Confetti, Donut chart, Bar chart race. A row of swatches: click to edit, add, remove, drag to reorder                          | Colour picker + sortable                            |
| Text list        | `z.array(z.string())`                           | auto        | Labels, statements, plan lines, integrations, series labels. Chips you add with Enter or by pasting lines, edit in place, drag to reorder                              | Tags input                                          |
| Number list      | `z.array(z.number())`                           | auto        | Chart data without labels. Number inputs in a row with add, remove and paste from a spreadsheet column                                                                 | Existing `NumberField` + sortable                   |
| Rows             | `z.array(z.object({…}))`                        | auto        | Already exists (`ObjectListField`). Upgrade: drag to reorder, duplicate, collapse, and support `code`, `keys`, `asset` and `color` columns                             | Existing `ObjectListField` + sortable               |
| Gradient         | CSS gradient string                             | `gradient`  | Text mask video's fill, Backdrop's `gradient` (`src/shared/nodes/camera.ts:442`, a free-text field today). Linear or radial, angle, 2–6 draggable stops                | Colour picker + a small stop bar written in `form/` |
| Theme            | `z.object` of colour tokens, shown as one group | `theme`     | snapcn and remocn theme tokens. A light/dark switch plus a collapsible list of token colours, with Reset                                                               | Colour field                                        |

Notes:

- **Automatic dispatch.** `schemaKind` gains `stringList`, `numberList` and `colorList` kinds (a `zColor()` element, detected the way `isColor` detects single colours), so plain zod schemas pick the right input without a widget name. Only `gradient` and `theme` need widget names; add both to `FIELD_WIDGETS`.
- **`ObjectListField` columns.** Nested fields inside a row go through the same `FieldControl`, so a `code`, `keys`, `asset` or `color` widget inside a row works with no extra code. Check the row layout at the Inspector's narrowest width.
- **Variables.** Text lists and row text columns render variables as chips (`VarText`), like plain text fields. Never show `{{…}}`.
- **One undo step.** Dragging inside the palette, a sortable list or the colour area calls `beginGesture`/`endGesture`, so a drag is one undo step and not one per pointer move.
- **Colour values.** The colour field keeps whatever format the value is in (`#hex`, `rgb()`, `rgba()`, `hsl()`, `oklch()`, `transparent`, CSS variable names are not allowed) and only rewrites the value when the user picks a colour. Swatches come from colours already used in the project, read from the workflow store, plus the theme's tokens.

## Phase 2: Ingest support

The ingest (`scripts/registry-ingest.ts`, with `scripts/lib/registry-controls.ts` and `scripts/lib/registry-codegen.ts`) builds every registry element's schema from `controls`. It needs these additions:

1. **`colors` control.** `{ type: "colors", default, min, max }` emits `z.array(zColor()).min(min).max(max).default([...])`. The ingest reads the component's own `NEUTRAL_COLORS` / `DEFAULT_COLORS` constant when no default is given, so defaults stay what the element draws today.
2. **`rows` with columns.** `{ type: "rows", columns: { text: { type: "text" }, at: { type: "number", unit: "frames" }, ... } }` emits `z.array(z.object({...}))` built from the column controls (each column through `zodFor`), so rows get the real list editor instead of a JSON box. Rows without `columns` stay an error at ingest, not a JSON fallback. Move Terminal simulator and every remotion-ui `z.record(z.string(), z.unknown())` list to columns in the same change.
3. **Nested props.** `nest: { themeBackground: "theme.background", ... }` in a curated item maps flat settings into a nested prop. The ingest adds a `nested(paths)` wrapper (in `framing.tsx`) that builds the object before the component renders, and drops keys left empty so the component's own default still applies. This covers `theme` objects, Launch anything's `content` and `media`, and Release teaser's `theme`.
4. **Marks from images.** `marks: "logos"` adds a `marksFromImages(prop)` wrapper that turns a rows setting (`image` as an asset, optional `background` as a colour) into the `{ mark: <Img/>, bg }` objects Logo enter expects.
5. **Reasons for hidden props.** `omit` changes from a string list to `Record<prop, reason>`, where the reason is one of `"timing"`, `"function"`, `"react-node"` or `"internal"`, plus free text. The phase 5 check uses it.
6. **Theme presets.** The `theme` widget's light/dark switch maps to the registry's own `defaultLightTheme` / `defaultDarkTheme` (remocn) and snapcn's token sets. The ingest records which file exports them, so the wrapper can merge the chosen preset with the user's token overrides.

Every addition gets a test in the ingest's test file (string output of `zodFor`, like the mix graph tests) and the generated files are regenerated in the same change.

## Phase 3: Registry elements

Defaults are today's values unless the notes say otherwise.

### 3a. Hidden colour palettes (11)

| Element               | Hidden today                               | New settings                                         | Input                                      | Notes                                                                       |
| --------------------- | ------------------------------------------ | ---------------------------------------------------- | ------------------------------------------ | --------------------------------------------------------------------------- |
| Shader mesh gradient  | `colors` (`NEUTRAL_COLORS`, 4)             | Colours (2–10)                                       | Palette                                    | The example you found.                                                      |
| Shader grain gradient | `colors` (`NEUTRAL_COLORS`)                | Colours (2–7)                                        | Palette                                    | Keeps `colorBack` as its own field.                                         |
| Shader warp           | `colors` (`NEUTRAL_COLORS`)                | Colours (2–10)                                       | Palette                                    |                                                                             |
| Shader swirl          | `colors` (`NEUTRAL_COLORS`)                | Colours (2–10)                                       | Palette                                    |                                                                             |
| Shader god rays       | `colors`, `colorBloom` (`#3a3a5c`)         | Colours (1–5), Glow colour                           | Palette, Colour                            |                                                                             |
| Ripple zoom           | `colors`, `speed`                          | Colours, Speed (advanced)                            | Palette                                    | Transition. Check the 5 transitions under software drawing (see **Risks**). |
| Wave wipe             | `colors`, `speed`, `zoom`                  | Colours, Speed and Zoom (advanced)                   | Palette                                    | Transition.                                                                 |
| Smoke dissolve        | `colors`, `speed`                          | Colours, Speed (advanced)                            | Palette                                    | Transition.                                                                 |
| Grain dissolve        | `colors`, `speed`                          | Colours, Speed (advanced)                            | Palette                                    | Transition.                                                                 |
| Swirl dissolve        | `colors`, `speed`                          | Colours, Speed (advanced)                            | Palette                                    | Transition.                                                                 |
| Confetti              | `colors`, `originX`, `originY`, `lifetime` | Colours, Burst from (spot), How long pieces last (s) | Palette, `spot`, number (`unit: "frames"`) | `startFrame` stays hidden (timing).                                         |

Shader dithering's `shape` and `type` are hidden too: add them as advanced enums (`shape`: the values the shader supports; `type`: 2×2, 4×4, 8×8).

### 3b. Hard-coded content (14)

| Element             | Hidden today                                                                                                   | New settings                                                                                      | Input                                                                         | Notes                                                                                                                                                              |
| ------------------- | -------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Glass code block    | `code` (`DEFAULT_CODE`), `aura`                                                                                | Code, Glow behind the window                                                                      | `code` widget, switch                                                         | `DEFAULT_CODE` imports `"remotion"`: the default names a library. Replace it with neutral code in `NEUTRAL_DEFAULTS`.                                              |
| Code morph          | `steps` (`{ at, code }[]`)                                                                                     | Versions: rows of Code + Show at                                                                  | Rows (`code`, number in frames shown as s)                                    |                                                                                                                                                                    |
| Keystroke           | `steps` (`{ at, keys, hold }[]`)                                                                               | Keys: rows of Keys + At + Hold                                                                    | Rows (`keys` widget, numbers)                                                 |                                                                                                                                                                    |
| Agent run           | `run` (title, prompt, plan, steps, answer)                                                                     | Title, Prompt, Plan (text list), Steps (rows: name, detail, length, icon, sources), Answer        | `nest` + text list + rows                                                     | Defaults from `defaultAgentRun`. Check that the copy doesn't name a company.                                                                                       |
| Logo enter          | `logos` (`SAMPLE_LOGOS`: Claude, Codex, Cursor and Grok marks)                                                 | Logos: rows of Image + Background                                                                 | Rows (`asset`, `color`) via `marksFromImages`                                 | The current default shows real companies' brand marks, which breaks the neutral-defaults rule. Default to `app:placeholders/logo-mark.svg` in 3–4 neutral colours. |
| Check list          | `items` (required, no default), `boxColor`, `weight`, `rowGap`, `seed`                                         | Items (rows: text, done), Box colour, Font weight, Row gap (advanced), Seed (advanced)            | Rows, colour, enum, number                                                    | `items` is required but neither exposed nor given a default, so `normalizeCheckListItems(undefined)` most likely throws on every render. Fix this first.           |
| Animated bar chart  | `data`, `labels`                                                                                               | Bars: rows of Label + Value                                                                       | Rows                                                                          | Map rows to the two props in a wrapper, or keep two lists if the component needs them apart.                                                                       |
| Animated line chart | `data`                                                                                                         | Values                                                                                            | Number list                                                                   |                                                                                                                                                                    |
| Signup flow         | Everything: title, description, name, email, password, button, toast, the "Sign in" and Google labels, `theme` | One text setting each, plus Theme                                                                 | Text, `theme` widget                                                          | Has no settings at all today. "Google" in the button label names a company: neutral default "Continue with email" or hide that button.                             |
| AI prompt flow      | Everything: prompt, button, answer lines, toast, `theme`                                                       | Prompt, Button label, Answer (text list), Toast title, Theme                                      | Text, text list, `theme`                                                      | Has no settings at all today.                                                                                                                                      |
| Command menu        | `items` (icon, label, shortcut), `pressedIndex`, `itemStyles`, `theme`                                         | Items (rows: icon, label, shortcut), Press item, Theme                                            | Rows (icon as an enum of lucide names in `icons.ts`, `keys`), number, `theme` | `itemStyles` stays hidden (`react-node`).                                                                                                                          |
| Release teaser      | `statements`, `logoSrc`, `theme`, `lightIntensity`, `audioSrc`, `volume`                                       | Statements (text list), Logo (asset), Theme colours, Light (advanced), Music (sound) and Volume   | Text list, asset, `theme`, number, `sound`                                    | Sound through the `sound` widget so it plays in the Mixer like other clips. Check how its audio meets the mix (see **Risks**).                                     |
| Launch anything     | `content` (25 lines of copy and a list), `media`, `screenImages`, `logos`, `logoSrc`, `audioSrc`, `volume`     | Copy grouped by scene (advanced groups), Screens (asset per showcase), Logos, Logo, Music, Volume | `nest`, text, text list, asset, rows, `sound`                                 | 25 copy fields is a lot: show the 4 most visible in the main list and the rest under Advanced, grouped by scene.                                                   |
| Spring settle       | `scenes`, `config`, `loop`                                                                                     | Loop (switch)                                                                                     | Switch                                                                        | `scenes` and `config` are React nodes and spring tuning: omit with reasons.                                                                                        |

### 3c. Fixed colours and looks (4)

| Element              | Hidden today                                                    | New settings                         | Input          | Notes                                                                                                                                                              |
| -------------------- | --------------------------------------------------------------- | ------------------------------------ | -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Polaroid             | `frameColor` (`#fdfcf8`), `captionColor` (`#26242c`)            | Frame colour, Caption colour         | Colour         |                                                                                                                                                                    |
| Text mask video      | `gradient` (gold, pink, teal)                                   | Gradient fill                        | Gradient       | Only shown when the fill is a gradient (`showIf`).                                                                                                                 |
| snapcn elements (10) | `fixedProps: { mode: "dark" }` on every one, and `theme` tokens | Theme (light/dark) and Theme colours | `theme` widget | Light mode is impossible today. Hero launch has no colour setting at all; this gives it one. Laptop frame also gets its recording light colour (`indicatorColor`). |
| snapcn Logo assemble | `images` fixed to 8 backdrops                                   | Images                               | Rows (`asset`) | Defaults stay the 8 backdrops.                                                                                                                                     |

### 3d. Background colour missing (12 remotion-ui scenes)

Stat card, Pricing card, Comparison table, Changelog entry, Logo wall, End card, Code reveal, Code diff wipe, Kanban move, File tree reveal, Drag-drop flow, Commit graph.

Each has `theme` (light/dark) but no `backgroundColor`, which falls back to the theme's page colour. Add **Background** (colour, empty meaning "from the theme") with the help text "Leave empty to follow the theme." The colour field needs an empty state for this ("Theme"), shared with the theme widget's Reset.

Also in remotion-ui:

- Changelog entry: `kindColors` (the colour of each change kind) as rows of Kind + Colour.
- Pricing card: `wasPrice` (the struck-out price) as text.
- End card: `logoSize`; Logo reveal: `size`, `strokeWidth`, `fill`.
- Multi-device lineup: the fixed device labels (Phone, Tablet, Laptop) as text settings.
- Stat card: `max`.

### 3e. Font weight (minor, 6)

Word push, Gradient scale cut text, Punch lines, Text highlight, Status cycle (and its chip weight), Logo drift. Add **Font weight** as the same 400–700 enum the other text elements use.

### Hidden on purpose

Give these an `omit` reason so phase 5's check passes: `delay`, `holdSeconds`, `delayInFrames`, `exitAtInFrames`, `exitInFrames`, `startFrame` (`timing`); `valueFormatter` (`function`); `springConfig`, `motion`, `exitScale`, `exitBlur`, `wordAt`, `release` (`internal`); `itemStyles`, `scenes`, `logo` in Text highlight (`react-node`); Ink arrow's and Speed lines' `to` (`internal`, the points wrapper sets it); Screen recording's `crop`, `sourceAspect` and `camera` (`internal`, the recording sets them).

## Phase 4: Demoflow's own nodes

These are in code Demoflow owns, so they change directly.

| Node / element     | Hard-coded today                                                                                                                   | New settings                                                           | Where                                                                                       |
| ------------------ | ---------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| Media file         | Images last 3 s (`IMAGE_MS`, `src/shared/nodes/media.ts:112`); with no length known, 5 s (`MEDIA_MS`)                              | Length (s), shown only for images (`showIf` on the picked file's kind) | `importMediaNode` params and `compile`; keep a length handle so the Timeline can trim it    |
| Cursor             | Arrow fill `#111418` and outline `#ffffff` (`src/video/camera/cursor.tsx:88`)                                                      | Colour, Outline colour                                                 | `cursor.style` params → `RecordingProps` → `cursor.tsx`                                     |
| Click ripple       | Ink colour, size (`RIPPLE_D`) and stroke (`RIPPLE_STROKE`) in `cursor.tsx`                                                         | Colour, Size (×), Ring width (advanced)                                | `cursor.clickRipple` params, `ripple` becomes an object in `RecordingProps`                 |
| Keystroke chips    | Keycap ground `#0a0a0a` and ink `#ffffff` (`src/video/camera/keycap-hud.tsx:17`), size                                             | Keycap colour, Text colour, Size (×)                                   | `cursor.keystrokeChips` params → `keystrokes` → `KeycapHUD`                                 |
| Tooltip            | Text is always `TEXT_COLOR` (unreadable on a light box), font size 18, weight 600, no font (`src/video/camera/annotations.tsx:79`) | Text colour, Font size, Font                                           | `annotationNode` params (shared by all four annotation nodes, shown per kind with `showIf`) |
| Callout arrow      | Font size 30, weight 800, no font (`annotations.tsx:146`)                                                                          | Font size, Font                                                        | Same                                                                                        |
| Show captions      | Panel `rgba(0,0,0,0.35)`, shadow and weight 800 (`src/video/core/captions.tsx:20`)                                                 | Panel colour (with alpha), Font weight                                 | `captionsSchema` in `src/shared/elements/core.ts` and `Captions`                            |
| Title card element | Font weight fixed                                                                                                                  | Font weight                                                            | `core/title` schema and component                                                           |

The Spotlight, Highlight box, Tooltip and Callout arrow share `annotationNode`; add the new fields there with `showIf` on `kind`, so each node only shows its own.

The full-page styles' page and bleed colours (`PAGE_BG`, `BLEED_BG`, `src/video/camera/demo-clip.tsx:55`) and the empty-clip placeholder colours stay as they are: they're internal fallbacks that a user never sees in a finished film. Write that down in the phase 6 docs so the next audit doesn't flag them.

## Phase 5: Checks that stop regressions

1. **Ingest check for hidden props.** After resolving an element's controls, the ingest reads the component's props (the `export type …Props` / `interface …Props` block and the destructured defaults, with ts-morph, which the ingest already uses) and fails when a prop is neither a control, a fixed prop, nor in `omit` with a reason. The failure names the element, the prop and its default. The audit script used for this plan did the same with regular expressions and found every case in phase 3.
2. **Defaults hidden in React nodes.** `defaultProblems` also renders each element's default marks and text props to static markup (`react-dom/server`, in the ingest only) and runs `namesLibrary` over the text and `aria-label`s, so a brand logo default (Logo enter) fails the ingest.
3. **Required props have a value.** The ingest fails when a required prop (no `?`) has neither a control, a fixed prop nor a default. That catches Check list's missing `items`.
4. **Colour literals in Demoflow's video code.** A test scans `src/video/core/**` and `src/video/camera/**` for colour literals (`#…`, `rgb(`, `hsl(`, `oklch(`) outside an allow-list of named internal constants (shadows, fallbacks, debug colours) kept in the test. A new literal must either become a setting or be added to the list on purpose.
5. **Every element renders with its defaults.** A script (not part of `bun run test`, since shader elements need WebGL) renders the middle frame of every element with its defaults through the card-still worker (`src/worker/render/card-still.ts`) and fails on any render error. Run it after every ingest. This also catches the Check list crash.

## Phase 6: Docs

- `AGENTS.md`, under **Params carry their own inspector UI**: add one paragraph with the rule ("A value a user would want to change is a setting; only timing owned by the Timeline, animation tuning, functions and React nodes stay in code, and hidden registry props carry an `omit` reason") and the new widgets (`colors` handled automatically, `gradient`, `theme`).
- `AGENTS.md`, widget list: add `gradient` and `theme`.
- This plan: mark each phase done as it lands.

## Shadcn inputs

The project uses shadcn with **Base UI** (`components.json` style `base-nova`); Base UI has been shadcn's default since July 2026. Installed today: `field`, `input`, `input-group`, `select`, `slider`, `switch`, `toggle-group`, `textarea`, `popover`, `command` and others. Neither shadcn nor Base UI (v1.9.0) has a colour picker, tag input, sortable list or gradient picker, so those come from community registries or are written in `src/renderer/editor/form/`.

### Recommendations

| Need             | Recommended                                                                                                   | Why                                                                                                                                                   | Alternatives                                                                                                                                                                                                                                                                                |
| ---------------- | ------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Colour picker    | **Dice UI Color Picker** (`bunx --bun shadcn@latest add @diceui/color-picker`, MIT, about 2.1k stars)         | Composable parts: area, hue and alpha sliders, eyedropper, format select (hex, rgb, hsl) and input; installs from the shadcn registry with no API key | **Kibo UI Color Picker** (MIT, about 4k stars, Figma-style with hue and alpha sliders and eyedropper; its shadcn install goes through 21st.dev and needs an API key); **Amplo** (`@amplo`, OKLCH fill picker, useful because remocn themes are OKLCH); react-aria-components colour pickers |
| Tags / text list | **Dice UI Tags Input** (`@diceui/tags-input`, MIT)                                                            | Add, edit in place and remove; `addOnPaste`, `addOnTab`, `editable`, validation; pairs with Dice UI Sortable for reordering                           | **shadcn Combobox** with `multiple` and chips (official, Base UI, but picks from a fixed list; fine for Font weight-like choices, not free text); **Emblor** (TagInput with autocomplete and drag reordering, built on shadcn's Radix parts)                                                |
| Sortable lists   | **Dice UI Sortable** (`@diceui/sortable`, MIT), which has a Base UI version (`@base-ui/react`, `@dnd-kit/*`)  | Item, handle and overlay parts; keyboard sorting; one dependency set for palette, text list, number list and rows                                     | **ReUI Sortable** (dnd-kit `SortableContext` / `useSortable`, with a drag handle); 21st.dev sortables (need an API key)                                                                                                                                                                     |
| Number input     | Keep Demoflow's `NumberField`; consider the **Base UI Number Field** primitive (already a dependency)         | Base UI's Number Field adds increment/decrement buttons and a scrub area, the same thing coss ui and ReUI wrap                                        | **coss ui Number Field** (Base UI based); **ReUI Number Field** (Base UI or Radix)                                                                                                                                                                                                          |
| Gradient picker  | **Write it in `form/`** on top of the colour picker                                                           | The only open shadcn gradient picker (Illyism) was last updated in February 2024 and is Radix-based; a stop bar plus angle is small                   | Pasta UI colour picker on 21st.dev (linear, radial and conic gradients with draggable stops; needs a 21st API key)                                                                                                                                                                          |
| Theme tokens     | **Write it in `form/`**: shadcn **Collapsible** (installed) + the colour field + **Toggle group** (installed) | Only composes installed parts                                                                                                                         | none needed                                                                                                                                                                                                                                                                                 |

Official shadcn components worth adding for these inputs (all Base UI):

- **Combobox** (`bunx --bun shadcn@latest add combobox`): searchable picker for Command menu icons (lucide names) and long enums; the `multiple` mode with chips for fixed multi-choice lists.
- **Radio group**: small visual choices (gradient type, light/dark when a toggle group is too wide).
- **Button group**: the palette's add/remove/reverse buttons and the rows editor's per-row actions.
- **Item**: rows in the list editors, so collapsed rows look like the rest of the Inspector.

Before installing any community component:

1. Install it into a scratch branch and run `bun run typecheck` and `bun run lint:fix` with it as-is. If it needs edits, write the equivalent in `src/renderer/editor/form/` instead.
2. Prefer the Base UI version where the registry has one (Dice UI publishes `base` variants under `/docs/components/base/`; check the colour picker and tags input have one, since the docs found only confirm it for Sortable). Don't add Radix as a dependency for one input.
3. Check its licence (all recommended ones are MIT) and add it to `THIRD_PARTY_NOTICES.md`.
4. Make sure it never loads anything from the internet (no CDN fonts or colour-name lists).
5. Check it in the dark editor theme: panels on `bg-sidebar`, pressed states on `--toggle-on`.

## Risks

- **Shader transitions under software drawing.** The five shader transitions and shader backgrounds already need checking with the export's software renderer; more colours don't change cost, but check one export of each with the **Draw with** setting switched.
- **Sound inside elements.** Release teaser and Launch anything play their own audio inside the picture. An exposed music setting must go through the mix (`withMasterStem`) like every other sound, or the picture/sound split breaks. If that's more than a wrapper, leave their audio off (fixed empty) and tell users to use a Music node.
- **Launch anything's size.** 25 copy fields plus media is a large Inspector. Group by scene under Advanced and keep the four main lines on top.
- **Ingest prop reading.** Reading props with ts-morph must cope with `extends Omit<…>` (the shader elements) and props spread through helpers. Where the type can't be read, the element must say so explicitly in the list (`props: "unchecked"`) rather than pass silently.
- **Default changes.** Glass code block, Logo enter and Signup flow get new neutral defaults, so their look changes. Write `examples/` again.

## Checks

For each phase:

- `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`.
- The no-comments check from `AGENTS.md`.
- `bun run registry:ingest -- --no-add` after any change to the list or the ingest, and no hand edits in generated files.
- Open each changed node in the Inspector, change every new setting, and check the Preview follows. Export a short clip with changed settings and compare it with the Preview: preview must equal render.
- Don't commit or push.

## Sources

- [shadcn/ui components](https://ui.shadcn.com/docs/components) and [changelog](https://ui.shadcn.com/docs/changelog) (Base UI default since July 2026; no colour, tags or sortable components)
- [shadcn Combobox](https://ui.shadcn.com/docs/components/combobox) (Base UI, `multiple` with chips)
- [shadcn registry index](https://ui.shadcn.com/r/registries.json)
- [Base UI releases](https://base-ui.com/react/overview/releases) (v1.9.0; Number Field, Slider, Combobox, Autocomplete; no colour component)
- [Dice UI Color Picker](https://www.diceui.com/docs/components/color-picker), [Tags Input](https://www.diceui.com/docs/components/tags-input), [Sortable (Base UI)](https://diceui.com/docs/components/base/sortable), [repository](https://github.com/sadmann7/diceui)
- [Kibo UI components](https://www.kibo-ui.com/components), [Color Picker on 21st.dev](https://21st.dev/@haydenbleasel/components/color-picker.md), [repository](https://github.com/haydenbleasel/kibo)
- [Emblor](https://github.com/JaleelB/emblor)
- [ReUI Sortable](https://reui.io/docs/components/radix/sortable), [ReUI Number Field (Base UI)](https://reui.io/docs/components/base/number-field)
- [coss ui](https://coss.com/ui) (Base UI based: Number Field, Autocomplete, Combobox, Input Group)
- [Illyism gradient picker](https://shadcn.io/template/illyism-gradient-picker), [Pasta UI colour picker with gradients](https://21st.dev/@syeddhasnainn/components/color-picker.md)
- [21st.dev shadcn colour pickers](https://21st.dev/community/components/explore/shadcn-color-picker)
