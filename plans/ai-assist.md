# AI assist

Buttons in the editor that suggest wording and names where a person is already working: headline copy, chapter titles for a take, and clearer names for picked elements. The assistant itself (the Agent panel, agents, their tools and logs) is in [assistant.md](assistant.md). Every feature has its own section with the same parts, so the plan can grow one section at a time.

Status: plan only. Nothing here is built yet.

## How to read this plan

- Each section is one feature and can be built on its own.
- Each section has the same parts: **What it adds**, **Fit with Opsis** (In place / Missing), **Work**, **Decisions**, **Risks**.
- Rules from `AGENTS.md` and `agent-docs/` hold throughout; when they disagree with this plan, they win.

## Conventions

- **Suggestions are never trusted.** Each suggestion is parsed into a flat Rust struct, its limits enforced after parsing, and every workflow change goes through `check_workflow` / `workflow_problems` (`crates/compile/src/check.rs`), the same checks the editor and the Local API use.
- **Applying a suggestion is one undo step**, written like a typed edit.
- **User-facing copy:** no library, tool, model or service names; sentence case; browser nodes are "actions".
- **Checks when a feature is done:** `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, then the app itself (`check-app`).

## Summary

| Feature                | What users get                                                                    | Status    |
| ---------------------- | --------------------------------------------------------------------------------- | --------- |
| Suggest chapters       | Chapter titles for a recorded take, from what it did, inserted as Chapter actions | not built |
| Headline copy variants | A list of alternative wordings next to title, headline and caption fields         | not built |
| Picker suggestions     | After picking an element: a clearer, unique name for it and a name for the action | not built |

## Suggest chapters

### What it adds

On a Record take that has run: **Suggest chapters**. Opsis splits the take where it moves to a new page or pauses, each part gets a title ("Create a project", "Invite a teammate"), and the user edits the titles and inserts them as Chapter actions and Timeline markers. Export video already writes chapters from Chapter actions into the file and the optional YouTube list (**Chapters in exports** in `agent-docs/timeline.md`). Unnamed actions get suggested names in the same dialog, since an action's name is also its camera, chapter and caption label.

### Fit with Opsis

In place:

- The action log: `ActionLog` (`crates/types/src/actions.rs`), carried in a take's result as `RecordingValue.log` (`crates/types/src/values.rs`), with each action's kind, time, label, text and node id.
- The Chapter action `browser.chapter` (`BrowserAction::Chapter`, `crates/types/src/values.rs`); chapter marks on the Timeline from `flatten_spec`; `film_chapters` (`crates/compile/src/timeline/chapters.rs`) for exports.
- `session_into` (`crates/compile/src/graph.rs`) gives a capture's session with the chain's steps in order.

Missing: splitting the log into parts, the titles, inserting Chapter nodes into a chain.

### Work

1. **Split.** `chapter_spans(log, session)` in `opsis-compile` (pure, property-tested): cut at navigations, existing chapters and gaps longer than a threshold; each span lists its actions' kinds, labels and node ids.
2. **Titles.** One title and one short description per span, keyed by the span's first node id; a title for a node id that isn't a span start is dropped. Typed text is left out unless the user ticks "Include typed text".
3. **Insert.** `insert_chapters(workflow, capture_node_id, chapters)` (pure): a `browser.chapter` node before each start node, rewiring the session edges; an existing Chapter at that spot is renamed instead. One update, then `check_workflow`.
4. **UI.** A button on the Record take's Inspector when it has a result; a dialog with editable titles, "Show a chapter card in the video" off by default, and a note that the next run records the take again.
5. **Tests.** Spans from golden logs; `insert_chapters` keeps one chain, keeps the session's step order and passes `check_workflow`.

### Decisions

- **Outputs (decided):** Chapter actions and Timeline markers, with file chapters and YouTube timestamps from the existing export.
- **Name unnamed actions (decided): yes, as suggestions** in the same dialog.

### Risks

- **Typed text can be private** (emails, test passwords): left out by default, and text typed from a variable that names a saved secret is never included.
- Inserting chapters changes node keys downstream, so the take records again on the next run.

## Headline copy variants

### What it adds

A sparkles button next to headline text fields (Title card's eyebrow, title and subtitle; Animated headline's text; other text elements; caption lines). It opens five alternatives in a chosen tone (Plain, Bold, Playful, Technical); clicking one applies it as one undo step, **More** gives five others.

### Fit with Opsis

In place:

- `title` and `kinetic-headline` (`crates/compile/src/elements/text/title.rs`, `kinetic_headline.rs`); the headline keeps its `*asterisks*` emphasis rule.
- Fields carry their UI as `x-` keywords in their JSON Schema and the Inspector builds forms from it (`crates/ui/src/inspector.rs`, `crates/qt/cpp/inspector_panel.cc`); `sparkles` is in the card icons.

Missing: a way to mark a field as copy, the variants, the button.

### Work

1. **Mark fields.** An `x-suggest` keyword (`headline`, `subtitle`, `eyebrow`, `caption`) on the core elements' fields and Captions from script's lines; registry text elements' structs get it the same way.
2. **Variants.** `{ variants: Vec<String> }` from the current text, the field's role, the tone, the project name, the site address, chapter titles and action names. Variants stay within the current length plus a margin, checked after parsing; the asterisks rule applies only to the headline.
3. **UI.** A button on fields with `x-suggest`; a popover list; applying writes the setting like a typed edit.
4. **Tests.** The asterisks rule is kept only for the headline; over-long variants are dropped.

### Decisions

- **Captions too (decided):** the button is also on caption text (Captions from script's lines).
- **Tones (decided):** Plain, Bold, Playful and Technical.

### Risks

- A longer variant can wrap; keep the length check.

## Picker suggestions

### What it adds

After picking an element, the Inspector offers a suggestion when the picked name is weak: "This text appears 3 times on the page. Use “Save invoice” (button) instead?", plus a name for the action ("Save the invoice") when it has none.

### Fit with Opsis

In place:

- The picker is a streamed view of the headless shell (`crates/stream/src/picker.rs` with its page script `picker.js`), returning `PickResult` (the target name, its role, a CSS fallback). A nameless element falls back to `css=…`.
- The Inspector's target field stores the name and its role; cards show it through `element_name` (`crates/compile/src/card_rows.rs`).

Missing: alternative names, how many elements share each, nearby context.

### Work

1. **Candidates.** In `picker.js`, collect every name source for the picked element (aria-label, label, placeholder, visible text, test id, title) and count elements with the same role and name; add the nearest heading or landmark text. `PickResult` gains `candidates: [{ name, role, source, count }]` and `context`.
2. **Suggest.** Prefer a unique candidate in the selector ladder's own order (role and name, label, placeholder, test id, text); show it under the target field with Use / Dismiss. A `css=` pick with a unique name candidate always gets a suggestion.
3. **Clearest name and action name.** `{ target, label }`, `target` one of the unique candidates; an action name is offered when the node has none.
4. **Tests.** Candidate ordering, and that a suggested target is always one of the counted candidates.

### Decisions

- **Button (decided):** suggestions appear only on a "Suggest a better name" button.

### Risks

- The in-page count approximates the selector ladder; the run is still the real test.
