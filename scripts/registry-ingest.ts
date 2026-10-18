import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { z } from "zod";
import { errorMessage } from "../src/shared/errors";
import { isRecord } from "../src/shared/guards";
import {
  renderLoaderEntry,
  renderLoaders,
  renderMeta,
  renderMetaEntry,
  type Resolved,
} from "./lib/registry-codegen";
import {
  controlsFor,
  defaultProblems,
  namesLibrary,
  parseConfig,
  propControls,
  propDocsSchema,
  type ParsedConfig,
  type PropDoc,
} from "./lib/registry-controls";
import { install, moveInstalled, rewriteImports } from "./lib/registry-install";
import { ROOT, SRC_DIR, VIDEO_DIR } from "./lib/tool-config";
import { literalOf, scratchFile } from "./lib/ts-literal";
import {
  CURATED,
  DESCRIPTIONS,
  REGISTRIES,
  unknownOverrideNames,
  type CuratedElement,
} from "./registry-ingest.list";

const OUT_META = path.join(SRC_DIR, "shared", "elements", "elements.generated.ts");
const OUT_COMPONENTS = path.join(VIDEO_DIR, "registry", "components.generated.ts");
const skipAdd = process.argv.includes("--no-add");
const onlyIndex = process.argv.indexOf("--only");
const only = onlyIndex === -1 ? undefined : process.argv[onlyIndex + 1];
if (only && skipAdd)
  throw new Error("--only picks the registry to install, so it does nothing with --no-add.");

const registryItemSchema = z.object({ description: z.string().nullish() });

async function fetchDescription(item: CuratedElement): Promise<string> {
  const url = REGISTRIES[item.registry].itemUrl(item.name);
  const res = await fetch(url);
  if (!res.ok) throw new Error(`${url}: ${res.status}`);
  const { description } = registryItemSchema.parse(await res.json());
  return (description ?? "")
    .split(/(?<=[.!?])\s+/)
    .map((s) => s.trim())
    .filter((s) => s && !namesLibrary(s))
    .slice(0, 2)
    .join(" ");
}

async function descriptionOf(
  item: CuratedElement,
  key: string,
  problems: string[],
): Promise<string | undefined> {
  const known = item.description ?? DESCRIPTIONS[item.name];
  if (known) return known;
  try {
    const fetched = await fetchDescription(item);
    if (fetched) return fetched;
    problems.push(`${key}: the registry gives no usable description; add one to DESCRIPTIONS`);
  } catch (error) {
    problems.push(`${key}: no description (${errorMessage(error)}); add one to DESCRIPTIONS`);
  }
  return undefined;
}

async function fetchConfig(item: CuratedElement, installed: string): Promise<ParsedConfig | null> {
  const reg = REGISTRIES[item.registry];
  for (const dir of reg.configDirs) {
    const url = `${reg.rawBase}/${dir}/${item.name}/config.ts`;
    const res = await fetch(url);
    if (res.ok) return parseConfig(await res.text(), item.name);
  }
  return parseConfig(readFileSync(installed, "utf8"), item.name);
}

async function fetchPropDocs(url: string): Promise<PropDoc[]> {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`${url}: ${res.status}`);
  return propDocsSchema.parse(await res.json()).props;
}

const durationTables = new Map<string, Promise<Record<string, number>>>();

function previewDurations(url: string): Promise<Record<string, number>> {
  let pending = durationTables.get(url);
  if (!pending) {
    pending = fetch(url)
      .then((res) => {
        if (!res.ok) throw new Error(`${url}: ${res.status}`);
        return res.text();
      })
      .then((text) => {
        const sf = scratchFile("preview", "ts", text);
        const table =
          literalOf(sf.getVariableDeclaration("PREVIEW_META")?.getInitializer(), isRecord, sf) ??
          {};
        const fallback =
          literalOf(
            sf.getVariableDeclaration("PREVIEW_DEFAULTS")?.getInitializer(),
            isRecord,
            sf,
          ) ?? {};
        const out: Record<string, number> = {
          "*": typeof fallback.durationInFrames === "number" ? fallback.durationInFrames : 120,
        };
        for (const [k, v] of Object.entries(table)) {
          const d = isRecord(v) ? v.durationInFrames : undefined;
          if (typeof d === "number") out[k] = d;
        }
        return out;
      });
    durationTables.set(url, pending);
  }
  return pending;
}

