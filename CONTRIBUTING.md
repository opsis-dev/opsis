# Contributing to Opsis

Thanks for helping. Opsis is an early preview, so the code moves quickly; open an issue before starting anything large, so we can agree on the approach first.

## Set up

You need [Bun](https://bun.sh), a Rust toolchain, and on Linux the WebKitGTK and GTK development packages (on Ubuntu 22.04 or later: `libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev`, plus `gstreamer1.0-libav` to play videos in the editor).

```sh
bun install
bun run video:bundle
bun start
```

Then open **Settings → Engine** and download the engine (the recording browser and FFmpeg), or choose programs you already have. Editing works without it; recordings and exports need it.

## Before you send a change

Run these from the repository root and fix what they report:

```sh
bun run format:fix
bun run lint:fix
bun run typecheck
bun run test
```

They cover both the TypeScript and the Rust code (`cargo fmt`, `cargo clippy -- -D warnings` and `cargo test` run inside them). If your change touches the app's windows, run the end-to-end tests too. They need `xvfb`, run in a virtual display only, and never open a window on your screen:

```sh
(cd e2e && bun install)
bun run test:e2e
```

## Rules the code follows

[AGENTS.md](./AGENTS.md) is the full guide; it applies to people as much as to coding agents. The ones that catch most first changes:

- **No comments in code.** If something needs explaining, rename it or split it up. Rules that span the codebase go in AGENTS.md.
- **TypeScript file names are kebab-case**; Rust module files are snake_case.
- **Never hand-edit generated or installed code.** AGENTS.md lists every generated file and the command that writes it (for example, `bun run types:export` after changing a Rust type that crosses into TypeScript).
- **Rust is the source of types and logic.** Shapes that cross a boundary are Rust types exported with specta; don't write them by hand in TypeScript, and don't copy logic into the editor that Rust already has.
- **Dependencies go through the package managers** (`bun add`, `cargo add`), at their latest release unless AGENTS.md pins them.
- **Scripts run with Bun:** `bun scripts/<name>.ts`.
- **User-facing text** is in sentence case, names no library or tool, and gives durations in seconds.
- **Golden files** in `fixtures/` are recorded again with `OPSIS_WRITE_GOLDEN=1` when a change is intended; review the diff instead of editing them.

## Commits and pull requests

- Keep each pull request to one change, with a short summary of what changed and why.
- Commit messages are one line describing the change.
- Say which checks you ran; the pull request template has the list.

## Licence

Opsis is licensed under the [GNU General Public License version 3 or later](./LICENSE), with an additional permission to combine it with Remotion, and contributions are accepted under the same terms, that permission included. Opsis renders with Remotion, which has its own licence; see the README.
