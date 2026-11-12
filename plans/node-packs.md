# Node packs

How nodes and elements install and update apart from the app: reusable subgraphs and templates first, then packs as data, then pack code as WebAssembly components described in WIT and loaded by wasmtime, then third-party packs from a file. Pack elements draw through the scene contract of [native-renderer.md](native-renderer.md).

Status: plan only. Nothing here is built yet. Builds after sections 1–7 of [release.md](release.md); phases 3 and 4 also need the native renderer's **Scene** part. Facts were checked on 5 November 2026.

## How to read this plan

- **Phase 1** (reusable nodes and templates) and **Phase 2** (packs as data) need nothing from the native renderer.
- **Contract**, **Host**, **Elements from packs**, **Nodes from packs** and **Shaders** make up phase 3, pack code.
- **Permissions and signing**, **Installing** and **SDK and checks** make up phase 4, third-party packs.
- Each part has the same sections: **What it changes**, **Fit with Opsis** (In place / Missing), **Work**, **Decisions**, **Risks**, **Sources**.

## Conventions

These restate rules from `AGENTS.md` and `agent-docs/`; when they disagree with this plan, they win. When a part ships, its rules move into a new `agent-docs/packs.md`, with pointers from `nodes.md`, `elements.md`, `runs.md` and `packaging.md`.

