# AI assist

How Opsis should add the AI assist features from milestone M7 of the original spec ("prompt → flow, auto-chapters, copy variants, element picker suggestions"): what each one gives users, how it uses a model the user chose, and the work needed. Every feature has its own section with the same parts, so the plan can grow one section at a time.

Status: plan only. Nothing here is built yet.

## How to read this plan

- **Model access** comes first: every other feature depends on it. The other four can be built in any order after it, and each can ship on its own.
- Each feature is checked before the next one starts.
- Each section has the same parts: **What it adds**, **Fit with Opsis** (Already in place / Missing, with the files and functions involved), **Work**, **Decisions for the user**, **Risks**, **Sources**.
- External facts link their source and the date they were checked.

## Conventions

These apply to every feature here. They restate rules from `AGENTS.md`; when the two disagree, `AGENTS.md` wins. When a feature ships, add its invariants to `AGENTS.md` (an "AI assist" paragraph next to "Local API").

**Privacy.**

- Nothing contacts a model until the user turns AI assist on in Settings. Off is the default, as with the Local API.
- An online model is asked only after the user agrees to that request: a dialog shows exactly what will be sent. A model on this computer needs no per-request dialog.
- Opening the user's site in the hidden browser happens only when they press a button that says so, like a run.
- Keys are encrypted with `safeStorage` and never reach a renderer window.

**Where code goes.**

- `src/shared/assist/`: the output schemas (zod), prompt builders and the functions that turn a model's answer into a workflow change. Pure and runtime-neutral, so they're tested without a model and could later be exposed on the Local API.
- `src/main/assist/`: the only code that talks to a model, through `fetch` in main. The renderer's CSP (`connect-src 'self' opsis-file: blob: data:`, `src/main/protocol.ts`) blocks other addresses, and keys stay in main.
- `src/worker/`: anything that needs the hidden browser (page outline).
- `src/renderer/assist/`: dialogs and the suggestion UI.
- Dependencies through `bun add` only; file names kebab-case; no comments.

**Model output is never trusted.** Every answer is parsed with the zod schema it was asked for, then every workflow change goes through `checkWorkflow` / `workflowProblems` (`src/shared/api/check.ts`), the same checks the editor and the Local API use. A model never writes params or edges directly. It fills a small draft that Opsis's code turns into nodes.

**User-facing copy.** No library, tool, model or service names in node titles, descriptions, labels, badges, toasts or errors. Decided exception, added to `AGENTS.md` when this is built: Settings → Assistant may name Anthropic and OpenAI in its provider choice and key field, and its circled `?` hover cards may name local model programs; nothing else names them. The feature is called "Assistant" in the UI. Sentence case. Browser nodes are "actions".

**Checks when a feature is done.**

- `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`.
- The no-comments check from `AGENTS.md`.
- Try it in the app (`bun start`) with a model on this computer and with an online model, and with AI assist off: no request leaves the app, and the buttons are hidden.
- Don't commit or push.

## Summary

| Feature                | What users get                                                                                                    | Needs                                 | Status    |
| ---------------------- | ----------------------------------------------------------------------------------------------------------------- | ------------------------------------- | --------- |
| Model access           | Settings → Assistant: a model on this computer or an online one with an encrypted key; consent per online request | nothing                               | not built |
| Prompt → flow          | Describe the demo in a sentence plus the site address; get actions, Record take, Auto zoom, a title and Export    | Model access, engine (hidden browser) | not built |
| Auto-chapters          | Chapter titles for a take, from what it did, inserted as Chapter actions                                          | Model access, a recorded take         | not built |
| Headline copy variants | A list of alternative wordings next to title and headline fields                                                  | Model access                          | not built |
| Picker suggestions     | After picking an element: a clearer, unique name for it and a name for the action                                 | picker change (works without a model) | not built |

## What the Local API already gives outside agents

