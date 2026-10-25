# Types and contracts

Rust is the one source of the shapes that cross a boundary: the project file, the spec, run results, run events, settings, IPC messages, sidecar messages and API bodies. TS gets generated types; forms and the Local API get JSON Schema generated from the same Rust types. Nothing is kept twice by hand except 1,134 lines of small logic, which shared fixtures keep in step.

## What it adds

- No zod/Rust drift: a field added in Rust appears in the generated TS, and a type error shows where the UI or the video bundle must follow.
- One description of every node setting serves the Inspector, the Local API's `/nodes/{type}` and validation.

## Fit with Opsis

Already in place:

- The zod schemas in `src/shared/schema` (workflow, timeline spec, values, actions, recording, events; 1,710 lines with the plumbing types) and `z.toJSONSchema` output for `/nodes/{type}` (`paramsJsonSchema`, `src/shared/api/catalog.ts`).
- Field metadata in zod `.meta()` (`FieldMeta`, `src/shared/elements/introspect.ts`): `widget`, `unit`, `labels`, `presets`, `showIf`, `advanced`, `hidden`, `timeKey`, `themes` and the rest listed in `AGENTS.md`.

Missing:

- Rust types, generators, and a way to put field metadata in Rust.

## The tools (checked 2026-10-25 on crates.io)

| Job                              | Crate                     | Version, licence                       | Use                                                                                                              |
| -------------------------------- | ------------------------- | -------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| Rust → TS types                  | `specta` + `tauri-specta` | 1.0.5 and 1.0.2 (both 2026-10-21), MIT | Exports Rust structs and enums as TS; `tauri-specta` types every Tauri command and, per its README, events       |
| Rust → TS types (alternative)    | `ts-rs`                   | 12.0.1, MIT                            | Plain type export without command typing; only needed if specta lacks something                                  |
| Rust → JSON Schema               | `schemars`                | 1.2.3, MIT                             | Node settings and API bodies as JSON Schema; `#[schemars(extend("x-widget" = "color"))]` adds the field metadata |
| JSON Schema → Rust (not planned) | `typify`                  | 0.8.0, Apache-2.0                      | Only if a schema must come from outside Rust                                                                     |
| Validating JSON against a schema | `jsonschema`              | 0.60.1, MIT                            | API bodies and model answers (`plans/ai-assist.md`)                                                              |

## Work

1. **Rust types.** Port the 1,710 lines listed as types in `workspace.md` to `serde` structs and enums in the `app` crate, with `#[serde(rename_all = "camelCase")]` and tagged enums matching today's JSON (the project file format does not change). Derive `specta::Type` and `schemars::JsonSchema` on each.
2. **Generated TS.** A build step (`cargo run --bin export-types` or tauri-specta's builder at startup in debug) writes `packages/ui/src/generated/bindings.ts` and `packages/video/src/spec/generated.ts`. Generated files join the "never hand-edit" table in `AGENTS.md`; a test fails when they are stale, like `schema.generated.sql` today.
3. **Field metadata.** Every `FieldMeta` key becomes an `x-` keyword through `schemars` `extend`: `x-widget`, `x-unit`, `x-labels`, `x-zero-label`, `x-presets`, `x-show-if`, `x-advanced`, `x-hidden`, `x-time-key`, `x-themes` and the rest, plus `x-framesAt` for the 30 fps settings. A Rust test checks every node's schema uses only widget names from `FIELD_WIDGETS`.
4. **Where zod stays.** Nowhere for shared shapes. The UI may still use zod for its own local forms (for example the New project dialog) if useful; nothing crossing a boundary is parsed with zod.
5. **Both-sides logic.** The 21 files (1,134 lines, `code-split.md` §5) stay TS where the UI or the video bundle runs them and get a Rust port. Their 613 lines of tests become JSON fixtures in `fixtures/` at the repo root (input and expected output per case) read by a vitest test and a Rust test, so both implementations are checked against the same cases.
6. **Data written once.** `sounds`, `backdrops`, templates and element metadata are JSON files that Rust embeds (`include_str!`) and the UI imports.

## Risks

- **Tagged unions.** Today's values use a `type` field; `serde(tag = "type")` and specta must export them as discriminated unions the UI narrows the same way. Check one round trip per value kind in the fixture tests.
- **Generated churn.** Every Rust type change touches generated files; keep them out of review noise with a `.gitattributes` `linguist-generated` line.

## Sources

- crates.io API for `specta`, `tauri-specta`, `ts-rs`, `schemars`, `typify`, `jsonschema` (2026-10-25); [schemars `JsonSchema` derive: `extend` and `transform`](https://docs.rs/schemars/latest/schemars/derive.JsonSchema.html); [tauri-specta](https://github.com/specta-rs/tauri-specta) (2026-10-25).
