# Assets and files

How specs name files, how a page gets their addresses, the two file servers, prepared files, model import and the project files the app follows. Paths are relative to `crates/` unless they name another folder. Footage addresses are in [footage.md](footage.md), the project file itself in [document.md](document.md), fonts in [fonts.md](fonts.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                  | Where                                                                                                           |
| ------------------------ | --------------------------------------------------------------------------------------------------------------- |
| Refs                     | `compile/src/asset_ref.rs` → `PreparedRef`; `prepare_props`                                                     |
| Host addresses           | `compile/src/hosted.rs` → `hosted`                                                                              |
| `app:` files on the page | `packages/video/src/assets.tsx` → `assetUrl`, `withAppFiles`; `publicFile` (`@opsis/runtime`)                   |
| Render file server       | `engine` (`ServeDir`); `prepared_routes`, `folder_routes`                                                       |
| Editor file server       | `desktop/src/file_server.rs`; `apps/frontend/src/editor/file-url.ts` → `fileUrl`, `useFileUrl`                  |
| Preparers                | `prepare` (`opsis-prepare`); typefaces checked against `fixtures/prepare/typefaces.json`                        |
| Prepared version         | `engine/src/versions.rs` → `PREPARED_VERSION`                                                                   |
| Model import             | `project/src/model.rs`                                                                                          |
| Followed files           | `project_refs_in`, `follow_loaded_files`; `project/src/watch.rs` → `LoadedFilesWatch`; editor `useProjectFiles` |
| Atomic writes            | `engine/src/files.rs` → `partial_file`                                                                          |

## Rules

### Refs and hosting

- **Specs hold host-independent refs: `project:<relative path>`, `prepared:…` or `app:<public path>`, and only Rust reads them.**
- **Before a spec goes to a page, `hosted` gives every `project:` and `prepared:` ref, and every font face, its address on that host** from `assets.projectBase` (the render asset server).
- **The page only turns `app:` into a public file address** (`assetUrl`, `withAppFiles`, through the runtime's `publicFile`).
- **The editor loads every file through the desktop file server, which takes the ref itself**; `fileUrl` only encodes it. `reveal_in_folder` takes a ref too.
- **The editor's file server is plain HTTP on 127.0.0.1 under a token made at each launch, with ranges, never a custom URL scheme**, because WebKitGTK can't play video or audio from one (tauri-apps/tauri#3725).
- **Exports stay readable like any file the user makes**: atomic writes keep the umask's mode (`partial_file`).

### Prepared files

- **Work the video would repeat in every render tab is done once in Rust and kept**:
  - 3D text typefaces: the face is picked in compile like the page picks it, then `wuff` and `skrifa` draw its outlines at the weight into three.js typeface JSON;
  - 3D models: `draco-gltf` decodes Draco and meshopt meshes into a plain GLB with every image packed in, so the page uses a plain `GLTFLoader`;
  - SVG logos, flattened by `usvg`;
  - textures: images over 4096 px on the long side are made smaller; others are served as they are.
- **Compile turns those settings into `prepared:<kind>/[weight/]<project|app>/<path>` refs (`prepare_props`), and `hosted` turns them into `.opsis/prepared/<path>` on the spec's project base.**
- **The render file server prepares a file on its first request** (`prepared_routes`) into `.opsis/cache/prepared/<sha256 of the path, the source's size and time and PREPARED_VERSION>`.
- **A preparer whose output changes needs `PREPARED_VERSION` to change**, because it keys the prepared files, the saved pictures and every render-phase node (see [AGENTS.md](../AGENTS.md) §2 on bumping before release).

### Models

- **Model import copies a model's side files into `assets/models/<name>/`** and refuses paths outside its folder or on the internet.

### Followed project files

- **After each compile Rust follows the exact files the film loads**: every `project:` ref in the compiled values, the project's font files and the project sources of prepared files (`project_refs_in`, `follow_loaded_files`), never the app's own `.opsis` files.
- **When one changes, Rust compiles again and sends every app window each changed file's size and modified time (`ProjectFilesChanged`)**, naming each file by every ref the film uses for it (`LoadedFilesWatch`).
- **Windows keep them as file versions keyed by ref (`useProjectFiles`)**; `useFileUrl` adds `?v=` to that file's address, so waveforms and filmstrips load it again.
- **Renders never carry versions; their cache keys stamp files themselves** ([runs.md](runs.md)).