The Local API (Settings → API, `src/main/api.ts`) already lets an agent on the same computer (Claude Code or any script) build a flow with no in-app AI: `GET /openapi.json`, `GET /nodes` and `GET /nodes/{type}` (`nodeCatalog`, `catalogEntry`, `paramsJsonSchema` in `src/shared/api/catalog.ts`), `POST /validate` (`checkWorkflow`), then edits under `/project`, each one store update and one undo step through `handle` in `src/renderer/editor/api-bridge.ts`, plus `POST /project/run` with events on `/project/run/events`. An outside agent inspects the site with its own browser.

In-app AI reuses the same pieces rather than a second path: the catalogue for node titles, descriptions and settings, `checkWorkflow` for validation, and one `useWorkflowStore.getState().update(...)` for the insert, as the bridge's `replace` does. What it adds that the API lacks is a page outline taken in Opsis's own hidden browser, and a model the user doesn't have to run themselves.

## Model access

### What it adds

A Settings page, **Assistant**, with three choices: Off (default), "A model on this computer", "An online model". Every assist feature asks through one main-process function, so the privacy rules live in one place.

- **On this computer: a local server.** Any server that speaks the OpenAI-compatible chat API on a loopback address. Ollama (MIT, `http://localhost:11434`), llama.cpp's `llama-server` (MIT) and LM Studio (`http://localhost:1234/v1`) all do, and all accept a JSON schema to shape the answer. The user installs and runs the server. Opsis only stores its address and model name, lists the server's models, and has a **Test** button.
- **No model inside Opsis (decided).** The assistant always talks to a service the user chooses, on this computer or online. Opsis ships no model runtime and downloads no models.
- **Online.** A provider, a model name and an API key. Both large providers can force an answer to match a JSON schema: Anthropic through `output_config.format` (type `json_schema`) and strict tool use, OpenAI through `text.format` with `strict: true`. A third option, "Compatible service", takes an address too; OpenRouter and similar services go there, with no preset of their own (decided).

### Fit with Opsis

Already in place:

- Global settings: `appSettingsSchema` (`src/shared/settings.ts`), written one section at a time with `patchSettings` (`src/main/settings.ts`) and followed in every window through `useAppSettings` / `patchAppSettings` (`src/renderer/store/app-settings.ts`).
- The Settings window: pages are listed in `PAGES` (`src/renderer/app/settings/settings-window.tsx`) and typed by `SettingsPage` (`src/shared/ipc.ts`). `ApiPage` (`api-page.tsx`) is the model for an off-by-default switch with a status badge.
- Encryption: `writeSecret` / `readSecret` (`src/main/secrets.ts`) use `safeStorage`, but per project (`<project>/secrets/`), and refuse to save when `isEncryptionAvailable()` is false.
- The app state database (`src/main/db/`, **App state is one database** in `AGENTS.md`): tables in `src/main/db/schema.ts`, exported by `bun run db:schema`, and the `projects` table with every project ever opened. Its unused column type `encryptedText()` (`src/main/db/crypto.ts`) seals a value with `safeStorage`, refuses to save without secure storage or when Linux's keyring is `basic_text`, and reports "This key can't be read on this computer. Enter it again." when a value won't decrypt.
- Asking before going online: the "Go online to get fonts?" `AlertDialog` (`src/renderer/fonts/fonts-panel.tsx`); main fetches with `fetch` (`src/main/fonts.ts`).
- IPC is restricted to the app's own windows by `assertTrustedSender` in `handle` / `handleFrom` (`src/main/main.ts`). Long main-process work is listed on exit with `track` (`src/main/exit.ts`).
- Licences of downloaded things are shown in Settings → Licences (`licences-page.tsx`, fonts via `fontLicence`).

Missing:

- An `assist` settings section, a page, a key store outside projects (no `secrets` table yet), the model client, a consent step and a request log (no `assist_log` table).

### Work

