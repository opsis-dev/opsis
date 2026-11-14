# Assets and files

How specs name files, how the renderer reads them, prepared files, model import and the project files the app follows. Paths are relative to `crates/` unless they name another folder. Footage addresses are in [footage.md](footage.md), the project file itself in [document.md](document.md), fonts in [fonts.md](fonts.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                  | Where                                                                                                                                                          |
| ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Refs                     | `compile/src/asset_ref.rs` → `PreparedRef`; `compile/src/elements/prepared.rs` → `prepare_props`                                                               |
| Host addresses           | `compile/src/hosted.rs` → `hosted`                                                                                                                             |
| Files the renderer reads | `render/src/native.rs` → `NativeFiles` (`bytes`, `size`, `frame`, `resolve`), `NATIVE_BASE`; prepared files `prepare/src/serve.rs` → `prepare`, `PreparedDirs` |
| Preparers                | `prepare` (`opsis-prepare`); typefaces checked against `fixtures/prepare/typefaces.json`                                                                       |
| Prepared version         | `engine/src/versions.rs` → `PREPARED_VERSION`                                                                                                                  |
| Asset and model import   | `project/src/import.rs` → `import_assets`; `project/src/model.rs` → `model_files`                                                                              |
| Followed files           | `compile/src/asset_ref.rs` → `project_refs_in`; `session/src/document.rs` → `follow_loaded_files`; `project/src/watch.rs` → `LoadedFilesWatch`, `watched_refs` |
| Written files            | `engine/src/files.rs` → `atomic_write`, `write_via_temp`, `partial_file`                                                                                       |

## Rules

### Refs and hosting

- **Specs hold host-independent refs: `project:<relative path>`, `prepared:…` or `app:<public path>`, and only Rust reads them.**
- **Before a spec is drawn, `hosted` gives every `project:` and `prepared:` ref, and every font face, its address under `assets.projectBase`**, which every render sets to `NATIVE_BASE` (`native:/`).
  - It first fills the spec's `fonts` when compile left them out (`film_fonts` with the default project fonts) and takes `assets` out of the spec; without a `projectBase` it stops there.
  - It also rewrites the sounds' and the mix's sources and writes the footage addresses ([footage.md](footage.md)).
- **`NativeFiles` reads every address from disk with no server**: `app:` from `public/`, `native:/<path>` from the project folder, `native:/.opsis/prepared/…` through `prepare`, and `native:/.opsis/footage/<id>/…` from `opsis-footage` as decoded frames or terminal frames files ([film.md](film.md), Output).
- **The editor reads project files from disk itself, resolving each ref in Rust** (`project/src/paths.rs` → `ref_to_path`); `reveal_in_folder` (`session/src/settings.rs`) takes a `project:` ref or an absolute path and refuses an `app:` or external ref.

### Written files

- **Files the app writes stay readable like any file the user makes**: both ways of writing through a temporary file, `atomic_write` (prepared files, image results, media caches) and `write_via_temp` (exports, previews, saved pictures), make it with `partial_file`, which leaves the mode to the umask.

### Prepared files

- **Work every render worker would repeat is done once and kept**:
  - 3D text typefaces: the face is picked in compile, then `wuff` and `skrifa` draw its outlines at the weight into three.js typeface JSON, which compile reads through `Measure::typeface`;
  - 3D models: `draco-gltf` decodes Draco and meshopt meshes into a plain GLB with every image packed in, which gltfio loads in the renderer program (KTX2 images through its Basis transcoder, [film.md](film.md));
  - SVG logos, flattened by `usvg`;
  - textures: images whose long side is over `LONG_SIDE_CAP` (`prepare/src/texture.rs`) are made smaller; others, and SVG textures, are served as they are.
- **Compile turns those settings into `prepared:<kind>/[weight/]<project|app>/<path>` refs (`prepare_props`), and `hosted` turns them into `native:/.opsis/prepared/<path>`.**
  - `prepare_props` looks only at the 3D text, 3D model, 3D card and 3D logo elements; a source `PreparedRef::new` refuses stays as it is. It takes only a `project:` ref that stays in the project or an `app:` path without `..`, so an absolute path, an external address and a bare relative path are refused too.
- **A file is prepared the first time the renderer reads it** (`prepare`) into `.opsis/cache/prepared/<key>`, the key a `versioned_hash` of `PREPARED_VERSION`, the prepared path and the source's size and modified time (`file_identity`); `NativeFiles::resolve` reads every prepared ref before the first frame, so a file that can't be prepared fails the render naming it.
  - A cache write that fails only logs a warning; the prepared file is still used.
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

- **`hosted` reaches only `MAX_PROP_DEPTH` levels into an item's props and skips nested timelines.**
- **A model's side files are not followed and not in its prepared key** (`file_identity` of the model file only), so editing one alone changes nothing until the model file changes.
