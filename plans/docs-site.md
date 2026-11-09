# Docs site

A user docs site for Opsis: a Blume site in `docs/` at the repository root, deployed as a static site, with screenshots of the real editor taken by the Qt scenarios and turned into web images by a script. It holds pages for people who use Opsis, not for agents: `AGENTS.md` and `agent-docs/` stay as they are.

## How to read this plan

- Section 1 makes the site, section 2 its screenshots, section 3 deploys it, section 4 writes the first pages. Each builds on the one before.
- Each section has the same parts where they apply: **In place**, **Work** (numbered, only what is left), **Decided**, **Risks**. Rules from `AGENTS.md` and `agent-docs/` hold throughout; when they disagree with this plan, they win.
- Each piece of work ends with the usual checks: `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, plus `bun run validate` in `docs`.
- The work adds a domain doc (`agent-docs/docs-site.md`, with its row in the `AGENTS.md` §8 table), a line in the `AGENTS.md` §6 layout and a row in the generated-code table in §2, all in the same change as the code they describe.

## Summary

| Topic          | What users get                                     | Left to do                                        | Status      |
| -------------- | -------------------------------------------------- | ------------------------------------------------- | ----------- |
| 1. The site    | A docs site to read before and while using Opsis   | the `docs/` member, its config and checks         | not started |
| 2. Screenshots | Pictures of the real editor beside each step       | the shot list, the script that moves and converts | not started |
| 3. Deploy      | The site on the web                                | the Vercel project                                | not started |
| 4. First pages | Install, first project, browser and terminal takes | the pages, through the three docs agents          | not started |

## 1. The site

**In place.**

- Blume 2.2.7 is the latest release (MIT, `node >=22.19.0`). Its CLI is `bin/blume.mjs` with a `#!/usr/bin/env node` line, so `bun run` starts it under Node.
- The three docs agents (`docs-feature-finder`, `docs-writer`, `docs-verifier`) are in `.claude/agents/`, ignored through `.git/info/exclude`. They already target `docs/content/` and `bun run validate` in `docs`.

**Work.**