1. **Settings.** Add `assist` to `appSettingsSchema`: `mode: "off" | "local" | "online"` (default `off`), `localAddress` (default `http://127.0.0.1:11434/v1`), `localModel`, `provider: "anthropic" | "openai" | "compatible"`, `onlineAddress`, `onlineModel`, `askEachTime` (default `true`).
2. **Key store.** Unless `plans/voice-and-captions.md` has already done it:
   - Add a `secrets` table to `src/main/db/schema.ts`: `id` integer primary key, `name` text not null, unique, `value` `encryptedText()` not null. Run `bun run db:schema`; development databases are replaced on the next start.
   - Remove the `// TODO: To be used later` comment above `encryptedText` and the note about it in `AGENTS.md`.
   - `src/main/app-secrets.ts`, shared with the voice plan's service keys: `writeAppSecret(name, value)`, `hasAppSecret(name)`, `clearAppSecret(name)`, and `readAppSecret(name)` for main only.

   `src/main/assist/keys.ts` wraps it with keys named `assist:<provider>`: `writeModelKey(provider, key)`, `hasModelKey(provider)`, `clearModelKey(provider)`, and `readModelKey` used only inside `src/main/assist/`. Saving shows `encryptedText`'s message when there's no keyring. The renderer only ever learns `hasKey: boolean`. Tests open the database with `openDatabase(IN_MEMORY)` and mock `safeStorage`.

3. **Client.** `src/main/assist/client.ts`: `askModel({ task, system, input, schema, signal })` returns parsed JSON or a plain error. It picks the provider:
   - local server: OpenAI-compatible chat completions with `response_format` and a JSON schema
   - Anthropic: Messages API with `output_config.format`
   - OpenAI: Responses API with `text.format`, `strict: true`
   - compatible: as the local server, at `onlineAddress`

   It refuses when `mode` is `off`, when an online request has no consent token (step 5), and treats a `localAddress` that isn't loopback as online. Each call is wrapped in `track("Waiting for the assistant", …)` and is cancellable.

