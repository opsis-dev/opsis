# AI assist

How Opsis adds an assistant: a model the user chooses (on this computer or online) that drafts a flow from a sentence, names chapters, suggests headline wording and better names for picked elements. Every feature has its own section with the same parts, so the plan can grow one section at a time.

Status: plan only. Nothing here is built yet.

## How to read this plan

- **Model access** comes first: every other feature depends on it. The other four can be built in any order after it, and each can ship on its own.
- Each section has the same parts: **What it adds**, **Fit with Opsis** (In place / Missing), **Work**, **Decisions**, **Risks**, **Sources**.
- External facts link their source and the date they were checked.

## Conventions

These restate rules from `AGENTS.md`; when the two disagree, `AGENTS.md` wins. When a feature ships, add its rules to `AGENTS.md` (an "Assistant" paragraph next to "Local API").

**Privacy.**

- Nothing contacts a model until the user turns the assistant on in Settings. Off is the default, as with the Local API.
- An online model is asked only after the user agrees to that request: a dialog shows exactly what will be sent. A model on this computer needs no per-request dialog.
- Opening the user's site in the hidden browser happens only when they press a button that says so, like a run.
- Keys live in the OS keyring (`Keyring` in `crates/store/src/keys.rs`) and never reach a window.

**Where code goes.**

- `crates/assist` (`opsis-assist`): the only code that talks to a model, with `reqwest`; consent tokens, the request log and keys go through it.
- `opsis-compile`: the task schemas, prompt builders and the pure functions that turn a model's answer into nodes (`draft_to_workflow`, `chapter_spans`, `insert_chapters`), next to the node definitions they create and checked with `check_workflow`.
- `opsis-capture`: the page outline from the hidden browser.
- `opsis-desktop`: the commands and events; the editor (`apps/ui`) has the dialogs and the suggestion UI only.

**Model output is never trusted.** Each task's answer is a flat Rust struct deriving `schemars::JsonSchema`; the answer is checked with `jsonschema`, parsed with `serde`, and its limits enforced. Every workflow change then goes through `check_workflow` / `workflow_problems` (`crates/compile/src/check.rs`), the same checks the editor and the Local API use. A model never writes settings or edges directly: it fills a small draft that Opsis's code turns into nodes.

**User-facing copy.** No library, tool, model or service names anywhere, with one decided exception added to `AGENTS.md` when this ships: Settings → Assistant may name Anthropic and OpenAI in its provider choice and key field, and its circled `?` hover cards may name local model programs. The feature is called "Assistant". Sentence case. Browser nodes are "actions".

**Checks when a feature is done.** `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`; try it in the app with a local and an online model, and with the assistant off (no request leaves the app, the buttons are hidden).

## Summary

| Feature                | What users get                                                                                                 | Needs                                 | Status    |
| ---------------------- | -------------------------------------------------------------------------------------------------------------- | ------------------------------------- | --------- |
| Model access           | Settings → Assistant: a model on this computer or an online one with a key in the keyring; consent per request | nothing                               | not built |
| Prompt → flow          | Describe the demo in a sentence plus the site address; get actions, Record take, Auto zoom, a title and Export | Model access, engine (hidden browser) | not built |
| Auto-chapters          | Chapter titles for a take, from what it did, inserted as Chapter actions                                       | Model access, a recorded take         | not built |
| Headline copy variants | A list of alternative wordings next to title and headline fields                                               | Model access                          | not built |
| Picker suggestions     | After picking an element: a clearer, unique name for it and a name for the action                              | picker change (works without a model) | not built |

## What the Local API already gives outside agents

The Local API (Settings → API, `crates/api`) already lets an agent on the same computer build a flow with no in-app model: `GET /openapi.json`, `GET /nodes` and `GET /nodes/{type}` (`node_catalog`, `catalog_entry`, the settings as JSON Schema), `POST /validate` (`check_workflow`), edits under `/project` (each one undo step), and `POST /project/run` with events on `/project/run/events`. An outside agent inspects the site with its own browser.

The assistant reuses those pieces rather than a second path: the catalogue for node titles, descriptions and settings, `check_workflow` for validation, and one project update for the insert. What it adds is a page outline from Opsis's own hidden browser and a model the user doesn't have to drive themselves.

## Model access

### What it adds

A Settings page, **Assistant**, with three choices: Off (default), "A model on this computer", "An online model". Every feature asks through one Rust function, so the privacy rules live in one place.

