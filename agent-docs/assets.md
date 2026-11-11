# Assets and files

How specs name files, how a page gets their addresses, the file servers, prepared files, model import and the project files the app follows. Paths are relative to `crates/` unless they name another folder. Footage addresses are in [footage.md](footage.md), the project file itself in [document.md](document.md), fonts in [fonts.md](fonts.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                  | Where                                                                                                                                                                                                                                                                                                               |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Refs                     | `compile/src/asset_ref.rs` → `PreparedRef`; `compile/src/elements/prepared.rs` → `prepare_props`                                                                                                                                                                                                                    |
| Host addresses           | `compile/src/hosted.rs` → `hosted`                                                                                                                                                                                                                                                                                  |
| `app:` files on the page | `packages/video/src/assets.tsx` → `assetUrl`, `withAppFiles`; `publicFile` (`@opsis/runtime`)                                                                                                                                                                                                                       |
| File servers             | `engine/src/http_files.rs` → `FileServer::start_with`, `file_routes`, `folder_routes` (`ServeDir`); the render's: `prepare/src/serve.rs` → `prepared_routes`, `prepare`, put together in `render/src/executors.rs` → `asset_server`; the terminal page's: `capture/src/terminal_paint.rs` → `TerminalPainter::open` |
| Preparers                | `prepare` (`opsis-prepare`); typefaces checked against `fixtures/prepare/typefaces.json`                                                                                                                                                                                                                            |
| Prepared version         | `engine/src/versions.rs` → `PREPARED_VERSION`                                                                                                                                                                                                                                                                       |
| Asset and model import   | `project/src/import.rs` → `import_assets`; `project/src/model.rs` → `model_files`                                                                                                                                                                                                                                   |
| Followed files           | `compile/src/asset_ref.rs` → `project_refs_in`; `session/src/document.rs` → `follow_loaded_files`; `project/src/watch.rs` → `LoadedFilesWatch`, `watched_refs`                                                                                                                                                      |
| Written files            | `engine/src/files.rs` → `atomic_write`, `write_via_temp`, `partial_file`                                                                                                                                                                                                                                            |

## Rules

### Refs and hosting

- **Specs hold host-independent refs: `project:<relative path>`, `prepared:…` or `app:<public path>`, and only Rust reads them.**
- **Before a spec goes to a page, `hosted` gives every `project:` and `prepared:` ref, and every font face, its address on that host** from `assets.projectBase` (the render asset server).
  - It first fills the spec's `fonts` when compile left them out (`film_fonts` with the default project fonts) and takes `assets` out of the spec, so the page never sees them; without a `projectBase` it stops there.
  - It also rewrites the sounds' and the mix's sources and writes the footage addresses ([footage.md](footage.md)).
- **Every page Opsis opens in the recording browser is served by a `FileServer`** (`engine/src/http_files.rs`): one folder at its root and extra routes beside it, on 127.0.0.1 with a free port and open CORS, until it is dropped.
  - A render's (`asset_server`) has the project folder at its root, prepared files, footage and the film page under `.opsis/film`; `bun run elements:render` builds one like it ([elements.md](elements.md)).
  - The terminal page's (`TerminalPainter::open`) has the app's public files at its root and the terminal bundle under `.opsis/terminal` ([terminal.md](terminal.md)).
- **The page only turns `app:` into a public file address** (`assetUrl`, `withAppFiles`, through the runtime's `publicFile`).
- **The editor reads project files from disk itself, resolving each ref in Rust** (`project/src/paths.rs` → `ref_to_path`); `reveal_in_folder` (`session/src/settings.rs`) takes a `project:` ref or an absolute path and refuses an `app:` or external ref.

### Written files

- **Files the app writes stay readable like any file the user makes**: both ways of writing through a temporary file, `atomic_write` (prepared files, image results, media caches) and `write_via_temp` (exports, previews, saved pictures), make it with `partial_file`, which leaves the mode to the umask.

### Prepared files

- **Work the video would repeat in every render tab is done once in Rust and kept**:
  - 3D text typefaces: the face is picked in compile like the page picks it, then `wuff` and `skrifa` draw its outlines at the weight into three.js typeface JSON;
  - 3D models: `draco-gltf` decodes Draco and meshopt meshes into a plain GLB with every image packed in, so the page needs only a `GLTFLoader`; KTX2 images stay KTX2 (`image/ktx2`), and the page's `KTX2Loader` (`packages/video/src/three/load-model.ts`) turns them into plain RGBA, never a GPU-compressed format (`PLAIN_PIXELS`), so a model looks the same on every computer; its transcoder ships in the film bundle (`basis_transcoder.js`, `.wasm`), found by `new URL(…, import.meta.url)`;
  - SVG logos, flattened by `usvg`;
  - textures: images whose long side is over `LONG_SIDE_CAP` (`prepare/src/texture.rs`) are made smaller; others, and SVG textures, are served as they are.
- **Compile turns those settings into `prepared:<kind>/[weight/]<project|app>/<path>` refs (`prepare_props`), and `hosted` turns them into `.opsis/prepared/<path>` on the spec's project base.**
  - `prepare_props` looks only at the 3D text, 3D model, 3D card and 3D logo elements; a source `PreparedRef::new` refuses stays as it is. It takes only a `project:` ref that stays in the project or an `app:` path without `..`, so an absolute path, an external address and a bare relative path are refused too.
- **The render file server prepares a file on its first request** (`prepared_routes` → `prepare`) into `.opsis/cache/prepared/<key>`, the key a `versioned_hash` of `PREPARED_VERSION`, the prepared path and the source's size and modified time (`file_identity`).
  - A cache write that fails only logs a warning; the prepared file is still served.
  - A file that can't be read or prepared answers 422, a path that isn't a prepared path 404, and a preparing task that fails to finish (`spawn_blocking`) 500.
- **A preparer whose output changes needs `PREPARED_VERSION` to change once Opsis is released** (not before, [AGENTS.md](../AGENTS.md) §2); what it keys is in [runs.md](runs.md).

### Models

- **Importing a file from outside the project copies it in** (`import_assets`): a file already inside the project is used where it is, a model (`MODEL_EXTENSIONS`) goes to `assets/models/<name>/` with its side files, anything else to `assets/` under a free name.
  - A model whose side files lie outside its folder or on the internet is refused (`OutsideFiles`), and one whose side file is missing fails (`MissingFiles`).

### Followed project files

- **After each compile Rust follows the exact files the film loads**: every `project:` ref in the compiled values, the project's font files and the project sources of prepared files (`project_refs_in`, `follow_loaded_files`), never the app's own `.opsis` files (`watched_refs` drops them).
  - Following the same refs in the same folder again does nothing.
- **When one changes, Rust compiles again and sends every app window each changed file's size and modified time (`ProjectFilesChanged`, declared in `session/src/events.rs`)**, naming each file by every ref the film uses for it (`LoadedFilesWatch`). No native view handles that event; the editor follows the change through the new compile.
- **A render's cache key stamps each project file its settings name by size and modified time (`file_stamps`)**, so the followed-file events never feed a key ([runs.md](runs.md)).

## Deliberate limits

- **`hosted` and `withAppFiles` reach only `MAX_PROP_DEPTH` levels into an item's props and skip nested timelines.**
- **A model's side files are not followed and not in its prepared key** (`file_identity` of the model file only), so editing one alone changes nothing until the model file changes.
