# Python scripts and just

How the repository's scripts move from TypeScript under bun to Python under `uv`, with `just` as the one command runner, so `package.json`, `bun.lock` and every npm dependency go away.

Status: plan only. Nothing here is built yet.

## Why

- **The TypeScript left is only `scripts/`** (and the `check-app` skill's `seed.ts`): 84 files, 8,412 lines, 12 of them Vitest tests. The film page, `packages/video` and `packages/runtime` are gone with the native renderer, so nothing in the app needs a JavaScript toolchain.
- **bun, TypeScript, oxlint, oxfmt, knip and Vitest are kept only for those scripts**, and `package.json` holds the command list as well as the dependencies.
- **`uv` is already required** (`uvx` runs `clang-format` and `clang-tidy`, `scripts/cpp.ts`), so Python adds no new program to a development machine.

## In place

- `bun run setup` already fetches every pinned download, the npm packages included, as registry tarballs pinned by sha256 and size (`scripts/lib/setup/`, `pins.generated.json`), so the fonts and the paper shaders no longer come through `bun install`.
- Every script is run as `bun scripts/<name>.ts` from a `package.json` script; nothing imports a script from outside `scripts/` except the `check-app` skill.
- Rust never runs bun; it only names `bun run setup` in error messages (`crates/golden`, `crates/qt`, `crates/skia`, `crates/filament` build scripts) and `bun run <name>` in a few messages and tests (`crates/compile/tests/spec_export.rs`, `crates/packager`, `crates/project/src/files.rs`, `crates/ui/src/look.rs`).

## Work

### The runner

- **A root `justfile` replaces `package.json`'s `scripts` block**, one recipe per command, each running `uv run scripts/<name>.py` or the tool directly (`cargo`, `zig`, `ruff`).
- **Recipe names** follow the current command names with `-` in place of `:` (`just lint-fix`, `just fonts-bundle`), since `just` recipe names can't hold `:`.
- **Every mention of `bun run <name>` and `bun start` changes to `just <name>`**: AGENTS.md (the generated-files table, §3, §4), every `agent-docs/` file that names a command, README, CONTRIBUTING, the pull request template, the `check-app` skill, the CI workflows, and the Rust messages and tests listed above.

### The port

- **Each `scripts/*.ts` becomes `scripts/*.py`, and `scripts/lib/` a package `scripts/opsis_scripts/`** (snake_case module files), run through `uv run` with a root `pyproject.toml`, `uv.lock` and `.python-version`.
- **The libraries change to the standard library or one maintained package each:**

| Now                               | Python                                                                                                                       |
| --------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| `zod`                             | `pydantic`                                                                                                                   |
| `smol-toml`                       | `tomllib`                                                                                                                    |
| `@zip.js/zip.js`                  | `zipfile`                                                                                                                    |
| `node:child_process`, `node:fs`   | `subprocess`, `pathlib`, `shutil`                                                                                            |
| `fetch` (downloads, pins)         | `httpx`                                                                                                                      |
| `oxc-parser` (the comment check)  | `tokenize` and `ast` for Python; the other languages stay line by line                                                       |
| `fast-check`                      | `hypothesis`                                                                                                                 |
| Vitest                            | `pytest`                                                                                                                     |
| `playwright-core` (`brand-icons`) | the recording browser over CDP, from a small Rust binary beside `opsis-capture`'s, or `websockets` against the same protocol |

- **The icon sets (`lucide`, `@tabler/icons`, `iconoir`, `@hugeicons/core-free-icons`, `@mynaui/icons`, `morphicons`, `@lucide/lab`) and `playwright-core` move into setup's npm pins**, read from `.setup/paths.json` like the fonts (`npmPackage`), so `glyphs:bundle` and `playwright:scripts` keep reading the same files.
- **The Opsis oxlint plugin goes**: its rules are about TypeScript (function declarations, components, `findById`). The rules that still mean something in Python come from Ruff's own (`E731`, no assigned lambdas, for function declarations).

### Checks

- **Lint and format**: Ruff (`ruff check --fix`, `ruff format`) for Python; dprint for JSON, Markdown and TOML, pinned by setup like Zig; `cargo fmt`, `zig fmt` and clang-format as now.
- **Types**: mypy in strict mode over `scripts/`, as `just typecheck`.
- **Unused code and dependencies**: `deptry` for the dependencies; Ruff's unused-import and unused-variable rules for the rest.
- **The comment check** (`scripts/comments.py`) learns Python: no `#` comments and no docstrings, with the functional directives (`# noqa`, `# type:`) allowed as today's `oxlint-disable` lines are.
- **Tests**: the 12 Vitest files become pytest files beside their modules, property tests through `hypothesis`; `just test` runs them and `cargo test --workspace`.
- **The editor-data build** that Vitest's global setup runs (`scripts/lib/build-editor-data.ts`) becomes a pytest session fixture.

### Removed

- `package.json`, `bun.lock`, `tsconfig.json`, `oxlint.config.ts`, `oxfmt.config.ts`, `vitest.config.ts`, `knip.config.ts`, `scripts/lib/oxlint-base.ts`, `oxfmt-base.ts`, `vitest-base.ts`, `scripts/oxlint-plugin-opsis.ts`, `node_modules/`.
- The AGENTS.md rules that only TypeScript needs: kebab-case TS file names, function components, function declarations as an oxlint rule, the Opsis oxlint rules, `*.config.ts` configs, "bun runs every script". Their Python counterparts are written where the rule still holds.
- `oven-sh/setup-bun` and `bun install` in both CI workflows, replaced by `astral-sh/setup-uv`, `extractions/setup-just` and `uv sync`.

## Decisions

- **Recipe names**: `-` in place of `:` (`just lint-fix`), or `just` modules so the command reads `just lint fix`.
- **`brand-icons`**: a Rust binary that drives the recording browser the way `opsis-capture` does, or a Python script speaking CDP itself.
- **Type checker**: mypy (stable, the most used), or Astral's `ty` once it has a stable release.

## Risks

- **The port is a rewrite of every script at once**: the setup, release, packaging, e2e and profiling scripts all change together, so each needs running once after the port (`just setup` from an empty cache, `just make`, `just e2e-qt`, `just profile`).
- **The docs site needs a JavaScript toolchain** ([docs-site.md](docs-site.md)): Blume runs under Node with its own `package.json` in `docs/`, so that plan keeps bun or npm for `docs/` alone, outside the root.
- **The glyph and font bundles are generated files**: after the port, `just glyphs-bundle` and `just fonts-bundle` must write exactly the same files, which `git diff` shows.
