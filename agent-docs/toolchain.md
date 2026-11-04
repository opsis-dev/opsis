# Toolchain

The Rust nightly and how to move it, the Zig build of `opsis-vt`'s wrapper, the Clippy workspace allows and the repository's own oxlint rules. Paths are relative to the repository root. The everyday commands and the lint exception format are in [AGENTS.md](../AGENTS.md) §3 and §4; Ghostty itself is in [terminal.md](terminal.md).

## Code map

| Concern                                | Where                                                                                                                 |
| -------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| Nightly pin                            | `rust-toolchain.toml` (with `rustfmt` and `clippy`)                                                                   |
| Minimum compiler                       | `rust-version` in `[workspace.package]`, `Cargo.toml`, inherited by every crate                                       |
| CI toolchain steps                     | every `dtolnay/rust-toolchain` step of `.github/workflows/build.yml`                                                  |
| Workspace lints                        | `[workspace.lints.rust]`, `[workspace.lints.clippy]` in `Cargo.toml`; `[lints] workspace = true` in every crate       |
| Zig build of the wrapper               | `crates/vt/build.rs`, `crates/vt/zig/build.zig`, `crates/vt/zig/build.zig.zon`                                        |
| Opsis oxlint rules                     | `scripts/oxlint-plugin-opsis.ts`                                                                                      |
| Shared oxlint, oxfmt, Vitest configs   | `scripts/lib/oxlint-base.ts`, `scripts/lib/oxfmt-base.ts`, `scripts/lib/vitest-base.ts`, `scripts/lib/tool-config.ts` |
| Comment check                          | `scripts/comments.ts`                                                                                                 |
| Rust test runner (stages FFmpeg first) | `scripts/test-rust.ts`                                                                                                |

## Rules

- **The computer keeps one nightly, the one in `rust-toolchain.toml`.** To move it, in one change:
  - change the date in `rust-toolchain.toml` and in every `dtolnay/rust-toolchain` step of `.github/workflows/build.yml`;
  - raise `rust-version` to the new compiler's;
  - run `rustup default` with the new toolchain and uninstall the old one;
  - move `~/projects/agent-os/rust-toolchain.toml` to the same date and leave it uncommitted there.
- **Zig 0.17 builds the wrapper because Ghostty needs the same major and minor version.** `crates/vt/build.rs` runs `zig build` (the `ZIG` variable names another program) and refuses any other version.
- **`build.rs` always names the Zig target** (`x86_64-linux-gnu`, `aarch64-macos`, `x86_64-windows-msvc`…), because Zig's native target compiles for the building computer's processor.
- **CI installs Zig 0.17.0 with `mlugg/setup-zig` in every job and runs `zig build --fetch` before Cargo.** Zig unpacks the packages into `crates/vt/zig/zig-pkg` (ignored by git); the tarballs stay in Zig's global cache.
- **The Clippy workspace allows are permanent and no line is added to them.** Each exists because:
  - `missing_errors_doc`, `missing_panics_doc`: they ask for doc comments, which the no-comments rule bans.
  - `option_if_let_else`: it would turn every `if let … else` into `map_or_else`.
  - `too_many_lines`: it caps functions at 100 lines.
  - `must_use_candidate`: it asks for `#[must_use]` on every function whose result could be ignored.
  - `struct_excessive_bools`: it flags structs with more than three `bool` fields, mostly settings structs with their switches.
  - `significant_drop_tightening`: it guesses a lock's lifetime from its last use, so it flags guards something still borrows and locks held across an `await` on purpose. A lock that blocks others needlessly is still tightened by hand (`check_media_files`).
  - `module_name_repetitions`: it flags item names that repeat their module's name; it sits in `restriction`, and the allow keeps it off should Clippy move it into `pedantic`.
- **The Opsis oxlint rules that fix code add their import from `common/`**, so run `lint:fix` until it reports nothing ([AGENTS.md](../AGENTS.md) §2).

## Deliberate limits

- **TypeScript 7 has no parser API**, so the comment check parses TS with `oxc-parser`; the native compiler only type-checks.
- **`lint:zig` builds the wrapper in Debug as a check** into `target/zig-check`; it is not the build Cargo links.