function exportedNames(file: string): string[] {
  const text = readFileSync(file, "utf8");
  return [...text.matchAll(/export\s+(?:default\s+)?(?:function|const)\s+([A-Za-z0-9_]+)/g)].map(
    (m) => m[1],
  );
}

function findElementFile(item: CuratedElement): string | null {
  const dir = path.join(VIDEO_DIR, "elements", REGISTRIES[item.registry].installDir(item));
  const candidates = [`${dir}/${item.name}.tsx`, `${dir}/${item.name}/index.tsx`];
  return candidates.find((c) => existsSync(c)) ?? null;
}

const pascal = (s: string) =>
  s.replace(/(^|[-_])([a-z0-9])/g, (_m, _p, c: string) => c.toUpperCase());

function exportNameOf(
  item: CuratedElement,
  file: string,
  cfg: ParsedConfig | null,
): string | undefined {
  const exported = exportedNames(file);
  return (
    item.exportName ??
    (cfg?.componentName && exported.includes(cfg.componentName) ? cfg.componentName : undefined) ??
    exported.find((e) => e === pascal(item.name)) ??
    exported.find((e) => /^[A-Z]/.test(e))
  );
}

async function registryDuration(item: CuratedElement): Promise<number | undefined> {
  const url = REGISTRIES[item.registry].durationsUrl;
  if (!url) return undefined;
  const durations = await previewDurations(url);
  return durations[item.name] ?? durations["*"];
}

type Ingested = { meta: string } & ReturnType<typeof renderLoaderEntry>;

async function resolveItem(item: CuratedElement, problems: string[]): Promise<Resolved | null> {
  const file = findElementFile(item);
  if (!file) {
    problems.push(`${item.registry}/${item.name}: not installed`);
    return null;
  }
  if (item.installOnly) return null;
  const reg = REGISTRIES[item.registry];
  const key = `${reg.keyPrefix}/${item.name}`;
  const cfg = item.noConfig || reg.propsUrl ? null : await fetchConfig(item, file);
  const description = await descriptionOf(item, key, problems);
  const exportName = exportNameOf(item, file, cfg);
  if (!exportName) {
    problems.push(`${item.registry}/${item.name}: no component export found`);
    return null;
  }
  const source = readFileSync(file, "utf8");
  const props = reg.propsUrl
    ? propControls(item, file, exportName, await fetchPropDocs(reg.propsUrl(item.name)))
    : undefined;
  const controls = controlsFor(item, source, props ? props.controls : cfg?.controls, !!props);
  problems.push(...defaultProblems(key, controls, item.fixedProps));
  const duration =
    item.durationInFrames ??
    cfg?.durationInFrames ??
    props?.durationInFrames ??
    (await registryDuration(item)) ??
    90;
  return { item, key, file, exportName, description, controls, duration };
}

async function ingest(item: CuratedElement, problems: string[]): Promise<Ingested | null> {
  const resolved = await resolveItem(item, problems);
  return resolved ? { meta: renderMetaEntry(resolved), ...renderLoaderEntry(resolved) } : null;
}

async function generate(): Promise<void> {
  const problems: string[] = [...unknownOverrideNames()];
  const ingested: Ingested[] = [];
  for (const item of CURATED) {
    const result = await ingest(item, problems);
    if (result) ingested.push(result);
  }
  const helpers = new Set(ingested.flatMap((i) => i.helpers));
  for (const [file, text] of [
    [OUT_META, renderMeta(ingested.map((i) => i.meta))],
    [
      OUT_COMPONENTS,
      renderLoaders(
        ingested.map((i) => i.loader),
        helpers,
      ),
    ],
  ] as const) {
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, text);
    console.log(`wrote ${path.relative(ROOT, file)}`);
  }
  console.log(`${ingested.length} elements`);
  if (problems.length) {
    console.warn(`\n${problems.length} problem(s):\n  ${problems.join("\n  ")}`);
    process.exitCode = 1;
  }
}

if (!skipAdd) {
  for (const registry of Object.keys(REGISTRIES)) {
    if (only && registry !== only) continue;
    const items = CURATED.filter((i) => i.registry === registry);
    if (items.length) install(items);
  }
  moveInstalled();
  rewriteImports();
}
await generate();