- **A pack can only reach what the host links into it.** No pack code runs outside wasmtime; nothing in a pack reaches the network.
- **Drawing is deterministic.** An element's `draw` gets no clock, no random source, no files and no WASI; it gets only the `host` interface (text measuring, Opsis's own `spring` and `random`, logging).
- **Built-in nodes and elements keep their ids**, and a project that used them before packs opens unchanged.
- **No comments** in WIT, Rust or any new file.
- **User-facing copy follows AGENTS.md**: no runtime, format or library names in titles, descriptions, warnings or errors; "packs", "nodes", "elements" and "actions" in words.
- **Checks when a part is done:** `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, plus the part's own pack tests.

## Decided

- **Phases:** no-code nodes first, packs as data second, pack code third, third-party packs fourth. Third-party packs install from a file, with no online catalogue at first. Unsigned packs may run code after a clear warning naming what they can do; signed ones skip it.
- **Pack code is WebAssembly components described in WIT** (`opsis:pack`), for elements, compile steps and executors alike. A pack is one component.
- **wasmtime is the loader**: Cranelift compiles packs, and it is the reference runtime the component model and WASI are built on. Not Extism: its ABI is its own, on plain modules, and it doesn't use the component model or WIT (extism/extism#666, open since 2024-01-26). Not wasmi, and not an embedded JavaScript engine.
- **Packs may be written in any language that builds WIT components.** Rust (`wit-bindgen` on `wasm32-wasip2`) is the documented and tested one; others are allowed and unsupported.
- **Pack executors are WASM components only**, never separate programs. They get Opsis's `files` interface and WASI preview 2, scoped: `wasi:filesystem` preopened to the run's input files and output folder, `wasi:clocks`, `wasi:random`. Drawing never gets WASI.
- **Built-in elements and nodes are native Rust behind the same contract**: linked into the app, returning the same types, with no boundary cost. If a built-in needs something the contract lacks, the contract grows.
- **Packs may ship GLSL shaders**, which Filament's runtime material compiler builds into materials when the pack loads.
- **After release, every minor of the current pack API major keeps loading.** `opsis:pack@1.x` packs load until 2.0; a new major drops the older one.

## Summary

| Part                    | What it changes                                                                      | Needs                                                |
| ----------------------- | ------------------------------------------------------------------------------------ | ---------------------------------------------------- |
| Phase 1                 | reusable subgraph nodes, project templates, per-node presets                         | —                                                    |
| Phase 2                 | `pack.json`, first-party packs grouping today's nodes, packs listed per project      | Phase 1                                              |
| Contract                | the `opsis:pack` WIT: types, host, files, elements, nodes; versioning                | the scene types, shared with native renderer's Scene |
| Host                    | `opsis-packs`: wasmtime engine, compile cache, linking, limits, instances per worker | Contract                                             |
| Elements from packs     | pack elements in the catalogue, the resolve pass, `draw` per frame                   | Host                                                 |
| Nodes from packs        | pack settings schemas, `compile`, `execute` with files and scoped WASI               | Host                                                 |
| Shaders                 | pack GLSL compiled at load and run like built-in effects                             | Host, native renderer's Effects                      |
| Permissions and signing | manifest permissions, the warning, signatures                                        | Nodes from packs                                     |
| Installing              | install from a file, update, remove, the data folder and project packs               | Permissions and signing                              |
| SDK and checks          | the Rust SDK crate, a template pack, the pack checker                                | Contract                                             |

## Phase 1

### What it changes

- **"Save as reusable node" on a selection** (nodes, inner edges, exposed ports and settings) into the data folder or the project. A `core.subgraph` node compiles by expanding its contents in `opsis-compile`.
- **"Save project as template"** and per-node presets.

### Fit with Opsis

- **In place:** templates from `templates.json` (`opsis-project`), the shared selection (`crates/session/src/shared.rs`) and the document's gestures (`opsis-document`).
- **Missing:** the subgraph node, its expansion in compile, saved presets.

### Work

1. `core.subgraph` and its expansion, with property tests that a subgraph compiles to the same spec as its contents inline.
2. Save-as actions in the editor; presets in the store.

### Decisions

- **Where saved nodes live** when not in the project: the data folder's own `nodes/` or the app database.

### Risks

- **Port and setting renames inside a saved subgraph** break the projects that use it; saved subgraphs carry a version.

### Sources

- [release.md](release.md) §7.

## Phase 2

### What it changes

- **A `pack.json` manifest**: id, name, semver version, the pack API version, permissions, nodes and elements, and the component file for phases 3 and 4.
- **First-party packs group today's nodes without changing their ids.**
- **Projects list the packs they use**, and an unavailable node names its missing pack.
- **Pack id and version join the node cache key** (`opsis-runner`).

### Fit with Opsis

- **In place:** the catalogue and categories (`crates/compile/src/node.rs` → `Category::label`), node cache keys ([runs.md](../agent-docs/runs.md)).
- **Missing:** the manifest, the grouping, the project's pack list, the cache-key change.

### Work

1. The manifest type in Rust with its JSON Schema (`schemars`), exported like every boundary type.
2. First-party packs as manifests over today's nodes.
3. The project's pack list and the "isn't available" message.

### Decisions

- None.

### Risks

- **Cache keys change** when pack id and version join them; pre-release, caches are cleared rather than migrated (AGENTS.md: no old formats).

### Sources

- [release.md](release.md) §7.

## Contract

### What it changes

- **One WIT package, `opsis:pack`, is the contract for pack code.**
  - `types`: the scene as a flat list of nodes with child indices (WIT has no recursive types), each carrying the CSS and SVG subset of [native-renderer.md](native-renderer.md) **Scene**: the style record under CSS property names, text runs, inline SVG trees, images and footage frames, Lottie, 3D scenes, shader effects, slots for children and transitions; `Clock`, `Stage`, assets, spring settings.
  - `host`: `measure-text`, `spring`, `random`, `log`.
  - `files`: read inputs, write outputs, report progress.
  - `elements`: `list-elements`, `shaders`, `assets`, and the `element` resource with `open`, `length` and `draw(clock, stage, transition)`.
  - `nodes`: `list-nodes`, `compile`, `execute`.
  - Worlds: `element-pack` (imports `host`, exports `elements`) and `pack` (adds `files`, the WASI interfaces above and `nodes`).
- **Settings and compile input and output are JSON strings**, checked against the element's or node's JSON Schema. Rust owns `TimelineSpec` and its schema; the pack API version pins which one a pack compiles to.
- **Versioning**: the package carries the pack API version. Minors only add; a major may remove. The host keeps a linker for every minor of the current major.

### Fit with Opsis

- **In place:** a first draft, `opsis:pack@0.1.0`, written during the native-renderer research. It passes `wasmtime::component::bindgen!` for both worlds under wasmtime 50.0.1 (wit-parser 0.261.0) with no warnings; that check renamed the displacement filter's `map` field to `source`, since `map` is a WIT keyword. The draft is in [Appendix: draft WIT](#appendix-draft-wit).
- **Missing:** the WIT in the repository, the `wasi` imports in the `pack` world, a guest built against it, the fields a built-in needs that the draft can't carry yet (Work 4), though the Decided section puts built-ins behind the same contract, and scene records that carry the CSS and SVG subset (Work 5): the draft's `layout`, `box-shape`, `path-shape`, `text-node` and `node` hold a smaller vocabulary of their own.

### Work

1. The WIT in the repository with its version, checked by `bindgen!` in a test.
2. The `pack` world gains `wasi:filesystem`, `wasi:clocks` and `wasi:random` imports for executors.
3. A guest pack in Rust built against it in the tests, so both sides are checked.
4. Give the draft what today's built-ins declare, so each can be described by it:
   - `element-info`: the default length, `optional_clip`, fixed and kind props, the look, and whether the design size is exact;
   - `port`: the label, whether it is optional and its file need, and a port kind that is the `PortType` enum rather than a free string;
   - `node-info`: the phase, the input family, whether it is a browser action or hidden, and `uses_project_size` and `uses_project_fonts`;
   - `category`: the `Category` and `ElementCategory` enums rather than strings.
5. Write the scene records from `opsis-scene`'s style struct and SVG tree, and check both worlds with `bindgen!` again.

### Decisions

- **Where the scene types are written**: shared with [native-renderer.md](native-renderer.md) **Scene** (generated from the WIT, or hand-written with a test).
- **Which comes first, this Contract or native-renderer's Scene.** Each is written as waiting on the other: the Summary has Contract needing Scene, and Scene's node types are this plan's WIT `types`. Whichever starts first writes the scene types, and the other builds on them.
- **WIT file layout**: one file, or one per interface in a `wit/` folder.

### Risks

- **A field added to a record breaks older guests** under the component model; minors therefore add new records, functions or variant cases in new interfaces rather than changing existing ones.

### Sources

- `wasmtime` 50.0.1 `src/runtime/component/`, `wit-bindgen` 0.63.0, Zed's extension write-up (https://www.plushcap.com/content/zed/blog/life-of-a-zed-extension-rust-wit-wasm), the draft WIT.

## Host

### What it changes

- **A new crate, `opsis-packs`, loads and runs pack code.**
  - One wasmtime `Engine` with the component model, Cranelift, `epoch_interruption` to stop a call that runs too long, and a `ResourceLimiter` for memory.
  - Compiled components cached in the data folder (`Component::serialize`), keyed by the component's hash and the wasmtime version.
  - A `Linker` per pack API minor; draw-only packs get `host` alone; executors get `files` and scoped WASI.
  - One instance per pack per render worker, so parallel render chunks don't share pack state.
- **Floating point stays identical across machines**: `cranelift_nan_canonicalization` is on, so NaN bit patterns don't differ between CPUs.

### Fit with Opsis

- **In place:** `AppState.tasks` for long work ([AGENTS.md](../AGENTS.md) §7), the data folder layout ([app-state.md](../agent-docs/app-state.md)), child temp folders ([engine.md](../agent-docs/engine.md)).
- **Missing:** the crate, the cache, the limits, the per-worker instances.

### Work

1. `opsis-packs` with the engine, cache, linkers and limits.
2. Errors from a pack (trap, limit, invalid scene) become the node's error in words, naming the pack.
3. Tests with a guest that traps, loops, allocates past its limit and returns an invalid scene.

### Decisions

- **The time limit per `draw` and per `execute` call**, set by measurement.
- **The memory limit per instance.**

### Risks

- **wasmtime adds Cranelift to the app**; the size wasn't measured.
- **The cost of a `draw` call** (copying a scene across the boundary every frame) wasn't measured; measure with a heavy element before fixing the scene layout.
- **A wasmtime security fix means a new app release**, as with any bundled runtime.

### Sources

- wasmtime 50.0.1 (https://github.com/bytecodealliance/wasmtime): `crates/wasmtime/src/config.rs` (`consume_fuel`, `epoch_interruption`, `cranelift_nan_canonicalization`, `cache`, `memory_reservation`), `crates/wasmtime/src/runtime/limits.rs` (`ResourceLimiter`), `crates/wasmtime/src/runtime/component/component.rs` (`serialize`).

## Elements from packs

### What it changes

- **Pack elements join the catalogue** with their settings schema, defaults, fonts, design size and authored-clock flag (the 30 fps clock, `.authored_clock`) from `element-info`, beside built-in elements.
- **Before a render, `assets(key, settings)`** lists what the element needs; the native renderer's resolve pass loads it and `element.open` receives an `asset-info` for each.
- **Each frame, `draw` returns scene nodes** that take the same path as built-in elements: hashes, caches, layout, Skia, Filament.

### Fit with Opsis

- **In place:** by then, the native renderer's scene, resolve pass and caches.
- **Missing:** the catalogue entries from packs, the asset hand-over, the call per frame.

### Work

1. Pack elements in the catalogue and the Inspector, with the pack named on the card.
2. The asset hand-over and the `draw` call in the render worker.
3. A built-in element reimplemented as a test pack, compared frame by frame with the built-in one.

### Decisions

- None.

### Risks

- **A pack element that returns a huge scene every frame** slows every render it's in; the limits above and a scene-size cap apply.

### Sources

- [native-renderer.md](native-renderer.md) **Scene** and **Elements**.

## Nodes from packs

### What it changes

- **Pack nodes join the catalogue** with ports, settings schema and defaults from `node-info`.
- **`compile` runs inside `opsis-compile`'s compile** for pack nodes and returns spec fragments as JSON, checked against the schema before use.
- **`execute` runs pack executors** with `files` and scoped WASI, under `AppState.tasks`, with progress through `files.progress`.

### Fit with Opsis

- **In place:** executors and their contexts (`ExecutorContext`, output folders), the run's events and progress ([runs.md](../agent-docs/runs.md)).
- **Missing:** the pack node path through compile, validation and the run.

### Work

1. Pack nodes in compile, with their fragments validated.
2. Pack executors in the run, with WASI preopens limited to the run's inputs and output folder.

### Decisions

- **What `wasi:random` and `wasi:clocks` mean for caching**: an executor that reads them is still cached by its inputs, as built-in executors are today.

### Risks

- **An executor that writes outside its output folder** is stopped by the preopens; tested with a guest that tries.

### Sources

- `wasmtime-wasi` 50.0.1 (`WasiCtxBuilder`, preopened directories).

## Shaders

### What it changes

- **`shaders` returns the pack's GLSL sources**, each the fragment code of an unlit material; Filament's runtime material compiler (`filamat`) builds each into a material when the pack loads, and a pack whose shader fails doesn't load.
- **Pack effects run like built-in ones**: full-screen passes on Filament over input layers, on the GPU or the software driver.

### Fit with Opsis

- **In place:** by then, the native renderer's effects path.
- **Missing:** loading pack shaders, limits on them, and `filamat` linked into the app (`libfilamat.a` is 35,180,872 B in Filament v1.80.0's Linux archive; its size in the linked program is unmeasured).

### Work

1. Compiling at load, with the shader's error shown on the pack.
2. Limits: uniforms per effect, inputs per effect, texture size.

### Decisions

- **How a pack names a shader's parameters**: listed beside the source in `shader-source`, or read from the source. `MaterialBuilder` takes each one by name and type.

### Risks

- **A shader that runs for seconds per frame** on the software drivers; the render's time limit per frame applies.

### Sources

- Filament v1.80.0 (https://github.com/google/filament): `libs/filamat/include/filamat/MaterialBuilder.h` (`material(code)`, `shading`, `parameter`, `build`), `filament/include/filament/Material.h` (`Builder::package`); [native-renderer.md](native-renderer.md) **Effects and 3D**.

## Permissions and signing

### What it changes

- **The manifest lists permissions**: `files`, `wasi:filesystem`, `wasi:clocks`, `wasi:random`. The host links only what is granted.
- **Unsigned packs show a warning naming each permission** before their code first runs; signed packs skip it.

### Fit with Opsis

- **In place:** update signing ([release.md](release.md) §5); code signing is still to come (§4).
- **Missing:** the permission list, the warning, signature checks.

### Work

1. Permissions in the manifest and the linker.
2. The warning in words, per pack, remembered once accepted.
3. Signature checks on install.

### Decisions

- **Who signs**: one Opsis key, or publishers' keys trusted by the user.
- **The signature format.**

### Risks

- **A signed pack's key leaking** means revocation; decide how before signing starts.

### Sources

- [release.md](release.md) §4, §5, §7.

## Installing

### What it changes

- **Packs install from a file** into the data folder; updates replace a pack by id with a higher version; removing a pack leaves projects that use it showing the missing-pack message.
- **A project can carry its own packs**, so it opens on another machine.

### Fit with Opsis

- **In place:** the data folder, project folders and asset refs ([assets.md](../agent-docs/assets.md)).
- **Missing:** the pack file format, install, update and remove, project packs.

### Work

1. The pack file: a zip of `pack.json`, the component and the pack's own files.
2. Install, update and remove in Settings, and project packs.

### Decisions

- **Whether a project carries its packs** by copying them into the project folder, or only lists them.

### Risks

- **Two projects needing different versions of one pack** on one machine; the data folder keeps versions side by side.

### Sources

- [release.md](release.md) §7.

## SDK and checks

### What it changes

- **`opsis-pack-sdk`, a Rust crate for pack authors**, wraps `wit-bindgen` with the builder API built-in elements use, so a pack element reads like a built-in one.
- **A template pack** with one element and one node.
- **A pack checker** that validates the manifest, the WIT version and every shader, then draws each element at chosen frames into stills.

### Fit with Opsis

- **In place:** `render-elements`; by then, the builder API from the native renderer.
- **Missing:** the SDK crate, the template, the checker.

### Work

1. The SDK and template, built for `wasm32-wasip2` in CI.
2. The checker as a Rust binary in the workspace.
3. Docs for pack authors: Rust first; other component languages named as allowed and unsupported.

### Decisions

- **Whether `wasm32-wasip2` joins `rust-toolchain.toml`'s targets** for the whole workspace or only CI.

### Risks

- **`cargo-component` 0.21.1 dates from 2025-03-18**; the SDK uses `wit-bindgen` (0.63.0) and the `wasm32-wasip2` target directly.

### Sources

- `wit-bindgen` 0.63.0, `cargo-component` 0.21.1 (crates.io), ComponentizeJS 0.23.0 (2026-09-21, experimental, SpiderMonkey per component) for JavaScript authors.

## Lua as a pack language

Findings only, checked on 11 November 2026 when Lua scripting was built ([scripting.md](../agent-docs/scripting.md)). Pack code stays WebAssembly components loaded by wasmtime, and nothing here changes that.

- **Native Lua per node is the wrong home for drawing and executing.** The Lua probe links the computer's `libm`, so `math.sin` can differ between machines, which the packs plan guards against inside wasmtime; and native Lua has no sandbox of the kind an executor needs.
- **Fusion (DaVinci Resolve) does run nodes as Lua**: a Fuse is a Lua script that defines a node, runs on LuaJIT and reloads as it is edited, and its documentation says it may be slower than a compiled plugin. This comes from search summaries of Blackmagic's documents; the Fuse SDK pages returned 404.
- **Lua can instead be one more language for writing packs**: a stock Lua interpreter built as an `opsis:pack` component, with the author's `.lua` put into it by the pack checker. A pack stays one component, and every limit of the packs plan holds.
- **wasmtime 50.0.1 turns the exception-handling proposal on by default** under Cranelift (`Config::wasm_exceptions`, `crates/wasmtime/src/config.rs`); the option for the older encoding is hidden and deprecated. Lua raises errors with `setjmp` and `longjmp` (`LUAI_THROW`, `ldo.c`), which on WASI compile to exceptions, so the toolchain must emit the standard encoding.
- **The Bytecode Alliance's article on exceptions (6 November 2025)** reports a Lua interpreter using `setjmp` and `longjmp` working on wasmtime.
- **An older port, `lua-wasi`, stubs `setjmp` and `longjmp`**, so any Lua error there aborts the process; that is what a build without exception handling looks like.
- **`mlua` supports WebAssembly only on `wasm32-unknown-emscripten`** (its README), so the interpreter component would be C Lua with bindings generated for C, or `piccolo`, which is experimental.
- **None of this was built**: this machine has no WASI sysroot and no wasmtime program. The cost of an interpreter inside WebAssembly on every `draw` is unmeasured.
- **Zed and Zellij both chose WebAssembly for extensions**; Zed compiles Rust to `wasm32-wasip2`.

### Sources

- `wasmtime` 50.0.1 `src/config.rs`; `lua-src` 551.0.2 `lua-5.5.1/ldo.c`; `mlua` 0.12.2 `README.md`.
- Exceptions in Cranelift and Wasmtime (https://bytecodealliance.org/articles/wasmtime-exceptions); lua-wasi (https://github.com/singlestore-labs/lua-wasi/blob/main/README.md).
- The Resolve scripting API (https://wiki.dvresolve.com/developer-docs/scripting-api), the Fusion Fuse SDK (https://documents.blackmagicdesign.com/UserManuals/Fusion_Fuse_SDK.pdf).
- Zed's extensions (https://zed.dev/docs/extensions/developing-extensions), Zellij's plugins (https://zellij.dev/documentation/plugins.html).

## Appendix: draft WIT

`opsis:pack@0.1.0`, as checked with `wasmtime::component::bindgen!` (wasmtime 50.0.1) for both worlds on 5 November 2026; `shader-source`'s `glsl` field is not covered by that check. **Contract**'s work adds the `wasi` imports to the `pack` world, writes the scene records from the CSS and SVG subset and moves the file into the repository.

```wit
package opsis:pack@0.1.0;

