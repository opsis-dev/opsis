# Demoflow

A single Electron desktop app: a node-based editor ([xyflow](https://reactflow.dev)) that drives a real website with [Playwright](https://playwright.dev), turns the recording into a cinematic product demo with [Remotion](https://remotion.dev), and lets you stack motion-graphics elements, transitions, text, audio and FFmpeg nodes on top. Everything runs locally. See [PLAN.md](./PLAN.md) for the full spec.

## Quick start

```sh
bun install
bun start          # electron-forge start (Vite dev servers + Electron)
```

The full edition ships the engine (one Chrome Headless Shell for capture and render, plus FFmpeg/ffprobe). In dev and in the lite edition, install it once from Settings → Engine. Then **New project → Sample: TodoMVC → Run**: a live picture-in-picture shows what the capture browser is doing, and every capture can be replayed with its step timeline, and its Playwright trace (actions grouped by node, before/after screenshots, console, network, errors) opens in the Trace tab. The sample records `demo.playwright.dev/todomvc`, builds the auto-zoom camera track, adds a Remocn title and renders 16:9 + 9:16 MP4s into `<project>/.demoflow/out/`. Ready-made copies of every template live in [`examples/`](./examples) (File → Open).

## Scripts

| Script                                  | What it does                                                                                 |
| --------------------------------------- | -------------------------------------------------------------------------------------------- |
| `bun start`                             | Run the app in development                                                                   |
| `bun run package` / `bun run make`      | Package / build installers; `DEMOFLOW_EDITION=lite` leaves the engine out (default `full`)   |
| `bun run typecheck`                     | `tsc --noEmit` (TypeScript 6.0.3)                                                            |
| `bun run lint` / `bun run lint:fix`     | oxlint (`oxlint.config.ts`), incl. the frame-driven-animation guardrails                     |
| `bun run format` / `bun run format:fix` | oxfmt (`oxfmt.config.ts`)                                                                    |
| `bun run test`                          | vitest — compile(), graph, camera track, capture sync, cache, plus programatic-demo's suites |
| `bun run registry:ingest`               | Re-install the curated Remocn/snapcn/RemotionUI elements and regenerate their manifests      |
| `bun run routes`                        | Regenerate the TanStack Router route tree                                                    |
| `bun run studio`                        | Remotion Studio on the `DemoTimeline` composition                                            |
| `bun run examples`                      | Rewrite `examples/*/project.demoflow.json` from the templates                                |

## Architecture

```
Renderer (React 19 + shadcn/Base UI + xyflow)  ── preload (contextBridge: window.demoflow) ──  Main (Electron)
   canvas · palette · zod inspector · <Player>                                                   windows · menus · dialogs
   live compile (graph → TimelineSpec)                                                            projects · protocols · safeStorage
                                                                                                   │ MessagePort
                                                                                Worker (utilityProcess, one per job)
                                                          engine: validate → topo-sort → sha256 cache → phase lanes → events
                                                          capture (Playwright) · media (bundled FFmpeg) · render (@remotion/renderer)
```

- **Nodes never generate Remotion code.** They produce artifacts (files) and compile to a `TimelineSpec`; one fixed composition, `DemoTimeline`, renders any spec by looking components up in a registry. The editor's `<Player>` and the worker's `renderMedia()` use the same composition, so **preview = render**.
- **Browser nodes form a chain** (session in → session out). The Record/Still node closes it and the engine replays the whole chain in one continuous take, with programatic-demo's eased cursor glide, click log, clapperboard sync and camera track.
- **Caching:** every node is hashed (`type + params + upstream hashes + engine version`); executed nodes cache under `.demoflow/cache/<hash>`. Edit one headline and only that branch recompiles; captures re-run only when their chain changes.

```
src/
├─ main/       Electron main: window, menus, IPC, demoflow-file:// + demoflow-app:// protocols, projects, secrets, workers
├─ preload/    typed window.demoflow API
├─ shared/     runtime-neutral (UI + worker + Remotion bundle)
│  ├─ schema/      zod: Workflow v1, ActionLog, TimelineSpec, values, events
│  ├─ nodes/       node definitions (ports, zod params, compile()) — 167 nodes
│  ├─ elements/    element manifests (core + generated Remocn/snapcn/RemotionUI metadata)
│  ├─ compile/     graph → values → TimelineSpec
│  └─ templates/   project templates (incl. the §14 sample)
├─ worker/     utilityProcess: engine/, executors/, capture/ (Playwright), media/ (FFmpeg), render/ (Remotion)
├─ video/      Remotion root: DemoTimeline, registry, core elements, camera/ (programatic-demo), elements/ (Remocn, snapcn, RemotionUI)
└─ renderer/   React UI: routes (TanStack Router, hash history), editor/, store/ (Zustand + zundo), components/ (shadcn + React Flow UI)
```

## Node catalogue

Browser (18) · Capture (Record, Still, Login once) · Camera (Auto zoom, Manual zoom, Pan, Follow cursor, Idle speed-up, Speed ramp, Crop, Device frame, Backdrop, Motion blur) · Cursor (style, ripple, keystroke chips, spotlight, highlight box) · Text & overlays (title card, kinetic headline, lower third, callout, tooltip pinned to element, callout arrow, captions) · Transitions (6 from `@remotion/transitions`, 14 Remocn, 5 RemotionUI, light leak) · Backgrounds · Effects · Elements (UI blocks, charts, scenes, device mockups) · Audio (music bed, SFX, mix) · Timeline (sequence, stack, trim, loop, delay, speed, duration fit, branch) · Media/FFmpeg (import, trim, concat, speed, reverse, scale/pad, convert, GIF, extract audio, loudnorm, thumbnail, faststart, probe) · Output (render with 16:9/1:1/4:5/9:16 variants and follow-the-action reframing, still presets, export Remotion project).

Element nodes are generated from the registry: 109 curated Remocn/snapcn/RemotionUI components (`scripts/registry-ingest.list.ts`) plus 21 Demoflow core elements. Colour-preset duplicates of the same shader and components that duplicate each other across registries are left out on purpose.

## Decisions taken

- **Scaffold:** `create-electron-app` (Forge 8 + Vite 8), with the Remotion pieces of `remotion-dev/template-electron` ported in (prebuilt bundle, compositor staging, render) and rendering moved into a utilityProcess.
- **UI:** shadcn with **Base UI** and the **nova** preset; React Flow UI registry components on the canvas; TanStack Router with **file-based routes** over hash history.
- **Inspector:** a custom zod 4 → shadcn renderer (no form library), writing straight to the store.
- **FFmpeg:** GPLv3 `ffmpeg` and `ffprobe` builds from the pinned ffmpeg-static GitHub release, shipped as separate executables in the engine folder (`resources/engine/ffmpeg` in the full edition) with their licence.
- **Versions:** latest of everything; all `remotion`/`@remotion/*` pinned to the same exact version (4.0.535); TypeScript 6.0.3 until 7.1 (Remotion needs `typescript.sys`).

## Not in this build

Per the agreed scope (§14 + the full local §6 catalogue + M5 media nodes): TTS / transcription / music ducking (M4), installer signing, auto-update and the CI matrix (M6), and AI assist (M7) are not implemented. Captions are timed from a script (`Captions from script`), not transcribed.

## Licences

Demoflow is licensed under the [Apache License 2.0](./LICENSE).

Third-party notices are in [THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md). Remotion is used under its free licence (individuals and companies of up to 3 people); whoever owns rendered output at a larger company needs a Remotion company licence. Some snapcn and RemotionUI elements load fonts through `@remotion/google-fonts` (network); everything else is local.