4. **Schemas that every provider accepts.** Strict online modes reject `minimum` / `maximum`, string lengths and optional-heavy objects (Anthropic: at most 24 optional parameters per request). Write each task's output schema in `src/shared/assist/` as flat zod objects with every field required (empty string for "none"), turn it into JSON Schema with `z.toJSONSchema` (as `paramsJsonSchema` does), and enforce limits after parsing.
5. **Consent.** IPC `assist:ask` (via `handleFrom`). For an online request the renderer first shows `AssistConsentDialog`: what will be sent (the exact text, expandable) and Send / Cancel, plus "Don't ask again", which turns `askEachTime` off for the whole app (decided: app-wide, not per project). It can be turned back on in Settings → Assistant. Main issues a one-time token per approved request id and rejects online requests without one.
6. **Page.** `src/renderer/app/settings/assistant-page.tsx` and a `SettingsPage` `"assistant"` entry: the three-way choice ("Off", "A model on this computer", "Online"), address and model fields, provider choice (Anthropic, OpenAI, Compatible service) and key field (Save / Remove; never shown back), **Test** (one tiny request; for online, through the consent dialog), and "Ask before every online request". "A model on this computer" has a circled `?` hover card (the same `InfoHint` as the voice plan) naming programs known to work (Ollama, llama.cpp's server, LM Studio) and their usual addresses; the plain label is all a normal user sees.
7. **Indicator.** Like the API badge, show a small "Assistant" badge in the app bar while `mode` is `online`, so the user knows requests can leave the computer.
8. **Request log.** Add an `assist_log` table to `src/main/db/schema.ts`: `id` integer primary key, `at` integer, `projectId` integer referencing `projects` (null when no project is open, `onDelete: "set null"`), `task`, `service`, `model` text, `request` and `response` JSON text, `error` text (null on success), `ms` integer; run `bun run db:schema`. `logAssist(entry)` in `src/main/assist/log.ts`, called by `askModel` after every request, local or online. Keys and request headers are never logged. Settings → Assistant lists the entries newest first with what was sent and what came back, and has **Clear log**. Keep the last 1,000 entries or 30 days, whichever is smaller, pruned at startup. Tests: an entry round-trips, nothing secret is stored, pruning keeps the newest.
9. **Tests.** The schemas convert to JSON Schema with no unsupported keywords and every field required. The client refuses without the switch, without consent and on a non-loopback local address (with a fake `fetch`). Keys never appear in IPC replies.

### Decisions for the user

- **Naming providers (decided).** Anthropic and OpenAI are named in Settings → Assistant's provider choice and key field; OpenRouter goes under Compatible service; local programs are named only in the `?` card. Everywhere else it's "online model" or "the assistant".
- **No built-in model (decided).** Only services the user runs or chooses.
- **"Don't ask again" (decided).** Allowed, app-wide.

### Risks

- **Linux keyrings.** Without a desktop keyring, `safeStorage` falls back to `basic_text`, which is a hardcoded password. Refusing to store keys there means some Linux users must paste the key each session; offer that, kept in memory only.
- **Small local models** follow schemas but plan poorly. Keep every task to "fill this small draft" and let Opsis's code do the structure.
- **A local server isn't the internet but is still a server.** Never connect to a non-loopback address without the online consent flow.
- **Cost and limits** of online models belong to the user; show the model's error text plainly and don't retry more than once.

### Sources

- [Ollama: structured outputs](https://docs.ollama.com/capabilities/structured-outputs), [Ollama repository (MIT)](https://github.com/ollama/ollama) (checked 2026-10-16)
- [llama.cpp server README](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md): OpenAI-compatible chat, `response_format` with JSON schema, `--jinja` tool calls (checked 2026-10-16)
- [LM Studio: OpenAI compatibility](https://lmstudio.ai/docs/developer/openai-compat) (checked 2026-10-16)
- [Anthropic: structured outputs](https://platform.claude.com/docs/en/build-with-claude/structured-outputs) (schema limits, complexity limits), [OpenAI: structured outputs](https://developers.openai.com/api/docs/guides/structured-outputs) (checked 2026-10-16)
- [Electron: safeStorage](https://www.electronjs.org/docs/latest/api/safe-storage) (`basic_text` on Linux; checked 2026-10-16)

## Prompt → flow

### What it adds

On an empty canvas, in the New project dialog and in the Edit menu: **Describe a demo…**. The user writes one sentence ("Sign up, create a project and invite a teammate") and the site address. Opsis opens the page in the hidden browser, takes an outline of what's on it, and asks the model for a draft. The user reviews the draft as a list of actions, edits it, and inserts it: Session, Go to page, the actions, Record take, Auto zoom, a Title card, a Sequence and Export, as one undo step.

### Fit with Opsis

Already in place:

- The hidden browser: `openSession` (`src/worker/capture/session.ts`) launches the engine's headless shell with the Session node's settings and an optional saved sign-in. Installed `playwright-core` 1.64.1 has `page.ariaSnapshot({ mode: "ai", boxes })`, an accessibility outline with element refs (`[ref=e2]`), made for models.
- An on-demand worker queue in main: `analyze` (`src/main/analysis.ts`) spawns a worker with `spawnWorker`, idles out after `IDLE_MS`, and checks `engineStatus(engineDir())` first. Messages are typed in `MainToWorker` (`src/shared/worker-protocol.ts`).
- The selector ladder: `autoCandidates` / `candidatesFor` / `softFirst` (`src/worker/capture/selectors.ts`) find an element by its visible name and role, and `dumpVisible` lists what's on screen when one isn't found. Targets are stored as a name plus `targetRole` (the picker's `${key}Role` convention in `src/renderer/editor/inspector.tsx`).
- Takes run through `playTake` (`src/worker/capture/driver.ts`), wrapped by `recordTake` and `stillTake`; a check take would be a third wrapper with no recording and no finale.
- Building graphs in code: `GraphBuilder` (`node`, `link`, `chain`, `build`; `src/shared/templates/build.ts`); `todoMvcSample` (`src/shared/templates/index.ts`) is exactly the target shape. Title card is `fx.core.title` (`elementNodeType` in `src/shared/elements/index.ts`).
- The catalogue (`catalogNode`, `nodeCatalog` in `src/shared/api/catalog.ts`) gives titles, descriptions and settings schemas; `checkWorkflow` validates; `useWorkflowStore` `update` and `setVariables` (`src/renderer/store/workflow.ts`) insert as one undo step.

Missing:

- A snapshot job, the draft schema, the draft-to-nodes mapping, the dialog.
- The full catalogue is about 400 KB and its schemas use keywords strict modes reject, so it can't be the model's schema as-is.

### Work

1. **Page outline.** `src/worker/assist/outline.ts`: open the page with `openSession` (the project's Session settings and saved sign-in when there is one), wait for load, take `page.ariaSnapshot({ mode: "ai" })`, cut it to a fixed size, and parse each ref into `{ ref, role, name }`. Return the outline text, the ref table, the page title and the final address. Main: `outlinePage()` in `src/main/assist/outline.ts`, queued like `analyze`, with a new `MainToWorker` message.
2. **Draft schema.** `src/shared/assist/flow.ts`: `flowDraftSchema` is `{ title, subtitle, actions: [{ do, ref, text, address, label }] }`, everything required. `do` is an enum of draft actions mapped to node types (`browser.navigate`, `browser.click`, `browser.type`, `browser.hover`, `browser.scroll`, `browser.press`, `browser.select`, `browser.wait`, `browser.chapter`). `ref` must be a ref from the outline (built as an enum per request), or empty.
3. **Catalogue as the model's tools.** The prompt lists each draft action with `catalogNode(def, false)`'s title and description, so new wording in a node definition reaches the model. A test checks every draft action maps to a visible type in `nodeCatalog()` and that the params `draftToWorkflow` writes pass that node's zod schema.
4. **Draft to nodes.** `draftToWorkflow(wf, draft, outline)` (pure): refs become `target` = the element's name plus `targetRole`; the first action is Go to page with `{{Site address}}`; then `capture.record`, `camera.autoZoom`, `fx.core.title` with the title, `timeline.sequence`, `output.render`. Place them to the right of existing nodes (move `nextPosition` from `api-bridge.ts` to `src/shared` and use it in both). Set the "Site address" variable when it's empty.
5. **Check and repair.** Run `checkWorkflow` on the result. If the new nodes have errors, send the problems back once; anything left shows in the review list.
6. **Review dialog.** `src/renderer/assist/describe-dialog.tsx`: sentence and address, progress ("Opening the page", "Writing the actions", "Checking"), then the draft as editable rows using the node icons from `icons.ts`. Rows whose target wasn't on the first page are marked "Not seen on the page". **Insert** applies one `update`, selects the new nodes and fits the view.
7. **Entry points.** A `MenuCommand` `edit:describe` in `appMenu()` (`src/shared/menu.ts`), the empty-canvas button, and a "Describe your demo" choice in `new-project-dialog.tsx`. Hidden when the assistant is off; without the engine, offers to set it up.
8. **Try before inserting.** "Try the actions" runs the new chain as a check take (no recording, a new wrapper over `playTake`) and reports targets that didn't resolve, with the `dumpVisible` list.
9. **Walk the site.** For flows across pages: a loop in the worker where the model picks one action at a time from a fresh outline, the worker performs it, and only actions that worked become the draft.
10. **Tests.** `draftToWorkflow` on fixed drafts and outlines: valid workflow, one chain, roles carried over, unknown refs rejected. A recorded outline of the TodoMVC sample as a fixture.

### Decisions for the user

- **Single page and walk (decided).** Both: the single-page draft is the default; **Walk the site** is a separate button that warns it clicks the real site (it can create accounts or data) before it starts.
- **New chain or extend (decided): ask each time.** When the project already has a Session, the insert dialog asks: a new take, or append to the selected chain.
- **Local API (decided): yes.** `POST /project/draft` takes the same compact draft and runs `draftToWorkflow` and `checkWorkflow`, so outside agents can use the small format. Model access itself stays off the API.

### Risks

- **Page content goes to the model.** The outline can include personal data from a signed-in page. The consent dialog shows the outline; offer "Use the page without signing in".
- **Names that repeat.** A name in the outline may match several elements; the ladder takes the first. Picker suggestions (below) and **Try before inserting** catch this.
- **Outline size.** Large pages exceed small models' useful context; cut by depth and size, and keep main landmarks first.

### Sources

- [Playwright: aria snapshots](https://playwright.dev/docs/aria-snapshots) and `page.ariaSnapshot` options in the installed `playwright-core@1.64.1` types (`mode: "ai"`, `boxes`; checked 2026-10-16)
- The original spec's node catalogue, row "AI (later)": "Script → flow (LLM writes Browser nodes from a sentence + DOM snapshot)"

## Auto-chapters

### What it adds

On a Record take that has run: **Suggest chapters**. Opsis splits the take where it moves to a new page or pauses, the model names each part ("Create a project", "Invite a teammate"), and the user edits the titles and inserts them. Decided outputs, all four: Chapter actions in the take's chain (the take records again on the next run), Timeline markers at the same moments, chapters written into exported MP4, MOV and MKV files, and a YouTube timestamp list next to the export. The last two come from Chapter actions in general, with or without the assistant; Export video already writes them (**Chapters in exports** in `AGENTS.md`), so Suggest chapters only adds the Chapter actions and markers. Optionally it also names unnamed actions, since an action's name is also its camera, chapter and caption label.

### Fit with Opsis

Already in place:

- The action log: `actionSchema` / `ActionLog` (`src/shared/schema/actions.ts`) with `kind`, `t`, `label`, `text`, `nodeId`. The Record executor writes it to `actions.json` and returns it as `outputs.recording.log` (`src/worker/executors/capture.ts`); the renderer has it in `useRunStore` `results` (`NodeResult`, `src/shared/schema/values.ts`).
- The Chapter action `browser.chapter` (`chapterNode`, `src/shared/nodes/browser.ts`: title, description, optional card). The driver logs it as a `chapter` action, and the Timeline draws chapter marks (`flattenSpec`, `marker-row.tsx`).
- The compiled `browser-session` value at a capture's `session` input (`sessionInto` in `src/shared/graph.ts`) lists the chain's steps in order.

Missing:

- Splitting the log into parts, the naming request, and inserting Chapter nodes into a chain.

### Work

1. **Split (no model).** `chapterSpans(log, chain)` in `src/shared/assist/chapters.ts`: cut at `navigate` actions, at existing chapters and at gaps longer than a threshold; each span lists its actions' kinds, labels and node ids.
2. **Name.** Schema `{ chapters: [{ startNodeId, title, description }] }` with `startNodeId` an enum of the spans' first nodes. Input: spans, node labels, project name. Typed text is left out unless the user ticks "Include typed text".
3. **Insert.** `insertChapters(wf, captureNodeId, chapters)` (pure): a `browser.chapter` node before each start node, rewiring the session edges; an existing Chapter at that spot is renamed instead. One `update`, then `checkWorkflow`.
4. **UI.** A button on the Record take's Inspector when it has a result; a dialog with editable titles, "Show a chapter card in the video" off by default, and a note that the next run records the take again.
5. **Tests.** Spans from fixture logs; `insertChapters` keeps one chain, leaves the session's step order intact and passes `checkWorkflow`.

### Decisions for the user

- **Outputs (decided):** Chapter actions, Timeline markers, file chapters and YouTube timestamps, as above.
- **Name unnamed actions (decided): yes, as suggestions** in the same dialog; the user accepts or edits each.

### Risks

- **Typed text can be private** (emails, test passwords). Leave it out by default and never send text typed from a `{{variable}}` that names a saved secret.
- Inserting chapters changes the node hashes downstream, so the take re-records on the next run.

### Sources

- The original spec's node catalogue, row "AI (later)": "Auto-chapter (labels from ActionLog)"

## Headline copy variants

### What it adds

A sparkles button next to headline text fields (Title card's eyebrow, title and subtitle; Animated headline's text; other text elements). It opens a list of five alternatives in a chosen tone (Plain, Bold, Playful, Technical); clicking one applies it as one undo step, **More** asks again.

### Fit with Opsis

Already in place:

- `core/title` and `core/kinetic-headline` in `src/shared/elements/core.ts`; the headline keeps the `*asterisks*` emphasis convention.
- Field metadata: `FieldMeta` (`src/shared/elements/introspect.ts`), rendered by `src/renderer/editor/zod-form.tsx`; `sparkles` is already in `src/renderer/editor/icons.ts`.

Missing:

- A way to mark a field as copy, and the request.

### Work

1. **Mark fields.** Add `suggest?: "headline" | "subtitle" | "eyebrow"` to `FieldMeta`; set it in `core.ts`. For registry text elements, add a rule in `scripts/registry-ingest.list.ts` and re-run `bun run registry:ingest -- --no-add` (never edit `elements.generated.ts`).
2. **Request.** `src/shared/assist/copy.ts`: schema `{ variants: string[] }`. Input: the current text, the field's role, tone, the project name, site address, chapter titles and action names. Keep variants within the current length plus a margin, checked after parsing; keep the asterisks rule for the headline.
3. **UI.** In `zod-form.tsx`, a button on fields with `suggest` when the assistant is on; a popover list; applying calls the Inspector's param update.
4. **Tests.** The prompt builder includes the asterisks rule only for the headline; over-long variants are dropped.

### Decisions for the user

- **Captions too (decided):** the variants button is on headline fields and on caption text (Captions from script's lines).
- **Tones (decided):** Plain, Bold, Playful and Technical.

### Risks

- Fonts and sizes don't change with the wording; a longer variant can wrap. Keep the length check.

### Sources

- The original spec's node catalogue, row "AI (later)": "Copywriter (headline variants)"

## Picker suggestions

### What it adds

After picking an element, the Inspector shows a suggestion when the picked name is weak: "This text appears 3 times on the page. Use “Save invoice” (button) instead?", plus a name for the action ("Save the invoice") when it has none. The uniqueness check works without a model; the model only chooses among names Opsis verified and words the action name.

### Fit with Opsis

Already in place:

- The picker runs in a sandboxed Electron window (`pickInWindow`, `src/main/browser-windows.ts`). `PICKER_SCRIPT` computes `nameOf`, `cssOf` and `roleOf` and returns `PickResult` (`target`, `css`, `role`, `selectorUsed`; `src/shared/ipc.ts`). A nameless element falls back to `css=…`.
- The Inspector's `pickTarget` (`inspector.tsx`) stores the name and `${key}Role`; cards show it through `elementName` (`src/shared/nodes/card.ts`).

Missing:

- Alternative names, how many elements share each, and nearby context.

### Work

1. **Candidates (no model).** In `PICKER_SCRIPT`, collect every name source for the picked element (aria-label, label, placeholder, visible text, test id, title) and count the elements with the same role and name. Add the nearest heading or landmark text. Extend `PickResult` with `candidates: [{ name, role, source, count }]` and `context`.
2. **Suggest.** Prefer a unique candidate in the ladder's own order (role and name, label, placeholder, test id, text). Show it under the target field with Use / Dismiss. A `css=` pick with a unique name candidate always gets a suggestion.
3. **With the assistant.** Schema `{ target, label }`, with `target` an enum of the unique candidates. The model picks the clearest and writes an action name when `node.label` is empty.
4. **Tests.** Candidate ordering and the enum: the model can't return a name that wasn't counted.

### Decisions for the user

- **Button (decided):** suggestions appear only on a "Suggest a better name" button, so nothing is sent unless asked.

### Risks

- The in-page count approximates the ladder (`autoCandidates` matches loosely too); the run is still the real test.
- The context is page content; online requests go through consent.

### Sources

- `src/main/browser-windows.ts`, `src/worker/capture/selectors.ts` (read 2026-10-16)

## Decided

- **Editions (decided):** both lite and full; the assistant adds no files to either.
- **Request log (decided):** every request and answer is stored in the app state database's `assist_log` table (**Model access**, step 8), with a list and **Clear log** in Settings → Assistant. Keys and headers are never logged.