interface types {
  record size {
    width: u32,
    height: u32,
  }

  record point {
    x: f32,
    y: f32,
  }

  record rect-geom {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
  }

  record rgba {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
  }

  record oklch-colour {
    l: f32,
    c: f32,
    h: f32,
    alpha: f32,
  }

  variant colour {
    srgb(rgba),
    display-p3(rgba),
    oklch(oklch-colour),
  }

  enum interpolation {
    srgb,
    linear-srgb,
    oklab,
    oklch,
    oklch-longer-hue,
  }

  enum spread {
    pad,
    repeat,
    reflect,
  }

  record stop {
    offset: f32,
    colour: colour,
  }

  record linear-gradient {
    start: point,
    end: point,
    stops: list<stop>,
    space: interpolation,
    spread: spread,
  }

  record radial-gradient {
    center: point,
    radius: point,
    stops: list<stop>,
    space: interpolation,
    spread: spread,
  }

  record conic-gradient {
    center: point,
    angle: f32,
    stops: list<stop>,
    space: interpolation,
  }

  enum fit {
    fill,
    contain,
    cover,
    none,
  }

  enum sampling {
    nearest,
    linear,
    cubic,
  }

  record image-paint {
    asset: u32,
    fit: fit,
    sampling: sampling,
  }

  variant paint {
    solid(colour),
    linear(linear-gradient),
    radial(radial-gradient),
    conic(conic-gradient),
    image(image-paint),
  }

