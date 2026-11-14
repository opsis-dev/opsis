import { execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { cachedName } from "./lib/pins";
import { readPins } from "./lib/setup/manifest";
import { downloadsDir, readPaths } from "./lib/setup/store";
import { ROOT } from "./lib/tool-config";

const OUT = path.join(ROOT, "QT_NOTICES.generated.md");
const BUILD_RS = path.join(ROOT, "crates", "qt", "build.rs");
const MODULE_SOURCES = ["qtbase", "qtsvg"];
const OPERATORS = new Set(["AND", "OR", "WITH"]);

type Component = {
  target: string;
  name: string;
  usage: string;
  licence: string;
  copyright: string;
  homepage: string;
};

function textField(block: string, field: string): string {
  const inline = block.match(new RegExp(`^${field}: (?!<text>)(.*)$`, "m"));
  if (inline) return inline[1].trim();
  const multi = block.match(new RegExp(`^${field}: <text>([\\s\\S]*?)</text>`, "m"));
  return multi ? multi[1].trim() : "";
}

function commentField(comment: string, field: string): string {
  const line = comment.split("\n").find((row) => row.trim().startsWith(`${field}:`));
  return line ? line.slice(line.indexOf(":") + 1).trim() : "";
}

function linkedArchives(prefix: string): { modules: Set<string>; plugins: string[] } {
  const source = readFileSync(BUILD_RS, "utf8");
  const modules = [
    ...(source.match(/const QT_MODULES[^=]*= \[([^\]]*)\]/)?.[1] ?? "").matchAll(/"(\w+)"/g),
  ].map((found) => found[1]);
  const plugins = [...source.matchAll(/\(\s*"([^"]+)",\s*"([^"]+)",\s*"([^"]+)",?\s*\)/g)];
  if (modules.length === 0 || plugins.length === 0)
    throw new Error(`Couldn't read QT_MODULES and STATIC_PLUGINS from ${BUILD_RS}.`);
  const seen = new Set<string>();
  const queue = [
    ...modules.map((module) => path.join(prefix, "lib", `libQt6${module}.a`)),
    ...plugins.map(([, folder, name]) => path.join(prefix, "plugins", folder, `lib${name}.a`)),
  ];
  while (queue.length > 0) {
    const archive = queue.shift() as string;
    if (seen.has(archive)) continue;
    seen.add(archive);
    const prl = archive.replace(/\.a$/, ".prl");
    if (!existsSync(prl)) continue;
    const libs = readFileSync(prl, "utf8").match(/^QMAKE_PRL_LIBS = (.*)$/m)?.[1] ?? "";
    for (const token of libs.split(/\s+/)) {
      const named = token.match(/libQt6(\w+)\.a$/);
      if (named) queue.push(path.join(prefix, "lib", `libQt6${named[1]}.a`));
    }
  }
  const qtModules = new Set(
    [...seen].flatMap((archive) => {
      const named = path.basename(archive).match(/^libQt6(\w+)\.a$/);
      return named ? [named[1]] : [];
    }),
  );
  return { modules: qtModules, plugins: plugins.map((found) => found[3]) };
}

function readSbom(prefix: string): { blocks: string[]; extracted: Map<string, string> } {
  const folder = path.join(prefix, "sbom");
  const text = readdirSync(folder)
    .filter((name) => name.endsWith(".spdx"))
    .map((name) => readFileSync(path.join(folder, name), "utf8"))
    .join("\n\n");
  const blocks = text.split(/\n(?=PackageName: |LicenseID: )/);
  const extracted = new Map<string, string>();
  for (const block of blocks) {
    const id = textField(block, "LicenseID");
    if (id) extracted.set(id, textField(block, "ExtractedText"));
  }
  return { blocks: blocks.filter((block) => block.startsWith("PackageName: ")), extracted };
}

function components(blocks: string[], targets: Set<string>): Component[] {
  const found: Component[] = [];
  for (const block of blocks) {
    const name = textField(block, "PackageName");
    const target = name.includes("_Attribution_")
      ? name.slice(0, name.indexOf("_Attribution_"))
      : name;
    const attribution = name.includes("_Attribution_") || name.startsWith("Bundled");
    if (!attribution || !targets.has(target)) continue;
    const comment = textField(block, "PackageComment");
    found.push({
      target,
      name: commentField(comment, "Name") || name,
      usage: commentField(comment, "Qt usage") || commentField(comment, "Description"),
      licence: textField(block, "PackageLicenseConcluded"),
      copyright: textField(block, "PackageCopyrightText"),
      homepage: textField(block, "PackageDownloadLocation"),
    });
  }
  return found.sort((a, b) => a.name.localeCompare(b.name));
}

function licenceIds(expression: string): string[] {
  return expression
    .split(/[()\s]+/)
    .filter((token) => token && !OPERATORS.has(token) && token !== "NOASSERTION");
}

function licenceTexts(ids: Set<string>, extracted: Map<string, string>): Map<string, string> {
  const work = mkdtempSync(path.join(tmpdir(), "opsis-qt-licences-"));
  try {
    const { qt } = readPins();
    const folders = MODULE_SOURCES.map((module) => {
      const root = `${module}-everywhere-src-${qt.version}`;
      const source = qt.sources.find((s) => s.module === module);
      if (!source) throw new Error(`The Qt pins have no ${module}.`);
      execFileSync(
        "tar",
        [
          "-xf",
          path.join(downloadsDir(), cachedName(source.url)),
          "-C",
          work,
          "--wildcards",
          `${root}/LICENSES/*`,
        ],
        { stdio: "inherit" },
      );
      return path.join(work, root, "LICENSES");
    });
    const texts = new Map<string, string>();
    const missing: string[] = [];
    for (const id of [...ids].sort()) {
      const file = folders
        .map((folder) => path.join(folder, `${id}.txt`))
        .find((candidate) => existsSync(candidate));
      const text = extracted.get(id) ?? (file ? readFileSync(file, "utf8") : undefined);
      if (text) texts.set(id, text.trim());
      else missing.push(id);
    }
    if (missing.length > 0) throw new Error(`No licence text for ${missing.join(", ")}.`);
    return texts;
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
}

function fence(text: string): string {
  return ["```", text.replaceAll("```", "'''"), "```"].join("\n");
}

function render(found: Component[], texts: Map<string, string>): string {
  const parts = [
    "# Notices for the code Qt compiles in",
    "",
    `Opsis links Qt ${readPins().qt.version} statically. These are the third-party components compiled into the Qt modules and plugins it links, read from that Qt build's software bill of materials by \`bun run qt:notices\`. Each component's licence text follows the list, once per licence.`,
    "",
  ];
  for (const component of found) {
    parts.push(`## ${component.name}`, "");
    const facts = [component.usage, `Licence: ${component.licence}.`].filter(Boolean).join(" ");
    parts.push(facts, "");
    if (component.homepage && component.homepage !== "NOASSERTION")
      parts.push(component.homepage, "");
    if (component.copyright && component.copyright !== "NOASSERTION")
      parts.push(fence(component.copyright), "");
  }
  parts.push("## Licence texts", "");
  for (const [id, text] of texts) parts.push(`### ${id}`, "", fence(text), "");
  return `${parts.join("\n").trimEnd()}\n`;
}

function main(): void {
  const prefix = readPaths().qt;
  const { modules, plugins } = linkedArchives(prefix);
  const { blocks, extracted } = readSbom(prefix);
  const names = new Set(blocks.map((block) => textField(block, "PackageName")));
  const targets = new Set<string>(plugins);
  for (const module of modules) {
    if (names.has(module)) targets.add(module);
    else if (names.has(`${module}Private`)) targets.add(`${module}Private`);
    else throw new Error(`The bill of materials has no target for libQt6${module}.a.`);
  }
  for (const plugin of plugins)
    if (!names.has(plugin)) throw new Error(`The bill of materials has no plugin ${plugin}.`);
  const found = components(blocks, targets);
  const ids = new Set(found.flatMap((component) => licenceIds(component.licence)));
  writeFileSync(OUT, render(found, licenceTexts(ids, extracted)));
  console.log(
    `${found.length} components from ${targets.size} Qt targets, ${ids.size} licences: ${path.relative(ROOT, OUT)}`,
  );
}

main();
