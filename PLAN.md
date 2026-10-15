# Demoflow — Build Spec

> A single Electron desktop app: a node-based editor (xyflow) that drives a real website with Playwright, turns the recording into a cinematic product demo with Remotion, and lets you stack motion-graphics components, transitions, text, audio and FFmpeg utility nodes on top. Everything runs locally — no server, no cloud, no monorepo.

*Researched October 2026. Versions and licences below were checked against official docs/repos at that time — re-check before shipping.*

---

## 0. TL;DR — the decisions

| Decision | Choice | Why |
|---|---|---|
| App shape | **One Electron app** (Electron Forge + Vite), single `package.json` | Forked from Remotion's official Electron template. No web version, no monorepo. |
| Canvas | **@xyflow/react 12.11.x** + **React Flow UI** (shadcn registry components) | Official shadcn-based node components (BaseNode, NodeHeader, LabeledHandle, …) install with `npx shadcn add https://ui.reactflow.dev/<name>` |
| Workflow format | **Versioned JSON graph** (n8n-style), compiled to a **TimelineSpec** JSON | Graph is for humans; TimelineSpec is the single prop object a fixed Remotion composition renders |
| Website capture | **Playwright ≥ 1.64** (Chromium), `page.screencast` API + action log | 1.59 added `page.screencast` (`start/stop`, `showActions`, `showChapter`, `showOverlay`); 1.64 added `fps` and action `style` |
| Video engine | **Remotion 4.0.53x** — `<Player>` for live preview, `@remotion/renderer` for export | Runs in the app's background Node process; full feature set, GPU-accelerated |
| Animations | **Remotion-native shadcn registries first** (Remocn, RemotionUI, snapcn, Remotion Elements); port web motion registries (Magic UI, Aceternity, Animate UI, React Bits…) to frame-driven | Remotion requires every animation to be a pure function of `useCurrentFrame()`; Framer Motion / CSS transitions don't survive a render |
| GSAP | **Not used** | Remotion's `interpolate()` / `spring()` / `Easing` cover everything; avoids GSAP's licence clause on visual animation builders |
| Video utility nodes | **FFmpeg bundled inside the app** (static binary per platform) | Spawned as a child process; nothing for the user to install |
| Base to fork | **`remotion-dev/template-electron`** (app shell) + **`programatic-demo`** (MIT; capture → camera → render) | Shell already packages Remotion correctly; programatic-demo already solves click log, auto-zoom camera, vector cursor, sync, stills, reels |
| Licensing | **Remotion free licence** (individual) | Unlimited use incl. commercial; pass `licenseKey: "free-license"` |

---

## 1. The wheels to copy

### 1.1 App shell + capture + camera (copy, don't write)

