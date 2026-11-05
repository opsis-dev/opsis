# Svelte runtime

How Opsis moves its pages off React's `useState` runtime onto Svelte 5: compiled components whose `$state` and `$derived` signals update only the DOM nodes that read them. The film page and the terminal page move in every case; the editor moves only if it stays a web app (see [native-editor.md](native-editor.md)).

Status: plan only. Nothing here is built yet. Facts were checked on 4–5 November 2026.

## How to read this plan

- **Toolchain** comes first: every other part needs it.
- **Film runtime** comes next, then **Film elements**, **3D** and **Terminal page**, in that order: the film page has the strongest safety net (golden frames and still images), so the runtime is proven there first.
- **Editor** is last and conditional: it runs only if [native-editor.md](native-editor.md) is not taken, or for the time before the native editor replaces `apps/frontend`.
- Each part has the same sections: **What it changes**, **Fit with Opsis** (In place / Missing), **Work**, **Decisions**, **Risks**, **Sources**.

## Conventions

These restate rules from `AGENTS.md` and `agent-docs/`; when they disagree with this plan, they win. When a part ships, its rules move into the doc that owns the domain (`film.md`, `terminal.md`, `editor.md`, `toolchain.md`).

- **Every animation stays a pure function of the frame.** No CSS transitions, `motion`, GSAP or Tailwind `animate-*` in `packages/video`; Svelte's own `transition:` and `animate:` directives join that ban.
- **The page protocol does not change.** `window.opsis` keeps `load(spec)`, `seek(frame)` and `holds()` exactly as `packages/runtime/src/page.tsx` defines them, so the frame driver in `crates/capture/src/film.rs` and the terminal painter need no change.
- **No comments** in `.svelte` files either: not in markup (`<!-- -->`), script or style.
- **Every function is a declaration** in `<script>` blocks and `.svelte.ts` modules too.
- **Golden files are recorded, never edited**; a part is done when the golden frames and `bun run elements:render` match without re-recording, except where a difference is intended and reviewed.
- **Checks when a part is done:** `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, the ignored render tests (`cargo test -p opsis-render -- --ignored` after `bun run film:bundle`), and `bun run elements:render`.

## Decided

- **Svelte 5 (runes), the latest stable release (5.57.3 on 4 November 2026).** Every library this plan uses is a stable release from an active team; the alternatives and why they lost are under [Alternatives](#alternatives).
- **`.svelte` files are type-checked by `svelte-check` on TypeScript 6**, its supported peer range (`^5 || ^6`). TypeScript 7 keeps checking every `.ts` and `.svelte.ts` file through tsgo. Microsoft's TypeScript 7.0 announcement tells Svelte projects to stay on 6.0 for component checking until TypeScript 7.1 ships its API, and svelte-check's TypeScript 7 mode still crashes ([language-tools #3063](https://github.com/sveltejs/language-tools/issues/3063), open since June 2026).
- **The runtime change does not fix the editor's measured slowness.** The profile put most editor blocking in style, layout and paint: 21,852 DOM elements for 439 nodes, every node and edge drawn off-screen too. The canvas now draws only what is on screen and gives React Flow each card's saved size and handles on open (`93999ea0`), so reopening the 438-node project draws 285 cards and 14,363 elements. Any runtime renders that same DOM; Svelte Flow has the same `onlyRenderVisibleElements` option and needs the same saved shapes.

## Summary

| Part          | What changes                                                      | Needs         |
| ------------- | ----------------------------------------------------------------- | ------------- |
| Toolchain     | Svelte build, type checking, linting, comment check, tests        | —             |
| Film runtime  | `packages/runtime` on signals; `window.opsis` unchanged           | Toolchain     |
| Film elements | the 211 `.tsx` files of `packages/video/src`, moved one at a time | Film runtime  |
| 3D            | `three/stage.tsx` and its 6 elements on Threlte                   | Film elements |
| Terminal page | `apps/terminal` and `video/terminal/view`                         | Film runtime  |
| Editor        | `apps/frontend`, only if it stays a web app                       | Toolchain     |

## Toolchain

### What it changes

Svelte joins the build, the type check, the lint, the comment check, the formatter and the tests, beside the TypeScript files that stay.

### Fit with Opsis

**In place.**

- TypeScript 7 (`typescript ^7.0.2`, tsgo) checks every member; `tsconfig.base.json` sets `"jsx": "react-jsx"`.
- oxlint with the Opsis JS plugin (`scripts/oxlint-plugin-opsis.ts`) and the shared base (`scripts/lib/oxlint-base.ts`: the `react` plugin, `rules-of-hooks`, the wall-clock library ban, the `@react-three/fiber` import ban in `video`).
- `scripts/comments.ts` parses every TS file with `oxc-parser`.
- oxfmt (`scripts/lib/oxfmt-base.ts`).
- Vitest with two projects, Node and happy-dom (`scripts/lib/vitest-base.ts`).
- Vite configs in `apps/film`, `apps/terminal` and `apps/frontend`; only the editor uses `@vitejs/plugin-react`.

**Missing.**

- The Svelte compiler and Vite plugin.
- A type check for `.svelte` files: tsgo can't read them.
- Lint coverage of markup: oxlint JS plugins don't read `.svelte` files and oxlint has no template linting.
- A comment check for `.svelte` files: `oxc-parser` can't parse them.
- Component test support for Svelte.

### Work

1. `bun add svelte` in `packages/runtime`, `packages/video`, `apps/film`, `apps/terminal` (and `apps/frontend` for the Editor part); `bun add -d @sveltejs/vite-plugin-svelte` (7.3.1) in each app.
2. Add the Svelte plugin to each app's `vite.config.ts` and to `scripts/lib/vitest-base.ts`.
3. Install svelte-check with the two TypeScript versions the way its README documents: `typescript@~6` (what svelte-check imports) beside `@typescript/native@npm:typescript@7` (tsgo), in each member that has `.svelte` files. That member's `typecheck` then runs tsgo through the alias for `.ts` files and `svelte-check` for `.svelte` files. Members without `.svelte` files keep `typescript ^7` unchanged.
4. Lint markup: `bun add -d eslint eslint-plugin-svelte` (3.24.0) and port the Opsis rules that read markup (`opsis/no-animate-class`, the restricted imports) into an ESLint config that runs only on `*.svelte`; oxlint keeps every `.ts` file. `lint` and `lint:fix` run both.
5. Comment check: give `scripts/comments.ts` a `.svelte` path that parses with `svelte/compiler`'s `parse` (modern AST) and reports HTML comments in markup and every comment in `<script>` and `<style>`.
6. Formatting: oxfmt formats `.svelte` through its Svelte support once the `svelte` package is installed; add `*.svelte` to `scripts/lib/oxfmt-base.ts`.
7. Tests: `bun add -d @testing-library/svelte` (5.4.2) where components are tested; component tests stay `*.test.ts` files that mount `.svelte` components.
8. Add the conventions above to `AGENTS.md` §2 and §4 and `agent-docs/toolchain.md`: the two TypeScript versions and why, the two linters and why, the `.svelte` comment path.

### Decisions

- **ESLint for markup, or a markup gap until oxlint reads `.svelte`.** The oxlint JS plugin announcement (March 2026) promises framework formats later; until then, ESLint on `.svelte` only keeps the film's determinism rules enforced in markup. Recommended: ESLint on `.svelte` only.
- **happy-dom or jsdom for Svelte component tests.** Run the first ported component's test on happy-dom (today's DOM project); move the Svelte tests to jsdom only if happy-dom fails them.
- **Rule names.** `opsis/use-function-components` has no meaning in Svelte (the file is the component); retire it with React, and add a rule that `.svelte` files are kebab-case like other TS files.

### Risks

- Two TypeScript versions in one repository until TypeScript 7.1 content mappers are stable (7.1.0-beta in October 2026).
- Two linters; a rule ported to ESLint can drift from its oxlint twin.

### Sources

- Svelte releases: https://github.com/sveltejs/svelte/releases (checked 2026-11-04)
- Vite plugin: https://github.com/sveltejs/vite-plugin-svelte (7.3.1, checked 2026-11-04)
- svelte-check and its TypeScript 7 section: https://github.com/sveltejs/language-tools/tree/master/packages/svelte-check (checked 2026-11-05)
- TypeScript 7.0 announcement: https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/ (checked 2026-11-04)
- TypeScript 7.1 beta: https://devblogs.microsoft.com/typescript/announcing-typescript-7-1-beta/ (checked 2026-11-04)
- eslint-plugin-svelte: https://github.com/sveltejs/eslint-plugin-svelte (3.24.0, checked 2026-11-05)
- oxlint JS plugins and template support: https://oxc.rs/docs/guide/usage/linter/js-plugins, https://oxc.rs/compatibility.html (checked 2026-11-04)
- oxfmt language support: https://oxc.rs/docs/guide/usage/formatter/language-support.html (checked 2026-11-04)
- Testing Library for Svelte: https://github.com/testing-library/svelte-testing-library (5.4.2, checked 2026-11-04)

## Film runtime

### What it changes

`packages/runtime` keeps the frame in one piece of state. A seek writes it inside `flushSync`, and only the bindings and `$derived` values that read the frame run again; static parts of an element cost nothing per frame.

### Fit with Opsis

**In place.**

- `page.tsx`: `mountFilm` with `createRoot`, `onUncaughtError` and `onRecoverableError` mapped to `failRender`; each seek is `flushSync(() => root.render(<FilmClock film frame>…))` followed by `ready()` (holds, then `document.fonts.ready`, then holds again).
- `clock.tsx`: one `ClockContext`; `useFrame`, `useFilm`; `Sequence`, `Freeze`, `Loop` provide derived clocks, and `Sequence` returns `null` outside its range.
- `holds.ts`: `holdRender`, `failRender`, `settled`, `openHolds` are framework-free; `useHeldLoad` uses `useState`, `useEffectEvent` and `useLayoutEffect`.
- `parts.tsx` (`AbsoluteFill`, `Img`), `environment.ts`, `assets.ts`, the maths (`interpolate`, `Easing`, `spring`, `random`), property-tested.
- `test-frame.tsx`, the test harness; `clock.test.tsx`, `holds.test.tsx`.

**Missing.** A Svelte clock, Svelte versions of `Sequence`, `Freeze`, `Loop`, `AbsoluteFill`, `Img`, a Svelte `useHeldLoad`, a Svelte test harness.

### Work

1. `clock.svelte.ts`: a `FilmClock` class with `frame = $state(0)` and the film's size, fps and length; `setClock`/`getClock` over Svelte context; `frame()` and `film()` getters replace `useFrame()` and `useFilm()`.
2. `film-clock.svelte`, `sequence.svelte`, `freeze.svelte`, `loop.svelte`: each provides a child clock whose frame is `$derived` from its parent's (`parent.frame - from`, held, or wrapped), keeping today's boundary slack (`boundarySlack`); `Sequence` renders its children inside `{#if}` instead of returning `null`, keeping `premountFor`.
3. `page.svelte.ts`: `mountFilm` mounts the film with `mount()` and `seek(frame)` runs `flushSync(() => { clock.frame = frame; })` then the existing `ready()`. Errors thrown while mounting or flushing go to `failRender`, as `onUncaughtError` does today.
4. `useHeldLoad` becomes `heldLoad(key, load)`: hold the frame, run the load, write the result inside `flushSync`, release. `holds.ts` stays as it is.
5. `absolute-fill.svelte` and `img.svelte` (the image holds its frame until decoded, as today).
6. `test-frame.svelte.ts`: mount a component at a frame and step it with `flushSync`; port `clock.test.tsx` and `holds.test.tsx` and keep their properties.
7. Keep the React runtime files beside the Svelte ones until the last element moves (see Film elements).

### Decisions

- **Svelte context or a module-level clock.** Context keeps nested `Sequence`/`Loop` clocks local like today; a module-level clock is simpler but can't express nested time. Recommended: context.
- **Where async errors go.** React's `onRecoverableError` has no Svelte twin; wrap `mount` and every `flushSync` and route errors to `failRender`, and add a test that a throwing element fails the render with its message.

### Risks

- Svelte batches writes outside `flushSync`; a write that escapes it (in an effect or a promise) lands after the frame is captured. Every write the page makes during a seek goes through `flushSync`, and a test steps a film backwards and forwards comparing frames.
- Svelte's `$effect` runs after the DOM updates; code that today relies on `useLayoutEffect` timing must use `$effect.pre` or run inside the seek.

### Sources

- Svelte runtime API (`mount`, `flushSync`, `tick`, `settled`): https://svelte.dev/docs/svelte/svelte (checked 2026-11-04)
- Runes (`$state`, `$derived`, `$effect`, `$effect.pre`): https://svelte.dev/docs/svelte/what-are-runes (checked 2026-11-04)
- Context: https://svelte.dev/docs/svelte/context (checked 2026-11-04)

## Film elements

### What it changes

The 211 `.tsx` files in `packages/video/src` (elements, parts, camera, lib, `film.tsx`, `DemoTimeline`) become `.svelte` components and `.svelte.ts` modules, one element at a time, with the film rendering correctly after every step.

### Fit with Opsis

**In place.**

- `ELEMENTS` (`packages/video/src/elements/index.ts`) keys implementations separately from the Rust metadata, so an element's implementation can change without touching `crates/compile`.
- The golden frames and `bun run elements:render` (still images compared byte for byte) catch any frame that comes out different.
- `useFrame()` is the only clock elements read.

**Missing.** A way to run a React element inside a Svelte film while the move is under way; Svelte versions of every element.

### Work

1. `react-island.svelte`: mounts a not-yet-ported React element with `createRoot` into its own element. After each Svelte seek it renders the React element with React's own `flushSync`, passing the island's local frame, and shares holds through `holds.ts`. Both flushes are synchronous, so the captured frame is exact.
2. `film.tsx` and `DemoTimeline` move first, so the film is a Svelte tree from the root; every element still in React runs through `ReactIsland`.
3. Move elements in this order, each followed by the golden frames and `elements:render`:
   1. text, cards, captions, overlays, backgrounds without 3D;
   2. camera (`packages/video/src/camera`) and footage (`FootageFrame`);
   3. transitions (`lib/transitions`, including `canvas-presentation.tsx`);
   4. 3D (next part).
4. `parts/shader-fields.tsx` moves from `@paper-design/shaders-react` to the plain `@paper-design/shaders` package (already a dependency), driven from the frame.
5. `lottie-web` and `roughjs` are already plain JavaScript; their wrappers become Svelte components.
6. When no element imports React: remove `react`, `react-dom`, `@react-three/fiber`, `@paper-design/shaders-react` from `packages/video` and `packages/runtime`, delete `react-island.svelte` and the React runtime files, drop the `react` oxlint plugin from the base for these members.
7. Update `agent-docs/film.md` and `agent-docs/elements.md` for the Svelte element file shape.

### Decisions

- **Order inside each group.** Elements with the most `useFrame` readers first, since they gain most from signals; `rg -c useFrame packages/video/src` lists them.
- **Props.** Svelte 5 props come from `$props()`; each element types them with its settings type from `packages/video/src/generated/spec.ts`, as today.
- **File names.** Components are kebab-case `.svelte` files (`film-clock.svelte`), imported under their PascalCase name, matching the kebab-case rule for TS files.

### Risks

- Destructuring a reactive object outside `$derived` reads it once; a value read that way stops following the frame. The golden frames show this as a frozen element.
- Two runtimes in the film bundle during the move make it larger and add React's scheduler to every frame that still has an island.

### Sources

- Svelte props: https://svelte.dev/docs/svelte/$props (checked 2026-11-04)
- Paper shaders (plain package): https://github.com/paper-design/shaders (checked 2026-11-04)

## 3D

### What it changes

`packages/video/src/three/stage.tsx` and the six elements on it (`background/shapes-3d.tsx`, `device/screen-3d.tsx`, `scene/card-3d.tsx`, `scene/logo-3d.tsx`, `scene/model-3d.tsx`, `text/text-3d.tsx`) move from React Three Fiber to Threlte.

### Fit with Opsis

**In place.** `stage.tsx` runs React Three Fiber with `frameloop="never"`, calls `advance()` from a layout effect on each frame and holds the render until the scene is ready. `three` itself is a dependency; prepared models, typefaces and textures come from `opsis-prepare`.

**Missing.** A Threlte stage with the same advance-on-seek behaviour.

### Work

1. `bun add @threlte/core` (8.7.0) in `packages/video`.
2. `stage.svelte`: Threlte `<Canvas renderMode="manual">`; on each seek, after the frame write, call `advance()` once and hold the frame until it has drawn, as today.
3. Move the six elements, each checked with the golden frames and `elements:render`.
4. Replace the oxlint ban on Fiber's `useFrame` with a ban on Threlte's `useTask` in `video`, for the same reason: the film must not run its own loop.

### Decisions

- **Threlte extras (`@threlte/extras`).** Use only what replaces code Opsis already has; nothing that starts its own clock.

### Risks

- Threlte's manual mode must not render on its own between seeks; a test renders a 3D still twice at the same frame and compares bytes.

### Sources

- Threlte: https://github.com/threlte/threlte (8.7.0, checked 2026-11-04)
- Threlte render modes: https://threlte.xyz/docs/learn/basics/render-modes (checked 2026-11-04)

## Terminal page

### What it changes

`apps/terminal/src/main.tsx` and `packages/video/src/terminal/view.tsx` move to Svelte with the video package they import.

### Fit with Opsis

**In place.** The terminal page paints live terminal frames for the desktop and is built by `bun run terminal:bundle`; `TerminalView` is shared with the film.

**Missing.** Svelte versions of the page entry and `TerminalView`.

### Work

1. Move `TerminalView` with the Film elements.
2. Rewrite `apps/terminal/src/main.tsx` as a Svelte mount; keep its page protocol unchanged for `TerminalPainter` in `crates/capture`.
3. Update `agent-docs/terminal.md`.

### Sources

- See Film runtime.

## Editor

Only if [native-editor.md](native-editor.md) is not taken, or for the time before it replaces `apps/frontend`.

### What it changes

`apps/frontend` (14,920 lines of own `.tsx`, 7,288 lines of `.ts`, 8,871 lines of installed shadcn and React Flow UI components) moves to Svelte, with the stack below.

| Need             | Today                           | Svelte                                              | Maintainer                    |
| ---------------- | ------------------------------- | --------------------------------------------------- | ----------------------------- |
| Graph canvas     | `@xyflow/react` 12.12           | `@xyflow/svelte` 1.7.1                              | xyflow (React Flow's authors) |
| Building blocks  | `@base-ui/react`, shadcn (nova) | `@ark-ui/svelte` 5.25.1 on `@zag-js/svelte` 1.47    | Chakra team                   |
| Charts           | —                               | `layerchart` 2.9.2                                  | techniq and contributors      |
| Sortable lists   | `@dnd-kit/*`                    | `svelte-dnd-action` 0.9.80                          | isaacHagoel                   |
| Toasts           | `sonner`                        | `svelte-sonner` 1.2.3                               | wobsoriano                    |
| Canvas (marks)   | `react-konva`                   | `svelte-konva` 1.0.1                                | Konva                         |
| Command palette  | `cmdk`                          | Ark UI Combobox                                     | Chakra team                   |
| Resizable panels | `react-resizable-panels`        | Ark UI Splitter                                     | Chakra team                   |
| Router           | `@tanstack/react-router`        | own hash router (7 routes)                          | —                             |
| State            | zustand (18 stores)             | `$state` classes in `.svelte.ts`                    | —                             |
| Motion           | `motion/react`                  | `motion` (plain)                                    | Motion                        |
| Video player     | Dice UI on `media-chrome`       | `media-chrome` web components                       | Mux                           |
| Icons            | `lucide-react`, `morphicons`    | `lucide-svelte`; glyph sets through `glyphs:bundle` | Lucide                        |
| Tests            | Testing Library React           | `@testing-library/svelte` 5.4.2                     | Testing Library               |

### Fit with Opsis

**In place.** The data layer is mostly framework-free: `generated/bindings.ts`, `app/tauri.ts`, `workflow/`, `timeline/` maths, the stores' logic. Rust owns the document; the editor mirrors it.

**Missing.** Svelte versions of every view; the 43 shadcn components and 9 React Flow UI components restyled on Ark UI and Svelte Flow.

### Work

1. Islands: mount Svelte components inside React with `mount()` on a ref'd element, props through a `$state` object, so the editor works at every step.
2. Order: Inspector fields (`editor/form`), the Mixer, the Timeline (the playhead and meters write signals directly instead of re-rendering every frame), the graph canvas on Svelte Flow with Opsis's own cards and `edge-lanes.ts`, then dialogs, menus, command palette and toasts on Ark UI, then the shell, routes and windows.
3. Stores become `$state` classes; `document-mirror.ts` and `run-mirror.ts` write them.
4. Keep the canvas drawing only what is on screen (`onlyRenderVisibleElements` with the saved card shapes, as `93999ea0` does for React Flow) and stop drawing nodes twice in the minimap.
5. Remove React, the React router, Base UI, shadcn's React components, dnd-kit, cmdk, sonner, react-resizable-panels, react-konva, `motion/react`, zustand's React hooks, Testing Library React.
6. Rewrite `agent-docs/editor.md` and the §2 installed-code table in `AGENTS.md` (shadcn and React Flow UI rows go).

### Decisions

- **Styling.** Keep Tailwind and the zinc tokens in `index.css`; Ark UI is unstyled.
- **Base UI parity.** Map each Base UI component the editor uses to an Ark UI one before starting; list any without a match.

### Risks

- Svelte Flow has no React Flow UI component registry; the 9 components are written by hand.
- `svelte-dnd-action` and `svelte-sonner` are each mainly one maintainer.

### Sources

- Svelte Flow: https://svelteflow.dev, https://github.com/xyflow/xyflow (1.7.1, checked 2026-11-04)
- Ark UI: https://github.com/chakra-ui/ark (5.25.1), Zag: https://github.com/chakra-ui/zag (checked 2026-11-05)
- LayerChart: https://github.com/techniq/layerchart (2.9.2, checked 2026-11-04)
- svelte-dnd-action: https://github.com/isaacHagoel/svelte-dnd-action (0.9.80, checked 2026-11-05)
- svelte-sonner: https://github.com/wobsoriano/svelte-sonner (1.2.3, checked 2026-11-05)
- svelte-konva: https://github.com/konvajs/svelte-konva (1.0.1, checked 2026-11-04)
- media-chrome: https://github.com/muxinc/media-chrome (checked 2026-11-04)

## Alternatives

Checked against the same gates (stable release, more than one active maintainer, building blocks, tested, works with TypeScript 7 and the oxc tools).

**Discarded, on stable releases.**

- **React 19.3 with React Compiler 1.0**: passes every gate, but the compiler only memoises; the runtime stays `useState` and a virtual tree. It is the no-change fallback. https://react.dev/blog/2025/10/07/react-compiler-1
- **Vue 3.5.44**: needs TypeScript 6 for `.vue` files like Svelte does; its graph library (Vue Flow) has one maintainer and three releases in 12 months. https://github.com/vuejs/core, https://github.com/bcakmakoglu/vue-flow
- **Solid 1.9.18**: plain TSX keeps the toolchain, but 1.x has one active maintainer and is being replaced by 2.0; the graph library is a one-person port (`@dschz/solid-flow`); its testing library was last released stable in 2024; no stable 3D bindings. https://github.com/solidjs/solid
- **Preact 11.1.1 with `@preact/signals` 2.11.4**: no building-block library of its own; Base UI crashes under Preact ([base-ui #4670](https://github.com/mui/base-ui/issues/4670)); React Three Fiber needs React's reconciler. https://github.com/preactjs/preact
- **`@preact/signals-react` 3.12.0**: a second state model inside React that needs a Babel transform. https://github.com/preactjs/signals
- **Angular 22.2.4**: its template checking won't run on TypeScript 7. https://github.com/angular/angular
- **Leptos 0.8, Dioxus 0.7, Sycamore 0.9**: Rust to WebAssembly; they leave the TypeScript toolchain and have no accessible building blocks or graph editor, and the film needs three.js, shaders and lottie. https://github.com/leptos-rs/leptos, https://github.com/DioxusLabs/dioxus, https://github.com/sycamore-rs/sycamore
- **alien-signals 3.2.1**: a reactive core with no renderer. https://github.com/stackblitz/alien-signals
- **Motion Canvas 3.17.2**: no release since December 2024; its generator timeline replays a scene from its start to seek backwards; canvas only. https://github.com/motion-canvas/motion-canvas
- **Revideo 0.11.0**: the same generator and canvas model. https://github.com/redotvideo/revideo
- **Remix 3 components**: manual `update()` calls, neither signals nor a compiler, no building-block library. https://github.com/remix-run/remix

**Pre-release or experimental.**

- Solid 2.0.0-rc.21, with Kobalte 2.0.0-alpha.4 and Ark 6.0.0-next.0 for it. https://github.com/solidjs/solid/blob/next/documentation/solid-2.0/MIGRATION.md
- Vue 3.6.0-rc.14 with Vapor mode. https://github.com/vuejs/core/releases
- Qwik 2.0.0-rc.4. https://github.com/QwikDev/qwik
- `@lit-labs/signals` 0.3.0 on the TC39 Signals polyfill; TC39 Signals is a Stage 1 proposal. https://github.com/tc39/proposal-signals
- Ripple 0.6.1 and TSRX 0.5.5. https://github.com/Ripple-TS/ripple
- React Compiler through oxc (`react({ compiler: true })`), marked experimental by the Vite plugin. https://github.com/vitejs/vite-plugin-react/blob/main/packages/plugin-react/README.md
- TypeScript 7.1.0-beta, whose content mappers would let tsgo check `.svelte` files directly. https://devblogs.microsoft.com/typescript/announcing-typescript-7-1-beta/
