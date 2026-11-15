# Contributing to Opsis

Thanks for helping. Opsis is an early preview, so the code moves quickly; open an issue before starting anything large, so we can agree on the approach first.

## Set up

You need [Bun](https://bun.sh), Node.js 22.12 or later for the TypeScript tests, [rustup](https://rustup.rs) (it installs the pinned nightly by itself), [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) for `bun run lint:fix` (`cargo install cargo-deny`), [uv](https://docs.astral.sh/uv/), Docker, clang with libc++ and libc++abi, CMake, make, pkg-config and git, and on Linux the system libraries the static Qt links; `bun run setup` checks for them and prints the `apt install` line for any that are missing.

```sh
bun install
bun run setup
bun start
```

`bun run setup` also installs the recording browser into `~/.config/Opsis/engine`, where the tests look for it (`--no-engine` skips it). Editing works without it; recordings need it, and so do the tests that draw with its software graphics driver.

## Before you send a change

Run these from the repository root and fix what they report:

```sh
bun run typecheck
bun run lint:fix
bun run format:fix
bun run test
```

They cover the TypeScript, Rust, Zig and C++ code (`cargo fmt`, `cargo clippy -- -D warnings`, `cargo deny`, `clang-tidy` and `cargo test` run inside them). If your change touches the app's windows, run the end-to-end scenarios too. They need `xvfb` (and `sway` for the `wayland` scenario), run in a virtual display only, and never open a window on your screen; every scenario but `wayland` needs the dev engine (below), so without it run `bun run e2e:qt wayland`:

```sh
bun run e2e:qt
```

The `tour`, `runs` and `api` scenarios, and the Rust tests that are ignored by default, need the dev engine: the recording browser `bun run setup` installs in `~/.config/Opsis/engine`. Run those tests with `bun run test:rust -p <crate> -- --ignored`; the golden and 3D picture tests also draw with its software graphics driver and the renderer program; each test's `#[ignore]` reason says what it needs. One of them, `crates/probe/tests/ffprobe.rs`, compares Opsis's media reader with ffprobe, which the app doesn't use; `bun run setup` fetches the pinned ffprobe beside FFmpeg.

## Rules the code follows

[AGENTS.md](./AGENTS.md) is the full guide, and [`agent-docs/`](./agent-docs) explains each part of the code; they apply to people as much as to coding agents. The ones that catch most first changes:

- **No comments in code.** If something needs explaining, rename it or split it up. Rules that span the codebase go in AGENTS.md, rules for one part in its doc in `agent-docs/`.
- **TypeScript file names are kebab-case**; Rust module files are snake_case.
- **Never hand-edit generated or installed code.** AGENTS.md lists every generated file and the command that writes it (for example, `bun run types:export` after changing a Rust type that crosses into TypeScript).
- **Rust is the source of types and logic.** Shapes that cross a boundary are Rust types, shared through the `cxx` bridge with the editor and exported with specta for the scripts that need them; don't write them by hand, and don't copy logic into the editor's C++ that Rust already has.
- **Dependencies go through the package managers** (`bun add`, `cargo add`), at their latest release unless AGENTS.md pins them. A Rust dependency's version lives once in the root `Cargo.toml`'s `[workspace.dependencies]`; AGENTS.md says how to add one.
- **Scripts run with Bun:** `bun scripts/<name>.ts`.
- **User-facing text** is in sentence case, names no library or tool a person never works with, and gives durations in seconds.
- **Golden files** in `fixtures/` are recorded again with `OPSIS_WRITE_GOLDEN=1` when a change is intended; review the diff instead of editing them.
- **A change that makes a doc wrong updates that doc** in the same change.

## Commits and pull requests

- Keep each pull request to one change, with a short summary of what changed and why.
- Commit messages are one line describing the change.
- Say which checks you ran; the pull request template has the list.

## Licence

Opsis is licensed under the [GNU General Public License version 3 or later](./LICENSE), and contributions are accepted under the same terms.