  enum blend-mode {
    normal,
    multiply,
    screen,
    overlay,
    darken,
    lighten,
    color-dodge,
    color-burn,
    hard-light,
    soft-light,
    difference,
    exclusion,
    hue,
    saturation,
    color,
    luminosity,
    plus-lighter,
  }

  record shadow {
    offset: point,
    blur: f32,
    spread: f32,
    colour: colour,
  }

  record turbulence {
    base-frequency: point,
    octaves: u8,
    seed: u32,
    fractal: bool,
  }

  record displacement {
    source: u32,
    scale: f32,
  }

  variant filter {
    blur(f32),
    drop-shadow(shadow),
    brightness(f32),
    contrast(f32),
    saturate(f32),
    hue-rotate(f32),
    grayscale(f32),
    sepia(f32),
    invert(f32),
    opacity(f32),
    colour-matrix(list<f32>),
    turbulence(turbulence),
    displacement(displacement),
  }

  record affine {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
  }

  variant transform {
    affine(affine),
    matrix(list<f32>),
  }

  record rounded-rect {
    rect: rect-geom,
    radii: list<f32>,
  }

  variant clip {
    rect(rect-geom),
    rounded(rounded-rect),
    polygon(list<point>),
    path(string),
  }

  enum mask-mode {
    alpha,
    luminance,
  }

