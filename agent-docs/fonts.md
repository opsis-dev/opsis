# Fonts

The bundled and terminal fonts, the user's font library, project fonts and their roles, and how a font name becomes a family everywhere a film draws text. Paths are relative to the repository root. Elements' declared families are in [elements.md](elements.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern          | Where                                                                                                                                                                                                       |
| ---------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Bundled families | `public/fonts/bundled`, `crates/compile/src/fonts/bundled.generated.json`, list in `scripts/fonts-bundle.ts`                                                                                                |
| Terminal stack   | `public/fonts/bundled/terminal`; `TERMINAL_FONTS` in `crates/compile/src/terminal/mod.rs` (with `terminal_fonts`, which the painters use, [terminal.md](terminal.md)) and `packages/video/src/terminal/view.tsx` |
| Library          | `crates/fonts` (`wuff`, `skrifa`, `fontdb`): `library.rs` → `FontLibrary`, `inspect.rs` → `plan_font_files`, `pick_faces`, `online.rs` → `catalog`, `download`; `crates/session/src/fonts.rs` → `list_fonts`, `installed_font_families`; files in `<data folder>/fonts`, rows in the `fonts` table (`crates/store/schema.sql`) |
| `library:` refs  | `crates/compile/src/asset_ref.rs` → `LIBRARY_SCHEME`, `library_ref`; resolved by `crates/ui/src/ui.rs` → `font_file`                                                                                         |
| Roles            | `crates/compile/src/edits/workflow.rs` → `set_font_role`                                                                                                                                                     |
| Fonts page       | `crates/qt/cpp/fonts_page.cc` → `FontsCard`, `AskOnline`, `OnlineAllowed`; host calls in `crates/ui/src/ui.rs` (`font_library`, `add_font_files`, `installed_fonts`, `font_review`, `font_licences`)        |
| Name → family    | `crates/compile/src/fonts/mod.rs` → `family_for`, `ROLE_NAMES`, `FAMILY_ALIASES`, `font_stack`                                                                                                              |
| Film font table  | `crates/compile/src/fonts/mod.rs` → `film_fonts`, `used_font_names`; `crates/types/src/fonts.rs` → `FilmFonts`                                                                                              |
| Page side        | `packages/video/src/fonts/font-provider.tsx` → `FontProvider`; `packages/video/src/fonts/registry.ts`                                                                                                       |
| Editor picker    | `crates/compile/src/fonts/mod.rs` → `font_choices`, through `crates/session/src/fonts.rs`; `crates/qt/cpp/font_picker.cc`; `crates/types/src/fonts.rs` → `FontChoice`                                       |

## Rules

- **The families in `FONTS` (`scripts/fonts-bundle.ts`) are bundled from Fontsource into `public/fonts/bundled` by `bun run fonts:bundle`, never edited by hand** (Latin, every weight as one variable face where Fontsource has one, italics where they exist).
- **The terminal's fixed stack is bundled under family names of its own (`TERMINAL_FONTS`)**: JetBrains Mono, Symbols Nerd Font Mono and Google's colour bitmap build of Noto Color Emoji (downloaded by `fonts-bundle.ts` and checked by sha256, versions pinned there), then Noto Sans Symbols 2 and Noto Sans SC as Fontsource subsets.
  - `TERMINAL_FONTS` is written in Rust and in the page; `packages/video/src/terminal/view.test.tsx` checks the page's list against the bundled hidden families, and nothing checks the Rust one, so a wrong Rust list shows only as terminal faces missing from `terminal_fonts` and from the families a film loads (`used_font_names`).
  - Each face but the first two has its `unicodeRange`; the emoji face takes Fontsource's emoji ranges from U+2000 up, because Fontsource's own Noto Color Emoji is OpenType-SVG, which the recording browser doesn't draw.
  - Those families are `hidden` from the pickers, and faces with a range load only when a page's text needs them, so a terminal frame holds the render until `document.fonts.load` has the faces for its text.
- **The library lives in `<data folder>/fonts` (the files, `library:` refs to the file server) and the database's `fonts` table** (one row per family; a row that doesn't parse is kept as it is). It is filled from font files or ZIPs (one file per style and weight, a variable file first, then woff2 before woff, TrueType and OpenType), fonts installed on the computer, or Fontsource online.
- **A project copies the library fonts it uses into `assets/fonts/<id>/` and lists them in `settings.fonts.families`**, so it works anywhere; a library font a project hasn't copied doesn't resolve, since the lookup never reads the library.
- **`settings.fonts` holds the roles `main`, `heading` (empty = main) and `code`.**
- **Rust works out which family every font name means (`family_for`)**: the names in `ROLE_NAMES` (`df-main`, `df-heading`, `df-code`, and `Inter` and `JetBrains Mono` for main and code) point at the role fonts, so every element follows the project fonts, and `FAMILY_ALIASES` swaps a family name for the one to look up; the lookup tries the project's families, then the bundled ones (`find_family`). Per-node fonts (the `font` widget, a family name) become a CSS stack through `font_prop` and `font_stack`, whose fallback (`cursive`, `monospace`, `serif` or `sans-serif`) comes from the bundled family's category, so a family only the project has falls back to `sans-serif`.
- **Each spec carries that table (`FilmFonts`)**: every name a film can use, lowercased (the hidden terminal families only when the film has a terminal take), and the names to load first (the roles and every font its items name). The page's `FontProvider` and `registry.ts` only look names up in it and register the faces.
  - Each role name and alias is a key holding the family it resolves to, and a hosted spec with no table gets one built from the bundled families alone (`crates/compile/src/hosted.rs`).
- **3D text and text drawn into images pick their face from the same `family_for`** ([elements.md](elements.md), [images.md](images.md)).
- **The editor's picker gets its groups, previews, stacks and the library from `font_choices`**, which hides `hidden` families and leaves out built-in and library families the project already has; each preview loads under a name of its own (`df-preview-<source>-<id>`), so it never takes a real family's place.
- **The Fonts settings page lists the library with `list_fonts`**, and its online list says which fonts are already there (`added`).
- **Fonts are never loaded from the internet at render time.**
- **Getting fonts online asks first** (`AskOnline`, "Go online to get fonts?", `fonts_page.cc`).

## Deliberate limits

- **The library keeps one pending review of added files** (`release_inspected` keeps only the latest).
- **Online fonts are Latin only**: a family with no Latin files can't be added (`NoLatinFiles`).