1. **The member.** Make `docs/` with `blume init`, then make it a workspace member: add `docs` to `workspaces` in the root `package.json`, name it `@opsis/docs`, and add `blume` with `bun add blume` inside it. Its scripts: `dev` (`blume dev`), `build` (`blume build`, output `dist/`), `typecheck` (`blume check`), `validate` (`blume validate --strict`).
2. **Config.** `docs/blume.config.ts` with the title, the logo from `brand/logo.svg` (the one source for the app icon), `content: { root: "content" }`, and the theme. `.blume/`, `dist/` and `node_modules/` in `.gitignore`.
3. **Checks.** Its own `tsconfig.json` (extending `tsconfig.base.json`, including Blume's generated types), `oxlint.config.ts` and the oxfmt coverage, so `typecheck`, `lint:fix` and `format:fix` cover it through turbo. `bun run lint:comments` covers `docs` (`scripts/comments.ts`). `validate` joins `bun run test` or runs as its own turbo task.
4. **Docs.** `agent-docs/docs-site.md`: the code map, the rules the agents carry (customer scope, the style, the `.md`/`.mdx` choice, frontmatter from Blume's schema), and how the screenshots and the deploy work. Add `docs/` to the `AGENTS.md` §6 layout.

**Decided.**

- The site lives in `docs/` at the root, not in `apps/`: nothing imports it and it builds no part of the app.
- Pages are written for people who use Opsis. The same scope and style rules as the agents: simple English, "you", UI labels in bold, no emojis, no em or en dashes, none of the names `AGENTS.md` §7 keeps out of the app's copy.
- No custom Astro pages at first; the content and Blume's own layouts are enough.

**Risks.**

- Blume's `init` writes its own `.gitignore`, `package.json` and formatter settings; fold them into the root's tools instead of keeping a second set.

## 2. Screenshots

The Qt scenarios already take a picture of every step: `bun run e2e:qt tour --shots` (or `runs`) writes `~/.cache/opsis-dev/qt-shots/<scenario>/<step name>.png`, plus `<step name>-<window>.png` for each other open window (`crates/qt/cpp/scenario.cc` → `Scenario::Shoot`, [e2e.md](../agent-docs/e2e.md)). The docs need some of those pictures, moved into `docs/public/images/` and turned into web images.

**Work.**

1. **The shot list.** `docs/screenshots.json`: one entry per image, with the image's name, the scenario (`tour` or `runs`), the step name and, when it isn't the main window, the window's suffix.
2. **The script.** `scripts/docs-shots.ts`, run as `bun run docs:shots`: it runs `bun run e2e:qt` with `--shots` for the scenarios the list names (only those), then converts each listed shot into `docs/public/images/<name>.webp` with the staged FFmpeg (`scripts/lib/ffmpeg.ts` → `stageFfmpeg`, `target/ffmpeg/opsis-ffmpeg-<target triple>`) and fails, naming the entry, when a listed shot is missing, which is what a renamed or removed step looks like.
3. **Generated code.** Add `docs/public/images/**` to the generated-code table in `AGENTS.md` §2, owned by the shot list and the scenarios, made again with `bun run docs:shots`.
4. **New shots.** A picture no step reaches yet gets a new step in `tour` (`crates/qt/cpp/e2e_steps.cc` and its parts) or `runs` (`crates/qt/cpp/e2e_runs.cc` and its parts), with a check, then an entry in the list. Never a separate scenario for the docs.
5. **The agents.** Update the writer and verifier in `.claude/agents/` to the list and the script: the writer adds list entries instead of copying PNGs, the verifier checks each image has an entry and a step.

**Decided.**

- Screenshots come only from `tour` and `runs`. Each image is one picture: the editor has one look, so there are no light and dark pairs.
- The converted images are kept in the repository so the site builds without a display or the dev engine.
- **Lossless WebP, not JPEG.** Screenshots are text, thin lines and flat colour: lossless WebP keeps them exact and comes out smaller than the PNG, where JPEG blurs text edges.
- **The bundled FFmpeg converts them** (its `libwebp` encoder, listed by `-encoders` on the staged program), so the script adds no dependency.
- **Shots are taken at scale 2.** The scenarios run on a `1600x1000` virtual screen (`SCREEN` in `scripts/e2e-qt.ts`) at scale 1, which looks soft on high-density screens; `docs:shots` runs them with `QT_SCALE_FACTOR=2` on a screen twice the size, so the editor keeps its 1600 by 1000 layout and every picture has twice the pixels.

**Risks.**

- A scenario check that reads pixels rather than widget positions may fail at scale 2; such a check reads the device pixel ratio instead.

- `runs` stops without the dev engine in `~/.config/Opsis/engine`, so pictures from it need the engine set up on the computer that makes them.
- Step names become part of the docs' inputs; renaming one breaks `docs:shots` until the list follows, by design.

## 3. Deploy

**In place.** A static `blume build` writes plain HTML, CSS and a local search index to `dist/`; any static host serves it. On Vercel Blume reads the site's address from the platform at build time, preferring the production domain over preview addresses.

**Work.**

1. **The Vercel project.** Root Directory `docs`, with **Include source files outside of the Root Directory in the Build Step** left on (the install reads the root `bun.lock`), the build command `bun run build`, the output `dist`, Node 22.19 or newer, deploying `main` and a preview for every pull request.
2. **Links.** A link to the site in `README.md` once it is live.

**Decided.**

- **Vercel hosts it**: no host config in the repository, the site's address detected at build time, a preview for every pull request.
- **No address is named.** `blume.config.ts` sets no `deployment.site`; the site uses whatever address the Vercel project serves.

**Risks.**

- Vercel installs the whole Bun workspace from the root `bun.lock` to build one member; check the install time and any install scripts, and filter the install to `@opsis/docs` if it is slow.

## 4. First pages

**Work.**

1. **Pages, in order:** installing Opsis on Linux (the packages `README.md` says work) and setting up the engine; a first project from a template; recording a browser take; recording a terminal take; camera, cursor and styles; exporting. Each through the agents: the feature finder for the area, the writer, then the verifier.
2. **Screenshots** for each page through section 2.

**Decided.** Only what works today: no Windows or macOS pages until `plans/release.md` section 2 is done, and nothing from `plans/` or `TODO.md`.