  record mask {
    source: u32,
    mode: mask-mode,
  }

  variant dimension {
    auto,
    px(f32),
    percent(f32),
  }

  variant track {
    auto,
    px(f32),
    percent(f32),
    fr(f32),
  }

  record edges {
    top: dimension,
    right: dimension,
    bottom: dimension,
    left: dimension,
  }

  enum display {
    flex,
    grid,
    none,
  }

  enum axis {
    row,
    column,
  }

  enum align {
    start,
    center,
    end,
    stretch,
    space-between,
    space-around,
  }

  enum position {
    relative,
    absolute,
  }

  record layout {
    display: display,
    axis: axis,
    wrap: bool,
    justify: align,
    align: align,
    gap: f32,
    padding: edges,
    margin: edges,
    width: dimension,
    height: dimension,
    position: position,
    inset: edges,
    grow: f32,
    shrink: f32,
    basis: dimension,
    columns: list<track>,
    rows: list<track>,
  }

  enum cap {
    butt,
    round,
    square,
  }

  enum join {
    miter,
    round,
    bevel,
  }

  record stroke {
    paint: paint,
    width: f32,
    cap: cap,
    join: join,
    dash: list<f32>,
    dash-offset: f32,
  }

  enum fill-rule {
    nonzero,
    evenodd,
  }

