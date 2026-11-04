# Fonts

The bundled and terminal fonts, the user's font library, project fonts and their roles, and how a font name becomes a family everywhere a film draws text. Paths are relative to the repository root. Elements' declared families are in [elements.md](elements.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern          | Where                                                                                                                              |
| ---------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| Bundled families | `public/fonts/bundled`, `crates/compile/src/fonts/bundled.generated.json`, list in `scripts/fonts-bundle.ts`                       |
| Terminal stack   | `public/fonts/bundled/terminal`, `TERMINAL_FONTS`                                                                                  |
| Library          | `crates/fonts` (`wuff`, `skrifa`, `fontdb`) → `installed_font_families`; files in `<data folder>/fonts`, rows in the `fonts` table |
| Name → family    | `crates/compile/src/fonts/mod.rs` → `family_for`, `font_stack`                                                                     |
| Film font table  | `FilmFonts`, `film_fonts`, `used_font_names`                                                                                       |
| Page side        | `FontProvider`, `packages/video/src/fonts/registry.ts`                                                                             |
| Editor picker    | `font_choices`, `list_fonts` (`FontChoice`)                                                                                        |

## Rules

- **16 families are bundled from Fontsource into `public/fonts/bundled`** (Latin, all weights, italics where they exist, about 1 MB).
- **The terminal's fixed stack is bundled under family names of its own (`TERMINAL_FONTS`)**: JetBrains Mono 2.304, Symbols Nerd Font Mono 3.4.0 and Google's colour bitmap build of Noto Color Emoji (downloaded by `fonts-bundle.ts` and checked by sha256), then Noto Sans Symbols 2 and Noto Sans SC as Fontsource subsets.
  - Each face but the first two has its `unicodeRange`; the emoji face takes Fontsource's emoji ranges from U+2000 up, because Fontsource's own Noto Color Emoji is OpenType-SVG, which the recording browser doesn't draw.
  - Those families are `hidden` from the pickers, and faces with a range load only when a page's text needs them, so a terminal frame holds the render until `document.fonts.load` has the faces for its text.
- **The library lives in `<data folder>/fonts` (the files, `library:` refs to the file server) and the database's `fonts` table** (one row per family; a row that doesn't parse is kept as it is). It is filled from font files or ZIPs (one file per style, variable and woff2 first), fonts installed on the computer, or Fontsource online.
- **A project copies the library fonts it uses into `assets/fonts/<id>/` and lists them in `settings.fonts.families`**, so it works anywhere.
- **`settings.fonts` holds the roles `main`, `heading` (empty = main) and `code`.**
- **Rust works out which family every font name means (`family_for`)**: `df-main`, `df-heading` and `df-code` point at the role fonts, `Inter` follows `main` and `JetBrains Mono` follows `code`, so every element follows the project fonts. Per-node fonts (the `font` widget, a family name) are found by name and turned into a CSS stack by `font_stack`.
- **Each spec carries that table (`FilmFonts`)**: every name a film can use, lowercased, and the names to load first (the roles and every font its items name). The page's `FontProvider` and `registry.ts` only look names up in it and register the faces.
- **3D text and text drawn into images pick their face from the same `family_for`** ([elements.md](elements.md), [images.md](images.md)).
- **The editor's picker gets its groups, previews and stacks from `font_choices` and the library from `list_fonts`**; the online list says which fonts are already there (`added`).
- **Fonts are never loaded from the internet at render time**, and getting fonts online asks first.