- **On this computer: a local server.** Any server that speaks the OpenAI-compatible chat API on a loopback address. Ollama (MIT, `http://localhost:11434`), llama.cpp's `llama-server` (MIT) and LM Studio (`http://localhost:1234/v1`) all do, and all accept a JSON schema to shape the answer. The user installs and runs the server; Opsis stores its address and model name, lists its models and has a **Test** button.
- **No model inside Opsis (decided).** Opsis ships no model runtime and downloads no models.
- **Online.** A provider, a model name and an API key. Anthropic can force an answer to a JSON schema through `output_config.format` (type `json_schema`) and strict tool use, OpenAI through `text.format` with `strict: true`. "Compatible service" takes an address too; OpenRouter and similar services go there, with no preset of their own (decided).

### Fit with Opsis

In place:

- Settings: `AppSettings` (`crates/types/src/settings.rs`), one section per area, checked with `garde`, written one section at a time with `patch_settings` and followed in every window through `useAppSettings`; `SettingsChanged` reaches every window.
- Settings pages: `SettingsPage` (`crates/types/src/shell.rs`) and `apps/ui/src/app/settings/`; the API page is the model for an off-by-default switch with a status badge.
- Keys: `Keyring::set_key` / `key` (`crates/store/src/keys.rs`) keep named secrets in the OS keyring and refuse when the platform has no secure store, with a plain message.
- Asking before going online: the "Go online to get fonts?" dialog (`apps/ui/src/fonts/fonts-panel.tsx`) in front of `opsis-fonts`'s online fetch.
- Commands are limited to the app's own windows by capabilities (`crates/desktop/capabilities`); long work is listed in the exit dialog through `AppState.tasks`.

Missing: an `assistant` settings section, the page, the client, consent, the request log, a key removal call.

### Work

1. **Settings.** An `assistant` section in `AppSettings`: `mode` (`off`, `local`, `online`; default `off`), `local_address` (default `http://127.0.0.1:11434/v1`), `local_model`, `provider` (`anthropic`, `openai`, `compatible`), `online_address`, `online_model`, `ask_each_time` (default `true`).
2. **Keys.** `Keyring::clear_key`; keys named `assistant:<provider>`. Commands `save_assistant_key`, `has_assistant_key`, `clear_assistant_key`; the key itself is read only inside `opsis-assist`. Windows only ever learn `has_key`.
3. **Client.** `ask_model(task, input, schema, cancel)` in `opsis-assist` returns the parsed struct or a plain error, choosing the provider: local server (chat completions with `response_format` and a JSON schema), Anthropic (Messages API with `output_config.format`), OpenAI (Responses API with `text.format`, `strict: true`), compatible (as the local server, at `online_address`). It refuses when `mode` is `off`, refuses an online request without a consent token, and treats a non-loopback `local_address` as online. Each call is listed in `AppState.tasks` ("Waiting for the assistant") and cancellable.
4. **Schemas every provider accepts.** Strict online modes reject `minimum`/`maximum`, string lengths and optional-heavy objects (Anthropic: at most 24 optional parameters per request). Each task's struct is flat with every field required (empty string for "none"), and limits are enforced after parsing. A test checks every task schema for unsupported keywords.
5. **Consent.** For an online request the editor first shows `AssistConsentDialog`: what will be sent (the exact text, expandable), Send / Cancel, and "Don't ask again", which turns `ask_each_time` off app-wide (decided) and can be turned back on in Settings. Rust issues a one-time token per approved request and refuses online requests without one.
6. **Page.** `assistant-page.tsx` and `SettingsPage::Assistant`: the three-way choice, address and model fields, provider choice and key field (Save / Remove; never shown back), **Test** (one tiny request; online through the consent dialog), "Ask before every online request". "A model on this computer" has a circled `?` hover card naming programs known to work and their usual addresses.
7. **Indicator.** An "Assistant" badge in the app bar while `mode` is `online`, like the API badge.
8. **Request log.** An `assistant_log` table in the store's schema: `id`, `at`, `project_id` (null when no project is open, set to null when the project is forgotten), `task`, `service`, `model`, `request` and `response` JSON, `error` (null on success), `ms`. `ask_model` writes an entry after every request; keys and headers are never logged. Settings → Assistant lists the entries newest first with what was sent and what came back, and has **Clear log**. Keep the last 1,000 entries or 30 days, whichever is smaller, pruned at start.
9. **Tests.** The client refuses without the switch, without consent and on a non-loopback local address (against `wiremock`); keys never appear in command results or events; log entries round-trip, nothing secret is stored, pruning keeps the newest.