  record box-shape {
    fill: list<paint>,
    radii: list<f32>,
    border: option<stroke>,
    shadows: list<shadow>,
    inset-shadows: list<shadow>,
  }

  record path-shape {
    data: string,
    view-box: option<rect-geom>,
    fill: option<paint>,
    fill-rule: fill-rule,
    stroke: option<stroke>,
    trim-start: f32,
    trim-end: f32,
  }

  record feature {
    tag: string,
    value: u32,
  }

  record variation {
    tag: string,
    value: f32,
  }

  record text-run {
    text: string,
    family: string,
    size: f32,
    weight: u16,
    italic: bool,
    tracking: f32,
    leading: f32,
    paint: paint,
    features: list<feature>,
    variations: list<variation>,
  }

  enum text-align {
    start,
    center,
    end,
    justify,
  }

  record text-node {
    runs: list<text-run>,
    align: text-align,
    wrap: bool,
    max-lines: option<u32>,
  }

  record text-metrics {
    width: f32,
    height: f32,
    baseline: f32,
    lines: u32,
  }

  record image-node {
    asset: u32,
    fit: fit,
    sampling: sampling,
    time: option<f64>,
  }

  record lottie-node {
    asset: u32,
    frame: f64,
  }

  record model-node {
    asset: u32,
    camera: list<f32>,
    pose: list<f32>,
    animation: option<string>,
    time: f64,
    environment: f32,
  }

