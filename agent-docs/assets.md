# Assets and files

How specs name files, how a page gets their addresses, the render file server, prepared files, model import and the project files the app follows. Paths are relative to `crates/` unless they name another folder. Footage addresses are in [footage.md](footage.md), the project file itself in [document.md](document.md), fonts in [fonts.md](fonts.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                  | Where                                                                                                                                                          |
| ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Refs                     | `compile/src/asset_ref.rs` → `PreparedRef`; `compile/src/elements/prepared.rs` → `prepare_props`                                                               |
| Host addresses           | `compile/src/hosted.rs` → `hosted`                                                                                                                             |
| `app:` files on the page | `packages/video/src/assets.tsx` → `assetUrl`, `withAppFiles`; `publicFile` (`@opsis/runtime`)                                                                  |
| Render file server       | `engine/src/http_files.rs` → `folder_routes` (`ServeDir`); `prepare/src/serve.rs` → `prepared_routes`, `prepare`; put together in `render/src/executors.rs`    |
| Preparers                | `prepare` (`opsis-prepare`); typefaces checked against `fixtures/prepare/typefaces.json`                                                                       |
| Prepared version         | `engine/src/versions.rs` → `PREPARED_VERSION`                                                                                                                  |
| Asset and model import   | `project/src/import.rs` → `import_assets`; `project/src/model.rs` → `model_files`                                                                              |
| Followed files           | `compile/src/asset_ref.rs` → `project_refs_in`; `session/src/document.rs` → `follow_loaded_files`; `project/src/watch.rs` → `LoadedFilesWatch`, `watched_refs` |
| Atomic writes            | `engine/src/files.rs` → `atomic_write`, `partial_file`                                                                                                         |

## Rules

### Refs and hosting

- **Specs hold host-independent refs: `project:<relative path>`, `prepared:…` or `app:<public path>`, and only Rust reads them.**
- **Before a spec goes to a page, `hosted` gives every `project:` and `prepared:` ref, and every font face, its address on that host** from `assets.projectBase` (the render asset server).
  - It also rewrites the sounds' and the mix's sources, and reaches only `MAX_PROP_DEPTH` levels into an item's props, skipping nested timelines; `withAppFiles` has the same limits.
- **The page only turns `app:` into a public file address** (`assetUrl`, `withAppFiles`, through the runtime's `publicFile`).
- **The editor reads project files from disk itself, resolving each ref in Rust** (`project/src/paths.rs` → `ref_to_path`); `reveal_in_folder` (`session/src/settings.rs`) takes a `project:` ref or an absolute path and refuses an `app:` or external ref.

- **Exports stay readable like any file the user makes**: atomic writes (`atomic_write`) give the file the usual mode the umask allows (`partial_file`).

### Prepared files

- **Work the video would repeat in every render tab is done once in Rust and kept**:
  - 3D text typefaces: the face is picked in compile like the page picks it, then `wuff` and `skrifa` draw its outlines at the weight into three.js typeface JSON;
  - 3D models: `draco-gltf` decodes Draco and meshopt meshes into a plain GLB with every image packed in, so the page uses a plain `GLTFLoader`;
  - SVG logos, flattened by `usvg`;
  - textures: images whose long side is over `LONG_SIDE_CAP` (`prepare/src/texture.rs`) are made smaller; others, and SVG textures, are served as they are.
- **Compile turns those settings into `prepared:<kind>/[weight/]<project|app>/<path>` refs (`prepare_props`), and `hosted` turns them into `.opsis/prepared/<path>` on the spec's project base.**
- **The render file server prepares a file on its first request** (`prepared_routes` → `prepare`) into `.opsis/cache/prepared/<key>`, the key a `versioned_hash` of `PREPARED_VERSION`, the prepared path and the source's size and modified time (`file_identity`).
  - A cache write that fails only logs a warning; the prepared file is still served.
- **A preparer whose output changes needs `PREPARED_VERSION` to change once Opsis is released** (not before, [AGENTS.md](../AGENTS.md) §2); what it keys is in [runs.md](runs.md).

### Models

- **Importing a file from outside the project copies it in** (`import_assets`): a file already inside the project is used where it is, a model (`MODEL_EXTENSIONS`) goes to `assets/models/<name>/` with its side files, anything else to `assets/` under a free name.
  - A model whose side files lie outside its folder or on the internet is refused (`OutsideFiles`), and one whose side file is missing fails (`MissingFiles`).

### Followed project files

- **After each compile Rust follows the exact files the film loads**: every `project:` ref in the compiled values, the project's font files and the project sources of prepared files (`project_refs_in`, `follow_loaded_files`), never the app's own `.opsis` files (`watched_refs` drops them).
  - Following the same refs in the same folder again does nothing.
  - A model's side files are not followed and not in its prepared key (`file_identity` of the model file only), so editing one alone changes nothing until the model file changes.
- **When one changes, Rust compiles again and sends every app window each changed file's size and modified time (`ProjectFilesChanged`)**, naming each file by every ref the film uses for it (`LoadedFilesWatch`). No native view handles that event; the editor follows the change through the new compile.
- **Renders never carry versions; their cache keys stamp files themselves** ([runs.md](runs.md)).