### Decisions

- **Naming providers (decided).** Anthropic and OpenAI in the provider choice and key field; OpenRouter under Compatible service; local programs only in the `?` card.
- **No built-in model (decided).**
- **"Don't ask again" (decided).** Allowed, app-wide.

### Risks

- **Linux without a Secret Service** can't keep keys; offer pasting the key per session, kept in memory only.
- **Small local models** follow schemas but plan poorly: every task fills a small draft and Opsis's code builds the structure.
- **A local server is still a server**: a non-loopback address always goes through online consent.
- **Cost and limits** of online models belong to the user; show the model's error plainly and retry at most once.

### Sources

- [Ollama: structured outputs](https://docs.ollama.com/capabilities/structured-outputs), [Ollama repository (MIT)](https://github.com/ollama/ollama) (checked 2026-10-16)
- [llama.cpp server README](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md) (OpenAI-compatible chat, `response_format` with JSON schema; checked 2026-10-16)
- [LM Studio: OpenAI compatibility](https://lmstudio.ai/docs/developer/openai-compat) (checked 2026-10-16)
- [Anthropic: structured outputs](https://platform.claude.com/docs/en/build-with-claude/structured-outputs), [OpenAI: structured outputs](https://developers.openai.com/api/docs/guides/structured-outputs) (checked 2026-10-16)
- crates.io: `reqwest` 0.13.5, `schemars` 1.2.3, `jsonschema` 0.60.1; `async-openai` 0.42.2 (MIT) if typed OpenAI requests are wanted; there is no official Anthropic Rust SDK, so the Messages API is called directly (checked 2026-10-25)

## Prompt → flow

### What it adds

On an empty canvas, in the New project dialog and in the Edit menu: **Describe a demo…**. The user writes one sentence ("Sign up, create a project and invite a teammate") and the site address. Opsis opens the page in the hidden browser, takes an outline of what's on it, and asks the model for a draft. The user reviews the draft as a list of actions, edits it, and inserts it: Open browser, Go to page, the actions, Record take, Auto zoom, a Title card, a Sequence and Export, as one undo step.

### Fit with Opsis

In place:

- The hidden browser: `Session::open` (`crates/capture/src/browser.rs`) launches the headless shell with Open browser's settings and talks CDP; a take applies a saved sign-in through its `SignIns` (`crates/capture/src/take.rs`).
- Selectors: `crates/capture/src/selectors.rs` and `locators.rs` find an element by visible name and role; targets are stored as a name plus its role setting (`targetRoleKey` in the editor).
- Takes: `crates/capture/src/take.rs` plays a session's steps; a check take would be one more caller with no recording.
- The catalogue (`node_catalog`, `catalog_entry`) gives titles, descriptions and settings; `check_workflow` validates; Title card is `fx.core.title`; the templates in `crates/project/src/templates.json` show the target shape.

Missing: the outline, the draft schema, the draft-to-nodes mapping, the dialog. The full catalogue is about 400 KB and its schemas use keywords strict modes reject, so it can't be the model's schema as it is.

### Work

1. **Page outline.** In `opsis-capture`: open the page with `Session::open` (the project's Open browser settings, plus its saved sign-in applied as a take does), wait for load, read `Accessibility.getFullAXTree`, keep landmarks and named controls, cut to a fixed size, and give each kept node a ref (`e1`, `e2`…) with its role and name. agent-browser's snapshot code (`cli/src/native/snapshot.rs`, Apache-2.0) is a working model of the same idea. Returns the outline text, the ref table, the page title and the final address; a command `page_outline` runs it in the background like analysis.
2. **Draft schema.** `FlowDraft { title, subtitle, actions: Vec<DraftAction { act, ref_id, text, address, label }> }`, every field required. `act` is an enum of draft actions mapped to node types (`browser.navigate`, `browser.click`, `browser.type`, `browser.hover`, `browser.scroll`, `browser.press`, `browser.select`, `browser.wait`, `browser.chapter`); `ref_id` is an enum of the outline's refs built per request, or empty.
3. **The catalogue as the model's tools.** The prompt lists each draft action with its node's title and description from the catalogue, so new wording in a node definition reaches the model. A test checks every draft action maps to a visible node type and that the settings `draft_to_workflow` writes pass that node's schema.
4. **Draft to nodes.** `draft_to_workflow(workflow, draft, outline)` (pure, in `opsis-compile`): refs become a target name plus role; the first action is Go to page with `{{Site address}}`; then `capture.record`, `camera.autoZoom`, `fx.core.title` with the title, `timeline.sequence`, `output.render`, placed to the right of existing nodes with the same placement the Local API uses for new nodes. Sets the "Site address" variable when it's empty.
5. **Check and repair.** Run `check_workflow` on the result; if the new nodes have problems, send them back once; anything left shows in the review list.
6. **Review dialog.** `describe-dialog.tsx`: sentence and address, progress ("Opening the page", "Writing the actions", "Checking"), then the draft as editable rows with the node icons. Rows whose target wasn't on the first page are marked "Not seen on the page". **Insert** applies one update, selects the new nodes and fits the view.
7. **Entry points.** A menu command `edit:describe` in `app_menu` (`crates/desktop/src/menu.rs`), the empty-canvas button, and "Describe your demo" in the New project dialog. Hidden when the assistant is off; without the engine, offers to set it up.
8. **Try before inserting.** "Try the actions" plays the new chain as a check take (no recording) and reports targets that didn't resolve, with what was visible.
9. **Walk the site.** For flows across pages: the model picks one action at a time from a fresh outline, the take performs it, and only actions that worked become the draft.
10. **Tests.** `draft_to_workflow` on fixed drafts and outlines: a valid workflow, one chain, roles carried over, unknown refs rejected; a recorded outline of the TodoMVC sample as a golden file.

### Decisions

- **Single page and walk (decided).** Both: the single-page draft is the default; **Walk the site** is a separate button that warns it clicks the real site (it can create accounts or data).
- **New chain or extend (decided): ask each time** when the project already has an Open browser.
- **Local API (decided): yes.** `POST /project/draft` takes the same draft and runs `draft_to_workflow` and `check_workflow`, so outside agents can use the small format; model access stays off the API.

### Risks

- **Page content goes to the model.** The outline can include personal data from a signed-in page; the consent dialog shows it, and "Use the page without signing in" is offered.
- **Names that repeat.** A name may match several elements; picker suggestions and **Try before inserting** catch this.
- **Outline size.** Large pages exceed small models' useful context; cut by depth and size, landmarks first.

### Sources

- [Chrome DevTools Protocol: Accessibility.getFullAXTree](https://chromedevtools.github.io/devtools-protocol/tot/Accessibility/#method-getFullAXTree) (checked 2026-10-25)
- agent-browser snapshot code, `cli/src/native/snapshot.rs` (Apache-2.0; read 2026-10-25)

## Auto-chapters

### What it adds

On a Record take that has run: **Suggest chapters**. Opsis splits the take where it moves to a new page or pauses, the model names each part ("Create a project", "Invite a teammate"), and the user edits the titles and inserts them as Chapter actions and Timeline markers. Export video already writes chapters from Chapter actions into the file and the optional YouTube list (**Chapters in exports** in `AGENTS.md`). Optionally it also names unnamed actions, since an action's name is also its camera, chapter and caption label.

### Fit with Opsis

In place:

- The action log: `ActionLog` (`crates/types/src/actions.rs`) with each action's kind, time, label, text and node id; the Record executor returns it with the recording, and the editor has it in the run store's results.
- The Chapter action `browser.chapter` (`crates/compile/src/nodes/browser.rs`); chapter marks on the Timeline from `flatten_spec`; `film_chapters` for exports.
- `session_into` (`crates/compile/src/graph.rs`) gives a capture's session with the chain's steps in order.

Missing: splitting the log into parts, the naming request, inserting Chapter nodes into a chain.

### Work

1. **Split (no model).** `chapter_spans(log, session)` in `opsis-compile`: cut at navigations, existing chapters and gaps longer than a threshold; each span lists its actions' kinds, labels and node ids.
2. **Name.** Schema `{ chapters: [{ start_node_id, title, description }] }` with `start_node_id` an enum of the spans' first nodes. Input: spans, node names, project name. Typed text is left out unless the user ticks "Include typed text".
3. **Insert.** `insert_chapters(workflow, capture_node_id, chapters)` (pure): a `browser.chapter` node before each start node, rewiring the session edges; an existing Chapter at that spot is renamed instead. One update, then `check_workflow`.
4. **UI.** A button on the Record take's Inspector when it has a result; a dialog with editable titles, "Show a chapter card in the video" off by default, and a note that the next run records the take again.
5. **Tests.** Spans from golden logs; `insert_chapters` keeps one chain, keeps the session's step order and passes `check_workflow`.

### Decisions

- **Outputs (decided):** Chapter actions and Timeline markers, with file chapters and YouTube timestamps from the existing export.
- **Name unnamed actions (decided): yes, as suggestions** in the same dialog.

### Risks

- **Typed text can be private** (emails, test passwords): left out by default, and text typed from a variable that names a saved secret is never sent.
- Inserting chapters changes node keys downstream, so the take records again on the next run.

## Headline copy variants

### What it adds

A sparkles button next to headline text fields (Title card's eyebrow, title and subtitle; Animated headline's text; other text elements). It opens five alternatives in a chosen tone (Plain, Bold, Playful, Technical); clicking one applies it as one undo step, **More** asks again.

### Fit with Opsis

In place:

- `core/title` and `core/kinetic-headline` (`crates/compile/src/elements/core.rs`); the headline keeps its `*asterisks*` emphasis rule.
- Fields carry their UI as `x-` keywords in their JSON Schema and the editor builds forms from it (`schema-form.tsx`); `sparkles` is in the card icons.

Missing: a way to mark a field as copy, and the request.

### Work

1. **Mark fields.** An `x-suggest` keyword (`headline`, `subtitle`, `eyebrow`) on the core elements' fields; registry text elements get it through a rule in `scripts/registry-ingest.list.ts` and `bun run registry:ingest -- --no-add`.
2. **Request.** Schema `{ variants: Vec<String> }`. Input: the current text, the field's role, the tone, the project name, the site address, chapter titles and action names. Variants stay within the current length plus a margin, checked after parsing; the asterisks rule only for the headline.
3. **UI.** A button on fields with `x-suggest` when the assistant is on; a popover list; applying writes the setting like a typed edit.
4. **Tests.** The prompt includes the asterisks rule only for the headline; over-long variants are dropped.

### Decisions

- **Captions too (decided):** the button is also on caption text (Captions from script's lines).
- **Tones (decided):** Plain, Bold, Playful and Technical.

### Risks

- A longer variant can wrap; keep the length check.

## Picker suggestions

### What it adds

After picking an element, the Inspector shows a suggestion when the picked name is weak: "This text appears 3 times on the page. Use “Save invoice” (button) instead?", plus a name for the action ("Save the invoice") when it has none. The uniqueness check works without a model; the model only chooses among names Opsis verified and words the action name.

### Fit with Opsis

In place:

- The picker is a streamed view of the headless shell (`crates/stream/src/picker.rs` with its page script `picker.js`), returning `PickResult` (the target name, its role, a CSS fallback). A nameless element falls back to `css=…`.
- The Inspector's target field stores the name and its role; cards show it through `element_name` (`crates/compile/src/card.rs`).

Missing: alternative names, how many elements share each, nearby context.

### Work

1. **Candidates (no model).** In `picker.js`, collect every name source for the picked element (aria-label, label, placeholder, visible text, test id, title) and count elements with the same role and name; add the nearest heading or landmark text. `PickResult` gains `candidates: [{ name, role, source, count }]` and `context`.
2. **Suggest.** Prefer a unique candidate in the selector ladder's own order (role and name, label, placeholder, test id, text); show it under the target field with Use / Dismiss. A `css=` pick with a unique name candidate always gets a suggestion.
3. **With the assistant.** Schema `{ target, label }`, `target` an enum of the unique candidates; the model picks the clearest and writes an action name when the node has none.
4. **Tests.** Candidate ordering, and the enum: the model can't return a name that wasn't counted.

### Decisions

- **Button (decided):** suggestions appear only on a "Suggest a better name" button, so nothing is sent unless asked.

### Risks

- The in-page count approximates the selector ladder; the run is still the real test.
- The context is page content; online requests go through consent.

## Decided

- **Editions:** both lite and full; the assistant adds no files to either.
- **Request log:** every request and answer is kept in the app database's `assistant_log` table, with a list and **Clear log** in Settings → Assistant. Keys and headers are never logged.