  record effect {
    shader: u32,
    inputs: list<u32>,
    uniforms: list<f32>,
    padding: f32,
  }

  enum slot {
    children,
    outgoing,
    incoming,
  }

  variant content {
    group,
    box(box-shape),
    path(path-shape),
    text(text-node),
    image(image-node),
    svg(u32),
    lottie(lottie-node),
    model(model-node),
    effect(effect),
    slot(slot),
  }

  record node {
    content: content,
    children: list<u32>,
    layout: layout,
    transform: option<transform>,
    origin: point,
    opacity: f32,
    blend: blend-mode,
    filters: list<filter>,
    backdrop: list<filter>,
    clip: option<clip>,
    mask: option<mask>,
    isolate: bool,
  }

  record scene {
    nodes: list<node>,
    root: u32,
  }

  record clock {
    frame: f64,
    fps: u32,
    length: u32,
  }

  record stage {
    width: u32,
    height: u32,
    design: option<size>,
    overlay-scale: f32,
    insets: list<f32>,
  }

  record transition-state {
    progress: f32,
    entering: bool,
  }

  enum asset-kind {
    image,
    svg,
    footage,
    model,
    lottie,
    font,
  }

  record asset-request {
    address: string,
    kind: asset-kind,
  }

  record asset-info {
    address: string,
    kind: asset-kind,
    width: u32,
    height: u32,
    length-frames: option<u32>,
  }