| Repo | Licence | Take |
|---|---|---|
| [remotion-dev/template-electron](https://github.com/remotion-dev/template-electron) | Remotion licence | **This becomes the repo.** Electron Forge + Vite + TypeScript + Tailwind; renders a Remotion composition from Electron, prebuilds the Remotion bundle at package time, stages the matching compositor binary, has save dialog + progress + cancel. |
| [ashrafchowdury/programatic-demo](https://github.com/ashrafchowdury/programatic-demo) (fork: [lukeharwood11/programatic-demo](https://github.com/lukeharwood11/programatic-demo)) | MIT | **Copy its engine into `src/worker/capture` and `src/video/camera`.** Playwright records a real flow and logs every click (position, bounding rect, departure time, typing end); Remotion builds a camera track from that log and composites it on a backdrop with a vector cursor + motion blur. Key files: `scripts/lib/selectors.ts` (accessible-name selector ladder), `scripts/lib/` cursor + flow helpers + session, `src/lib/click-log.ts` (types + timing constants), `src/lib/zoom.ts` (keyframe track), `src/lib/camera.ts` (pose interpolation), `src/DemoClip.tsx`, `src/WindowFrame.tsx`, `src/lib/still.ts` (social presets), `src/lib/reel.ts`, `src/lib/intro.ts`, `public/backdrops/`. Keep its MIT notice. |
| [AlanRoybal/openscreen-studio](https://github.com/AlanRoybal/openscreen-studio) | check repo | Reference for **click-driven auto-zoom + dwell-detection fallback**, and an editor that pre-places suggested zoom regions on a timeline. Same three-artifact pattern: video with cursor hidden, cursor telemetry file, session manifest. |
| connerkward/screenstudio-alternative-skill | MIT | Ideas for **idle speed-up**, **click-cluster zoom**, **keystroke overlay chips**, **9:16 follow-the-action export**. |

### 1.2 Node editor

| Source | Licence | Take |
|---|---|---|
| [React Flow UI components](https://reactflow.dev/components) | MIT | `base-node`, `node-header`, `labeled-handle`, `tooltip-node`, `node-status-indicator`, `zoom-slider`, `devtools`, etc. via shadcn CLI |
| [vercel-labs/workflow-builder-template](https://github.com/vercel-labs/workflow-builder-template) | OSS | Open-source visual workflow builder on React Flow. Copy UI ideas only: node palette, execution logs panel, "generate workflow from prompt". Ignore its Next.js/DB backend. |
| [01-ai/open-agentkit-builder](https://github.com/01-ai/open-agentkit-builder) | check repo | Small, readable React Flow + shadcn editor: start/end nodes, import/export, templates layout. |
| React Flow **Workflow Editor** template (Pro) | Pro subscription | Optional: ELKjs auto-layout, DnD sidebar, dark mode, sequential runner, Zustand store. |

### 1.3 Remotion-native motion registries (drop-in, already frame-driven)

| Registry | Install | What's in it |
|---|---|---|
| [Remocn](https://github.com/kapishdima/remocn) | `npx shadcn@latest add @remocn/blur-reveal` | 64+ components: text (blur reveal, typewriter, shimmer, slot-machine, matrix decode), transitions (chromatic-aberration wipe, frosted glass, spatial push, zoom-through), UI blocks (terminal, browser flow, charts, device mockup zoom), full scenes (launch trailer, changelog clip). Remotion 4, React 19, Tailwind 4. |
| [RemotionUI](https://remotionui.com) ([repo](https://github.com/riaz37/remotion-ui)) | `npx remotion-ui@latest init` / `npx remotion-ui add <name>` | 217 components: data story, karaoke captions, terminal simulator, device-mockup zoom, lower third, grid-pixelate wipe, logo reveal, code reveal, mesh-gradient BG, social clips. |
| [snapcn](https://github.com/snapcndev/snapcn) | `npx shadcn add @snapcn/<name>` | MIT. Built **for product demo videos**: text reveals, device frames, terminal sessions, logo stings, whole scenes. Themes from your shadcn tokens. |
| [Remotion Elements](https://www.remotion.dev/elements) | copy source | Official: backgrounds (liquid contours, paper, starburst), hand-drawn markers (circle, strike, cross-off, highlight), news-headline highlight, **Timed Captions (now free)**. |
| Official `@remotion/*` | npm | `transitions`, `captions`, `install-whisper-cpp`, `openai-whisper`, `elevenlabs`, `motion-blur`, `light-leaks`, `noise`, `shapes`, `paths`, `lottie`, `rive`, `three`, `skia`, `gif`, `google-fonts`, `fonts`, `animated-emoji`, `sfx`, `layout-utils`, `media`, `media-parser`, `zod-types`, `tailwind-v4` |

### 1.4 Web motion registries (great look, need porting)

Directories to mine: [ui.shadcn.com/directory](https://ui.shadcn.com/directory), [registry.directory](https://registry.directory), [21st.dev motion registries](https://21st.dev/community/shadcn-directory/motion), [shadcnregistry.com](https://shadcnregistry.com), [shoogle.dev/directory](https://shoogle.dev/directory).

High-value sources: **Magic UI**, **Aceternity UI**, **Animate UI**, **React Bits**, **Motion Primitives**, **Cult UI**, **SmoothUI**, **Skiper UI**, **Kibo UI**, **AnimBits**, animated icon sets (**pqoqubbw/icons**, **hugeicons-animated**, **heroicons-animated**). Skip GSAP-based registries (e.g. Pace UI).

These animate on the wall clock (Motion/Framer, CSS keyframes) → **use them as-is in the editor UI**, but **port** them before they become video nodes (recipe in §8.3).

### 1.5 Optional extras

| Thing | Use it for |
|---|---|
| [Remotion Editor Starter](https://www.remotion.dev/docs/editor-starter) ($600 one-time) | Timeline editor, canvas, font picker, asset handling, undo/redo, captions. Only if you want an editable **timeline view** next to the node graph. |
| [openvideodev/react-video-editor](https://github.com/openvideodev/react-video-editor) | Reference for WebGL shader transitions. |
| [Mediabunny](https://mediabunny.dev) (MPL-2.0) | Fast media probing / thumbnails / remux inside the UI without spawning FFmpeg. Nice-to-have. |

---

## 2. Architecture

Three processes, one app:

```
┌─────────────────────── Renderer process (React UI) ───────────────────────┐
│  xyflow canvas · palette · inspector (zod → form) · <Player> preview      │
│  · run panel                                                              │
│        │ Workflow JSON (v1)                                               │
└────────┼──────────────────────────────────────────────────────────────────┘
         │ IPC via preload (contextBridge)
┌────────▼───────────── Main process ───────────────────────────────────────┐
│  windows · menus · dialogs · project folder I/O · spawns/kills workers    │
└────────┬──────────────────────────────────────────────────────────────────┘
         │ MessagePort
┌────────▼───────────── Worker (Electron utilityProcess, Node) ─────────────┐
│  Engine: validate → topo-sort → hash/cache → run phases → emit events     │
│                                                                           │
│  A CAPTURE  Playwright (Chromium) → recording.mp4 + actions.json + stills │
│  B MEDIA    bundled FFmpeg: trim, speed, concat, audio, TTS, transcribe   │
│  C COMPILE  graph → TimelineSpec JSON (pure function, unit-testable)      │
│  D RENDER   @remotion/renderer.renderMedia(DemoTimeline) → mp4/webm/gif   │
│  E POST     FFmpeg: GIF palette, loudnorm, faststart, aspect variants     │
└───────────────────────────────────────────────────────────────────────────┘
```

**Core idea:** nodes never generate Remotion code. They produce **artifacts** (files on disk) and **timeline fragments** (JSON). One fixed composition, `DemoTimeline`, renders any TimelineSpec by looking components up in a **registry**. Preview (`<Player>` in the UI) and export (`renderMedia` in the worker) use the same composition, so **preview = render**.

There is no server. Remotion's renderer briefly serves the bundle on localhost to its own headless Chrome during a render; that's internal and automatic.

---

## 3. Electron specifics

### 3.1 Processes

- **Main**: window, menus, file dialogs, project open/save, auto-update, spawns the worker.
- **Preload**: exposes a typed `window.demoflow` API (`runWorkflow`, `cancel`, `onEvent`, `openProject`, `pickElement`, …) via `contextBridge`. `contextIsolation: true`, `nodeIntegration: false`.
- **Worker**: an Electron `utilityProcess` running the engine, so captures and renders never freeze the UI. One worker per run; cancel = kill + abort signals. (The template renders in main — move it out.)
- **Renderer**: plain Vite + React 19 SPA. Use a simple state-based screen switch or TanStack Router in client-only mode (Projects → Editor → Settings). No SSR, no framework.

### 3.2 Remotion packaging rules (from Remotion's Electron docs)

- Don't call `bundle()` at runtime in the packaged app. Build the Remotion bundle at package time and load the prebuilt directory. In dev, bundling per render is fine.
- Pass `binariesDirectory` pointing at the compositor package inside `app.asar.unpacked`; native binaries can't load from inside `app.asar`.
- Ship the compositor matching each target OS/arch (separate builds for glibc vs musl Linux).
- Use Remotion's pinned **Chrome Headless Shell** (it can download on first run via `ensureBrowser()` into app data); full Chrome is documented as less deterministic.
- Set `licenseKey: "free-license"` on `renderMedia()`.
- Use `gl: "angle"` for GPU rasterisation; programatic-demo measured ~10× faster renders than software.

### 3.3 Bundled binaries

| Binary | How it ships | Notes |
|---|---|---|
| **FFmpeg + ffprobe** | Static build per platform in Forge `packagerConfig.extraResource` (e.g. from `ffmpeg-static` / `@ffprobe-installer`, or your own builds). Resolve with `app.isPackaged ? path.join(process.resourcesPath, "ffmpeg", bin) : devPath`. ~70–80 MB per platform. | Must be a **full** build (programatic-demo's sync uses the `signalstats` filter, which Remotion's internal FFmpeg lacks). `ffmpeg-static` fetches the binary for the machine you build on → build each OS in its own CI job (GitHub Actions matrix). |
| **Remotion compositor** | Staged into `app.asar.unpacked` by the template's package step | Contains its own ffmpeg/ffprobe used by Remotion internally — leave it alone. |
| **Remotion Chrome Headless Shell** | Downloaded on first run (or bundled) | Used only for rendering. |
| **Playwright Chromium** | `playwright-core` in deps; set `PLAYWRIGHT_BROWSERS_PATH` to app data and install Chromium on first run (show a progress screen), or bundle it | Used only for capture. For the headed "Login once" step you can use the user's installed Chrome via `channel: "chrome"`. |
| **whisper.cpp** (optional) | `@remotion/install-whisper-cpp` downloads binary + model on first use into app data | For local captions. |

First-run screen: "Downloading render engine… capture browser… (optional) speech model". Everything lands in `app.getPath("userData")`.

### 3.4 Storage

A project is a folder:

```
my-launch-video/
├─ project.demoflow.json      # the Workflow (git-friendly)
├─ assets/                    # imported images, audio, logos
├─ secrets/                   # storageState.json for logged-in shoots (gitignored; encrypt with safeStorage)
└─ .demoflow/
   ├─ cache/<node-hash>/      # recordings, action logs, intermediate media
   └─ out/                    # final renders
```

No database. A small JSON file in `userData` keeps the recent-projects list.

---

## 4. Repo layout (single Electron app)

```
demoflow/                         # forked from remotion-dev/template-electron
├─ package.json                   # one package; all remotion/@remotion/* pinned to the same exact version
├─ forge.config.ts                # makers, extraResource (ffmpeg), asar.unpack (compositor), prebuild Remotion bundle
├─ vite.main.config.ts · vite.preload.config.ts · vite.renderer.config.ts · vite.worker.config.ts
├─ remotion.config.ts             # webpack alias mirroring tsconfig paths (Remotion ignores tsconfig paths)
├─ components.json                # shadcn (UI + registries)
├─ resources/
│  └─ ffmpeg/<platform>/          # ffmpeg, ffprobe (filled by CI per OS)
├─ public/backdrops/              # from programatic-demo
└─ src/
   ├─ main/                       # main process: windows, menus, IPC handlers, project fs, spawn worker
   ├─ preload/                    # contextBridge API
   ├─ shared/                     # imported by BOTH renderer and worker — no Node-only or browser-only deps
   │  ├─ schema/                  # zod: Workflow v1, ActionLog, TimelineSpec, events
   │  └─ nodes/                   # node DEFINITIONS: ports, params (zod), category, compile()
   ├─ worker/                     # utilityProcess entry (Node-only)
   │  ├─ engine/                  # DAG runner, hashing cache, phases, events, cancellation
   │  ├─ executors/               # execute() per node type (capture/media/render), keyed by node type
   │  ├─ capture/                 # Playwright driver: selector ladder, synthetic cursor, sync, session (from programatic-demo)
   │  ├─ media/                   # ffmpeg/ffprobe wrapper (spawn, progress parsing, cancel)
   │  └─ render/                  # renderMedia / renderStill wrappers, bundle location, binariesDirectory
   ├─ video/                      # Remotion root — bundled for export, imported by <Player> for preview
   │  ├─ Root.tsx                 # registers DemoTimeline (+ Still)
   │  ├─ DemoTimeline.tsx
   │  ├─ camera/                  # zoom.ts, camera.ts, click-log.ts, DemoClip, WindowFrame (from programatic-demo)
   │  ├─ registry/                # ElementManifest types + registry.generated.ts
   │  └─ elements/                # remocn/ · remotion-ui/ · snapcn/ · ported/
   └─ renderer/                   # React UI
      ├─ app/                     # screens: Projects, Editor, Settings, First-run
      ├─ editor/                  # canvas, palette, inspector, preview dock, run panel, element picker
      ├─ store/                   # Zustand (+ zundo for undo/redo)
      └─ components/ui/           # shadcn
```

**Why `shared/nodes` holds definitions but not `execute()`:** the renderer needs ports/params/compile() to draw nodes, build forms and preview; only the worker may import Playwright/FFmpeg/`@remotion/renderer`. Executors live in `worker/executors` keyed by node type. This keeps Node-only code out of the UI bundle.

---

## 5. Workflow JSON (v1)

```ts
// src/shared/schema/workflow.ts
export type PortType =
  | "session"      // live browser context handle (capture phase only)
  | "recording"    // { video: Artifact, actions: ActionLog, viewport, dpr }
  | "clip"         // timeline fragment with duration
  | "scene"        // ordered clips + overlays
  | "audio" | "captions" | "image" | "media" | "timeline" | "file";

export interface Workflow {
  version: 1;
  id: string;
  name: string;
  settings: { width: number; height: number; fps: 30 | 60; background?: string; theme?: "light" | "dark" };
  variables?: Record<string, string>;                // e.g. {{BASE_URL}}, {{PRODUCT_NAME}}
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
}

export interface WorkflowNode {
  id: string;
  type: string;                 // "browser.click", "camera.autoZoom", "fx.remocn.blurReveal" …
  position: { x: number; y: number };
  params: Record<string, unknown>;   // validated by the node def's zod schema
  label?: string;
  disabled?: boolean;
}

export interface WorkflowEdge {
  id: string;
  source: string; sourceHandle: string;   // port id
  target: string; targetHandle: string;
}
```

```ts
// src/shared/schema/actions.ts — the contract between capture and camera
export interface ActionLog {
  viewport: { width: number; height: number };
  dpr: number;
  t0: number;                         // ms epoch of the sync point / first frame
  actions: Array<{
    id: string;
    kind: "click" | "type" | "hover" | "scroll" | "navigate" | "wait" | "chapter";
    t: number;                        // ms from t0
    tEnd?: number;                    // e.g. typing end
    point?: { x: number; y: number };
    rect?: { x: number; y: number; w: number; h: number };
    cluster?: string;                 // groups beats for the camera
    label?: string;                   // human label, also used for captions/chapters
    selectorUsed?: string;            // which rung of the selector ladder won
  }>;
}
```

```ts
// src/shared/schema/timeline.ts — what DemoTimeline renders
export interface TimelineSpec {
  width: number; height: number; fps: number; durationInFrames: number;
  tracks: Array<{
    id: string;
    items: Array<{
      id: string;
      from: number; durationInFrames: number;
      component: string;              // registry key, e.g. "core/recording", "remocn/blur-reveal"
      props: Record<string, unknown>; // validated against the component's zod schema
      enter?: TransitionRef; exit?: TransitionRef;
    }>;
  }>;
  audio?: Array<{ src: string; from: number; volume?: number | string }>;
}
export interface TransitionRef { kind: string; durationInFrames: number; params?: Record<string, unknown> }
```

Example (abridged):

```json
{
  "version": 1,
  "id": "wf_signup_demo",
  "name": "Signup demo",
  "settings": { "width": 1920, "height": 1080, "fps": 60, "theme": "dark" },
  "variables": { "BASE_URL": "https://staging.acme.dev" },
  "nodes": [
    { "id": "s",  "type": "browser.session",  "position": {"x":0,"y":0},   "params": { "viewport": [1920,1080], "dpr": 2, "storageState": "secrets/acme.json" } },
    { "id": "n1", "type": "browser.navigate", "position": {"x":240,"y":0}, "params": { "url": "{{BASE_URL}}/signup", "after": 700 } },
    { "id": "n2", "type": "browser.type",     "position": {"x":480,"y":0}, "params": { "target": "Email", "text": "ada@acme.dev", "cluster": "form" } },
    { "id": "n3", "type": "browser.click",    "position": {"x":720,"y":0}, "params": { "target": "Create account", "cluster": "submit", "after": 1200 } },
    { "id": "rec","type": "capture.record",   "position": {"x":960,"y":0}, "params": { "fps": 60 } },
    { "id": "cam","type": "camera.autoZoom",  "position": {"x":1200,"y":0},"params": { "maxScale": 1.8, "hold": 1300, "motionBlur": true } },
    { "id": "t1", "type": "fx.remocn.blurReveal", "position": {"x":1200,"y":200}, "params": { "text": "Sign up in 10 seconds", "duration": 60 } },
    { "id": "seq","type": "timeline.sequence","position": {"x":1440,"y":0},"params": { "transition": "remocn/zoom-through" } },
    { "id": "out","type": "output.render",    "position": {"x":1680,"y":0},"params": { "codec": "h264", "variants": ["16:9","1:1","9:16"] } }
  ],
  "edges": [
    { "id":"e1","source":"s","sourceHandle":"session","target":"n1","targetHandle":"session" },
    { "id":"e2","source":"n1","sourceHandle":"session","target":"n2","targetHandle":"session" },
    { "id":"e3","source":"n2","sourceHandle":"session","target":"n3","targetHandle":"session" },
    { "id":"e4","source":"n3","sourceHandle":"session","target":"rec","targetHandle":"session" },
    { "id":"e5","source":"rec","sourceHandle":"recording","target":"cam","targetHandle":"recording" },
    { "id":"e6","source":"t1","sourceHandle":"clip","target":"seq","targetHandle":"in1" },
    { "id":"e7","source":"cam","sourceHandle":"clip","target":"seq","targetHandle":"in2" },
    { "id":"e8","source":"seq","sourceHandle":"timeline","target":"out","targetHandle":"timeline" }
  ]
}
```

**Design note:** browser-action nodes form a **chain** (session in → session out). The `capture.record` node closes the chain: the engine replays the whole chain once inside a single recording, so the editor shows actions as separate nodes while capture stays one continuous take. Offer a **Group** (subflow) node so a 20-step flow collapses into one box.

---

## 6. Node catalogue

Definition (shared — UI + worker):

```ts
// src/shared/nodes/browser/click.ts
export const clickNode = defineNode({
  type: "browser.click",
  category: "Browser",
  phase: "capture",
  inputs:  { session: "session" },
  outputs: { session: "session" },
  params:  z.object({
    target: z.string(),
    cluster: z.string().optional(),
    after: z.number().default(600),
    label: z.string().optional(),
  }),
});
```

Executor (worker only):

```ts
// src/worker/executors/browser/click.ts
export default defineExecutor(clickNode, async (ctx, { session }, p) => {
  await ctx.capture.click(session, p);
  return { session };
});
```

| Category | Nodes |
|---|---|
| **Browser** (capture phase) | Session (viewport, DPR, UA/mobile, storageState, locale, colour scheme), Navigate, Click, Type, Hover, Scroll (to element / by px, smooth), Select, Upload file, Press keys, Wait (element / network idle / ms), Assert (text visible → fail fast), Hoist (pre-resolve a target during a moving beat), Set state (`page.evaluate`), Mock route (`page.route`), Freeze clock (`page.clock`), Hide elements (cookie banners, chat widgets), Chapter (`screencast.showChapter`), Overlay HTML (`screencast.showOverlay`) |
| **Capture** | Record (screencast → mp4 + ActionLog), Still (screenshot at DPR 2–4, region by name/css/rect, window/cutout/bleed looks), Login once (headed, saves storageState) |
| **Camera** | Auto zoom (from clusters), Manual zoom region, Pan, Follow cursor, Idle speed-up, Speed ramp, Crop, Device frame (browser window / laptop / phone), Backdrop (image/gradient/flat), Motion blur |
| **Cursor** | Cursor style (vector, size, smoothing), Click ripple, Keystroke chips, Spotlight / dim-around-element, Highlight box (from ActionLog rects) |
| **Text & overlays** | Title card, Kinetic headline (word-by-word with inline emphasis markup), Lower third, Callout arrow, Tooltip pinned to element rect, Hand-drawn marker, Code reveal, Terminal session, Number counter, Logo sting |
| **Transitions** | `@remotion/transitions` (fade, slide, wipe, flip, clock-wipe), Remocn (chromatic wipe, frosted glass, spatial push, zoom-through), RemotionUI (grid pixelate), light leaks |
| **Backgrounds** | Mesh gradient, liquid contours, noise, starburst, paper, solid, image, video loop |
| **Audio** | Music bed (ducking under VO), SFX (`@remotion/sfx`), TTS voiceover (ElevenLabs / OpenAI / local Kokoro), Transcribe (whisper.cpp local / OpenAI / ElevenLabs) → Captions, Captions style (word-by-word, karaoke) |
| **Timeline** | Sequence (A→B with transition), Stack (layers), Trim, Loop, Delay, Duration fit (stretch to voiceover length), Branch by variable (A/B copy) |
| **Media / FFmpeg** | Import media, Trim, Concat (`-c copy` when codecs match), Speed, Reverse, Scale/pad, Convert (h264/hevc/vp9/prores), GIF (palettegen/paletteuse), Extract audio, Loudnorm, Thumbnail, Faststart, Probe |
| **Output** | Render (codec, CRF, fps), Variants (16:9, 1:1, 4:5, 9:16 with follow-the-action reframing), Still presets (wide 3840×2160, og 2400×1260, square, portrait, story), Export Remotion project (hand-edit the code) |
| **AI (later)** | Script → flow (LLM writes Browser nodes from a sentence + DOM snapshot), Auto-chapter (labels from ActionLog), Copywriter (headline variants) |

---

## 7. Engine (in the worker)

### 7.1 Execution

1. **Validate** with zod; check port types on every edge; detect cycles.
2. **Topo-sort**; group by phase (capture → media → compile → render → post).
3. **Hash** each node = `sha256(type + params + upstream hashes + engineVersion)`. If `.demoflow/cache/<hash>` exists, skip. Change one headline → only that card re-renders (programatic-demo does the same per reel segment).
4. **Run** with a concurrency limiter (`p-limit`). Capture is memory-bound — programatic-demo found ~3 Chromium+encoder workers comfortable on 16 GB; 8 ran out of memory.
5. **Emit events** over the MessagePort: `node:start | node:progress | node:log | node:done | node:error | run:done`. The UI colours nodes with React Flow's `node-status-indicator`.
6. **Cancel** = abort signal to every executor (Playwright `browser.close()`, `renderMedia` `cancelSignal`, FFmpeg `kill`), then kill the utilityProcess.

### 7.2 Compile (graph → TimelineSpec)

A pure function in `src/shared/nodes`: every node with a `compile()` contributes items to tracks. It runs in the UI too, so the Player preview updates live as you edit — before anything is rendered. Unit-test it heavily.

### 7.3 Render

```tsx
// src/video/Root.tsx
<Composition
  id="DemoTimeline"
  component={DemoTimeline}
  schema={timelineSpecSchema}
  calculateMetadata={({ props }) => ({
    durationInFrames: props.durationInFrames, fps: props.fps, width: props.width, height: props.height,
  })}
  defaultProps={emptySpec}
/>
```

`DemoTimeline` maps tracks → `<Sequence>` / `<TransitionSeries>`, and each item → `registry[item.component]`. The worker calls `selectComposition()` + `renderMedia()` against the prebuilt bundle with the spec as `inputProps`.

---

## 8. Animation library strategy

### 8.1 The rule

Every animation must be a pure function of `useCurrentFrame()`. Remotion's docs say so explicitly; Framer Motion has no integration; Tailwind `animate-*` classes and CSS transitions don't render correctly. Add an ESLint rule banning `framer-motion`, `motion/react`, `gsap` and `animate-` classes inside `src/video/`.

### 8.2 Registry manifest (the inspector builds itself)

```ts
// src/video/registry/types.ts
export interface ElementManifest<P> {
  key: string;                 // "remocn/blur-reveal"
  title: string;
  category: "text" | "transition" | "background" | "ui" | "scene" | "overlay" | "data";
  source: { registry: string; url: string; license: string };
  schema: z.ZodType<P>;        // use @remotion/zod-types for colours etc.
  defaultProps: P;
  defaultDurationInFrames: number;
  component: React.FC<P>;
  thumbnail?: string;          // rendered via renderStill by a dev script
}
```

`scripts/registry-ingest.ts` (dev-time only): runs `npx shadcn add` / `npx remotion-ui add` into `src/video/elements/<registry>/`, generates a manifest stub per component (props → zod via ts-morph), renders a thumbnail with `renderStill`, and writes `registry.generated.ts`. The node palette is generated from this file.

### 8.3 Porting a web motion component (Magic UI, Aceternity, etc.)

1. Copy the component from the registry into `src/video/elements/ported/`.
2. Replace `motion.div` + `animate/initial/transition` with plain elements whose style is computed from `const frame = useCurrentFrame()` via `interpolate()` / `spring({ frame, fps })` / `Easing`.
3. Replace `useEffect` timers, `requestAnimationFrame` and `Math.random()` with `random(seed)` from `remotion`, and derive everything from `frame`.
4. Replace CSS `@keyframes` with frame-computed styles (or `animation-play-state: paused` + a negative `animation-delay` derived from the frame — Remotion documents this).
5. Fonts via `@remotion/google-fonts` / `@remotion/fonts`; images via `<Img>` + `staticFile`.
6. Add an `ElementManifest`, render one test frame, and visually diff it in CI.

Mechanical enough for a coding agent; install Remotion's official agent skills (`npx skills add remotion`) to help.

### 8.4 Designer assets

Lottie (`@remotion/lottie`) and Rive (`@remotion/rive`) for animations made in After Effects / Rive. No GSAP anywhere.

---

## 9. Capture engine details (Playwright)

- **API**: `await page.screencast.start({ path, size, fps, quality, onFrame })` → `stop()`. `size` is a max bound (aspect kept; if omitted it fits 800×800 — always set it). `fps` defaults to 25; Firefox/WebKit cap near 25 → **Chromium only**.
- **Built-in decorations**: `showActions({ cursor, duration, position, style: { point, highlight, title } })`, `showChapter(title, { description, duration })`, `showOverlay(html, { duration })`. Useful for a "quick mode"; for cinematic output, **turn them off** and draw cursor/highlights in Remotion from the ActionLog (sharp at any resolution, restylable without re-shooting).
- **Resolution ceiling**: screencast frames are CSS-viewport pixels regardless of DPR, so video tops out at the viewport size (programatic-demo outputs 2560 wide, not 4K). Record at a larger viewport (e.g. 2560×1440) and let the camera zoom into it. Stills use `screenshot()` at `deviceScaleFactor` 2–4 for true 4K.
- **Sync**: Playwright doesn't report exactly when capture starts/stops. Default: programatic-demo's **clapperboard** — paint a full-screen magenta frame ~160 ms at t0, find it with the bundled FFmpeg's `signalstats`, trim to it. Alternative: capture via `onFrame` (each frame carries a timestamp), write frames yourself and align with action timestamps directly.
- **Selectors**: copy the accessible-name **selector ladder** (role+name → label → placeholder → test id → containing text → bare text), log which rung matched, plus per-flow `targets` overrides and a `css()` escape hatch.
- **Element picker** (editor feature): opens a headed browser; hover highlights, click writes the best locator into the selected node via IPC.
- **Synthetic cursor**: hide the OS cursor; glide a synthetic one along eased, distance-aware paths; log departure time so the camera can lead in.
- **Determinism**: `page.clock` to freeze time, `page.route` to mock APIs, fixed fonts, cookie-banner/chat-widget hiding, wait for elements instead of fixed sleeps.
- **Headless by default**: a visible window losing focus blurs inputs and pauses animations. Headed only for login and debugging.
- **Sessions**: one headed "Login once" node → `storageState.json` in the project's `secrets/`, encrypted with Electron `safeStorage`; reused across shoots.

---

## 10. Editor UX

- **Canvas**: React Flow UI `BaseNode` + `NodeHeader` + `LabeledHandle`; handle colours per `PortType`; `isValidConnection` enforces types; ELKjs auto-layout; minimap; snap-to-grid; group/subflow nodes; copy/paste; undo/redo (`zundo` on the Zustand store).
- **Palette**: generated from node definitions + element registry; searchable (`cmdk`); animated thumbnails.
- **Inspector**: auto-form from zod (`@autoform/react` or shadcn `Form`); colour pickers via `@remotion/zod-types` (`zColor`).
- **Preview dock**: `<Player>` bound to the live-compiled TimelineSpec; selecting a node seeks to its item; scrubbing highlights the active node. Recordings that haven't been captured yet show a placeholder card.
- **Run panel**: per-node logs, cached/fresh badges, artifacts with inline video/image preview, "re-run from here", open output folder.
- **Templates**: Feature launch, Changelog clip, Onboarding walkthrough, Social teaser (9:16), Before/after.
- **Editor chrome**: Magic UI / Animate UI / Motion Primitives used as-is for the app's own interface.
- **Desktop niceties**: native menus + shortcuts, recent projects, drag files onto canvas → Import media node, dock/taskbar progress during renders, notification when a render finishes.

---

## 11. Licences

| Item | Status for you (solo individual, Oct 2026) |
|---|---|
| **Remotion** | **Free licence**: individuals and companies up to 3 people, unlimited use, commercial use allowed. Pass `licenseKey: "free-license"`. Revisit only if you sell the app to companies with 4+ people (the licence is owed by whoever owns the rendered output). |
| Remotion Editor Starter | $600 one-time — optional, not needed |
| FFmpeg | Fine for personal use. If you ever distribute the app, use an LGPL build (or open-source the app under GPL if using a GPL build). |
| Playwright | Apache-2.0 |
| @xyflow/react | MIT (Pro templates are a paid subscription — optional) |
| programatic-demo, snapcn | MIT — keep their licence notices |
| Remocn, RemotionUI, React Flow UI, web motion registries | Check each repo's licence when ingesting; record it in the manifest `source.license` |
| Mediabunny (optional) | MPL-2.0 |
| GSAP | Not used |

---

## 12. Build order (milestones)

1. **M0 — Skeleton (2–3 days)**: fork `template-electron`; move rendering into a `utilityProcess` worker; copy programatic-demo's capture + camera code; bundle FFmpeg for your OS; prove `record → render` from a button in the app.
2. **M1 — Schema + engine (1 week)**: zod Workflow v1, ActionLog, TimelineSpec; DAG runner with hashing cache + events + cancel; definitions/executors for Browser, Capture, Camera, Output nodes; run a JSON file end to end.
3. **M2 — Editor (1–2 weeks)**: xyflow canvas with React Flow UI nodes; palette; zod inspector; live `<Player>` preview of the compiled spec; run panel; project open/save.
4. **M3 — Motion library (ongoing)**: registry-ingest script; ship Remocn + snapcn + RemotionUI + Remotion Elements + `@remotion/transitions`; port the top 20 web components.
5. **M4 — Audio & captions**: TTS, whisper.cpp captions, music ducking, SFX.
6. **M5 — Media nodes**: FFmpeg wrapper with progress parsing; GIF, concat, loudnorm, aspect variants with follow-the-action reframing.
7. **M6 — Packaging**: installers per OS via Forge makers, CI matrix (per-OS FFmpeg + compositor), first-run downloads screen, auto-update, code signing.
8. **M7 — AI assist**: prompt → flow, auto-chapters, copy variants, element picker suggestions.

---

## 13. Gotchas

- Pin every `remotion` / `@remotion/*` package to the same exact version (no `^`); mismatches break silently.
- Remotion's bundler ignores tsconfig `paths` — mirror aliases in `remotion.config.ts` (webpack override), or use relative imports inside `src/video/`.
- Code in `src/video/` and `src/shared/` must be importable by both the Remotion webpack bundle and the Vite renderer — no Node built-ins there.
- `delayRender()` for anything async (fonts, data) or you get blank frames.
- Use `@remotion/media` `<Video>`/`<Audio>` for recordings (Remotion's current recommendation).
- Load media in `<Player>` via a custom protocol (e.g. `demoflow-file://`) registered in main, not `file://` — keeps web security on and works with the cache folder.
- Cut on still beats, never mid-pan — a cut during camera motion reads as a mistake.
- Stitching with `-c copy` requires identical geometry/codec params — probe both and refuse otherwise.
- Smooth CSS gradients band under h264; programatic-demo uses baked, grained backdrop images for this reason.
- Overriding Remotion's render concurrency can make it slower — programatic-demo measured 6 workers 33% slower than the default.
- Fixed millisecond waits for UI are flaky; wait for elements.
- Flows that mutate the same app state must not run in parallel (`mutates` key → serial lane).
- On Linux, Chrome Headless Shell needs extra shared libraries; `chromeMode: "chrome-for-testing"` can use GPU drivers.

---

## 14. One-shot prompt for a coding agent

> Build **Demoflow**, a single Electron app (no monorepo), following `demoflow-build-spec.md` exactly.
> 1. Clone `remotion-dev/template-electron` as the repo. Add a fourth Vite build for a `utilityProcess` worker (`src/worker/index.ts`) and move all rendering from main into it; main spawns/kills it and relays events over a MessagePort; preload exposes a typed `window.demoflow` API.
> 2. Clone `ashrafchowdury/programatic-demo` (MIT) into a temp dir. Copy its `scripts/lib/*` capture code into `src/worker/capture/`, its `src/lib/{zoom,camera,click-log}.ts`, `DemoClip.tsx`, `WindowFrame.tsx` into `src/video/camera/`, and `public/backdrops/` into `public/backdrops/`. Keep the MIT notice.
> 3. Create `src/shared/schema/` with the zod types in §5 (Workflow, ActionLog, TimelineSpec, run events) and export inferred TS types.
> 4. Create `src/shared/nodes/` with `defineNode()` (ports, zod params, category, phase, optional `compile()`), and `src/worker/executors/` with `defineExecutor()`. Implement: browser.session/navigate/click/type/hover/scroll/wait, capture.record, capture.still, camera.autoZoom, fx.title, timeline.sequence, output.render.
> 5. Create `src/worker/engine/`: validation, topo sort, per-node sha256 cache under `<project>/.demoflow/cache`, phase ordering, events, abort signals.
> 6. Create `src/worker/media/`: FFmpeg/ffprobe wrapper resolving the binary from `process.resourcesPath/ffmpeg` when packaged; add `ffmpeg-static` for dev and configure Forge `extraResource`.
> 7. Create `src/video/`: `DemoTimeline` rendering a TimelineSpec via a component registry; register the programatic-demo clip as `core/recording`; add `@remotion/transitions` fade/slide/wipe. Mirror path aliases in `remotion.config.ts`.
> 8. Run `npx shadcn@latest add @remocn/blur-reveal` plus 10 more Remocn text/transition components into `src/video/elements/remocn/` and write an `ElementManifest` for each.
> 9. Create `src/renderer/`: React 19 + Tailwind 4 + shadcn; @xyflow/react 12 with React Flow UI `base-node`, `node-header`, `labeled-handle`, `node-status-indicator`; Zustand + zundo; palette from node defs + registry; zod auto-form inspector; `<Player>` preview of the live-compiled spec (media via a custom `demoflow-file://` protocol); run panel; Projects screen; project folders as in §3.4.
> 10. Ship a sample project that records a public demo site, auto-zooms, adds a Remocn title, and renders 16:9 + 9:16 MP4s.
> Rules: every animation is a pure function of `useCurrentFrame()`; no GSAP, Framer Motion or Tailwind `animate-*` inside `src/video/`; pin all Remotion packages to one exact version; Playwright ≥ 1.64, Chromium only; `licenseKey: "free-license"`; vitest tests for `compile()` and the camera track.

---

## Sources

- Remotion Electron template — https://github.com/remotion-dev/template-electron · docs https://www.remotion.dev/docs/electron
- programatic-demo — https://github.com/ashrafchowdury/programatic-demo · https://github.com/lukeharwood11/programatic-demo
- Playwright Screencast API — https://playwright.dev/docs/api/class-screencast
- Remotion releases — https://github.com/remotion-dev/remotion/releases
- Remotion renderMedia (binariesDirectory, chromeMode) — https://www.remotion.dev/docs/renderer/render-media
- Remotion third-party animation rules — https://www.remotion.dev/docs/third-party
- Remotion captions & API index — https://remotion.dev/docs/captions/api · https://remotion.dev/api
- Remotion Elements — https://convert.remotion.dev/elements · https://www.remotion.dev/elements/libraries
- Remotion Editor Starter — https://www.remotion.dev/docs/editor-starter/vs-studio
- Remotion licence & pricing — https://www.remotion.dev/docs/license/pricing · https://www.remotion.dev/docs/license/faq · https://www.remotion.dev/docs/telemetry
- Remocn — https://github.com/kapishdima/remocn · https://dev.to/jqueryscript/remocn-64-remotion-animation-primitives-and-scenes-1fed
- RemotionUI — https://remotionui.com
- snapcn — https://github.com/snapcndev/snapcn
- shadcn registry directories — https://ui.shadcn.com/directory · https://registry.directory · https://21st.dev/community/shadcn-directory/motion · https://www.shadcn.io/awesome/registries · https://shadcnregistry.com
- React Flow UI — https://reactflow.dev/components · https://reactflow.dev/learn/tutorials/getting-started-with-react-flow-components · https://reactflow.dev/ui/templates/workflow-editor
- Vercel workflow builder — https://github.com/vercel-labs/workflow-builder-template
- Open Agent Builder — https://github.com/01-ai/open-agentkit-builder
- OpenScreen Studio — https://github.com/AlanRoybal/openscreen-studio
- Mediabunny — https://mediabunny.dev