  record spring-config {
    mass: f64,
    stiffness: f64,
    damping: f64,
    overshoot-clamping: bool,
  }

  enum level {
    info,
    warn,
    error,
  }
}

interface host {
  use types.{text-run, text-metrics, spring-config, level};

  measure-text: func(runs: list<text-run>, max-width: option<f32>) -> text-metrics;
  spring: func(config: spring-config, frame: f64, fps: u32) -> f64;
  random: func(seed: string) -> f64;
  log: func(level: level, message: string);
}

interface files {
  read: func(address: string) -> result<list<u8>, string>;
  write: func(name: string, bytes: list<u8>) -> result<string, string>;
  progress: func(fraction: f32, label: option<string>);
}

interface elements {
  use types.{size, scene, clock, stage, transition-state, asset-request, asset-info};

  enum element-kind {
    component,
    wrapper,
    presentation,
    sequencer,
  }

  record element-info {
    key: string,
    title: string,
    description: string,
    category: string,
    kind: element-kind,
    settings-schema: string,
    defaults: string,
    fonts: list<string>,
    design: option<size>,
    authored-clock: bool,
  }

  record shader-source {
    name: string,
    glsl: string,
  }

  list-elements: func() -> list<element-info>;
  shaders: func() -> list<shader-source>;
  assets: func(key: string, settings: string) -> result<list<asset-request>, string>;

  resource element {
    open: static func(key: string, settings: string, assets: list<asset-info>) -> result<element, string>;
    length: func() -> option<u32>;
    draw: func(clock: clock, stage: stage, transition: option<transition-state>) -> result<scene, string>;
  }
}

interface nodes {
  record port {
    name: string,
    kind: string,
    many: bool,
  }

  record node-info {
    type-name: string,
    title: string,
    description: string,
    category: string,
    settings-schema: string,
    defaults: string,
    inputs: list<port>,
    outputs: list<port>,
    runs: bool,
  }

  list-nodes: func() -> list<node-info>;
  compile: func(type-name: string, settings: string, inputs: string) -> result<string, string>;
  execute: func(type-name: string, settings: string, inputs: string) -> result<string, string>;
}

world element-pack {
  import host;
  export elements;
}

world pack {
  include element-pack;
  import files;
  export nodes;
}
```
