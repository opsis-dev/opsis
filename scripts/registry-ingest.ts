import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
const errorMessage = (error: unknown): string =>
  error instanceof Error ? error.message : String(error);
import { pascalCase } from "@opsis/video/common/format";
import { isNumber, isRecord } from "@opsis/video/common/guards";
import { z } from "zod";
import { fetchOk } from "./lib/download";
import { INGEST_META, writeElementsJson } from "./lib/element-json";
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
  resolveControls,
  splitSentences,
  type ParsedConfig,
  type PropDoc,
} from "./lib/registry-controls";
import { install, moveInstalled, rewriteImports } from "./lib/registry-install";
import { applyPatches } from "./lib/registry-patches";
import { hiddenPropProblems } from "./lib/registry-props";
import { ROOT, VIDEO_DIR } from "./lib/tool-config";
import { literalOf, scratchFile } from "./lib/ts-literal";
import {
  CURATED,
  DESCRIPTIONS,
  REGISTRIES,
  unknownOverrideNames,
  type CuratedElement,
} from "./registry-ingest.list";

const OUT_META = INGEST_META;
const OUT_COMPONENTS = path.join(VIDEO_DIR, "registry", "components.generated.ts");
const skipAdd = process.argv.includes("--no-add");
const onlyIndex = process.argv.indexOf("--only");
const only = onlyIndex === -1 ? undefined : process.argv[onlyIndex + 1];
if (only && skipAdd)
  throw new Error("--only picks the registry to install, so it does nothing with --no-add.");

const registryItemSchema = z.object({ description: z.string().nullish() });

const DEFAULT_PREVIEW_FRAMES = 120;
const DEFAULT_ELEMENT_FRAMES = 90;

type WithProblems<T> = { value: T; problems: string[] };

async function fetchDescription(item: CuratedElement): Promise<string> {
  const res = await fetchOk(REGISTRIES[item.registry].itemUrl(item.name));
  const { description } = registryItemSchema.parse(await res.json());
  return splitSentences(description ?? "")
    .map((s) => s.trim())
    .filter((s) => s && !namesLibrary(s))
    .slice(0, 2)
    .join(" ");
}

async function descriptionOf(
  item: CuratedElement,
  key: string,
): Promise<WithProblems<string | undefined>> {
  const known = DESCRIPTIONS[item.name];
  if (known) return { value: known, problems: [] };
  try {
    const fetched = await fetchDescription(item);
    if (fetched) return { value: fetched, problems: [] };
    return {
      value: undefined,
      problems: [`${key}: the registry gives no usable description; add one to DESCRIPTIONS`],
    };
  } catch (error) {
    return {
      value: undefined,
      problems: [`${key}: no description (${errorMessage(error)}); add one to DESCRIPTIONS`],
    };
  }
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
  const res = await fetchOk(url);
  return propDocsSchema.parse(await res.json()).props;
}

type PreviewDurations = { byName: Record<string, number>; fallback: number };

const frameCount = (value: unknown): number | undefined =>
  isRecord(value) && isNumber(value.durationInFrames) ? value.durationInFrames : undefined;

function parsePreviewDurations(text: string): PreviewDurations {
  const sf = scratchFile("preview", "ts", text);
  const table =
    literalOf(sf.getVariableDeclaration("PREVIEW_META")?.getInitializer(), isRecord, sf) ?? {};
  const defaults = literalOf(
    sf.getVariableDeclaration("PREVIEW_DEFAULTS")?.getInitializer(),
    isRecord,
    sf,
  );
  return {
    byName: Object.fromEntries(
      Object.entries(table).flatMap(([name, entry]) => {
        const frames = frameCount(entry);
        return frames === undefined ? [] : [[name, frames]];
      }),
    ),
    fallback: frameCount(defaults) ?? DEFAULT_PREVIEW_FRAMES,
  };
}

let previews: { url: string; durations: Promise<PreviewDurations> } | undefined;

function previewDurations(url: string): Promise<PreviewDurations> {
  if (previews?.url !== url)
    previews = {
      url,
      durations: fetchOk(url)
        .then((res) => res.text())
        .then(parsePreviewDurations),
    };
  return previews.durations;
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

function exportNameOf(
  item: CuratedElement,
  file: string,
  cfg: ParsedConfig | null,
): string | undefined {
  const exported = exportedNames(file);
  return (
    item.exportName ??
    (cfg?.componentName && exported.includes(cfg.componentName) ? cfg.componentName : undefined) ??
    exported.find((e) => e === pascalCase(item.name)) ??
    exported.find((e) => /^[A-Z]/.test(e))
  );
}

async function registryDuration(item: CuratedElement): Promise<number | undefined> {
  const url = REGISTRIES[item.registry].durationsUrl;
  if (!url) return undefined;
  const { byName, fallback } = await previewDurations(url);
  return byName[item.name] ?? fallback;
}

type Ingested = { meta: string } & ReturnType<typeof renderLoaderEntry>;

const itemName = (item: CuratedElement) => `${item.registry}/${item.name}`;

const failed = (problems: string[]): WithProblems<null> => ({ value: null, problems });

type CheckedExport = { ok: true; name: string } | { ok: false; problem: string };

function componentExport(
  item: CuratedElement,
  file: string,
  cfg: ParsedConfig | null,
): CheckedExport {
  const name = exportNameOf(item, file, cfg);
  if (!name) return { ok: false, problem: `${itemName(item)}: no component export found` };
  const sceneItem = item.sceneItem ?? "";
  if (item.kind === "sequencer" && !exportedNames(file).includes(sceneItem))
    return { ok: false, problem: `${itemName(item)}: no scene item export "${sceneItem}" found` };
  return { ok: true, name };
}

async function controlsAndDuration(
  item: CuratedElement,
  file: string,
  exportName: string,
  cfg: ParsedConfig | null,
): Promise<Pick<Resolved, "controls" | "duration"> & { problems: string[] }> {
  const { propsUrl } = REGISTRIES[item.registry];
  const docs = propsUrl ? await fetchPropDocs(propsUrl(item.name)) : [];
  const props = propsUrl ? propControls(item, file, exportName, docs) : undefined;
  const source = readFileSync(file, "utf8");
  const found = controlsFor(item, source, props ? props.controls : cfg?.controls, !!props);
  const { controls, problems } = resolveControls(item, file, found, docs);
  const duration =
    item.durationInFrames ??
    cfg?.durationInFrames ??
    props?.durationInFrames ??
    (await registryDuration(item)) ??
    DEFAULT_ELEMENT_FRAMES;
  return { controls, duration, problems: problems.map((p) => `${itemName(item)}: ${p}`) };
}

async function resolveItem(item: CuratedElement): Promise<WithProblems<Resolved | null>> {
  const file = findElementFile(item);
  if (!file) return failed([`${itemName(item)}: not installed`]);
  if (item.installOnly) return failed([]);
  const reg = REGISTRIES[item.registry];
  const key = `${reg.keyPrefix}/${item.name}`;
  const cfg = reg.propsUrl ? null : await fetchConfig(item, file);
  const description = await descriptionOf(item, key);
  const exported = componentExport(item, file, cfg);
  if (!exported.ok) return failed([...description.problems, exported.problem]);
  const resolved = await controlsAndDuration(item, file, exported.name, cfg);
  const { controls, duration } = resolved;
  const shown = new Set([...controls.map(([k]) => k), ...Object.keys(item.fixedProps ?? {})]);
  const hidden = hiddenPropProblems({
    key,
    item,
    file,
    exportName: exported.name,
    shown,
  });
  return {
    value: {
      item,
      key,
      file,
      exportName: exported.name,
      description: description.value,
      controls,
      duration,
    },
    problems: [
      ...description.problems,
      ...resolved.problems,
      ...hidden,
      ...defaultProblems(key, controls, item.fixedProps),
    ],
  };
}

async function ingest(item: CuratedElement): Promise<WithProblems<Ingested | null>> {
  const { value, problems } = await resolveItem(item);
  return {
    value: value ? { meta: renderMetaEntry(value), ...renderLoaderEntry(value) } : null,
    problems,
  };
}

async function generate(): Promise<void> {
  const problems: string[] = [...unknownOverrideNames()];
  const ingested: Ingested[] = [];
  for (const item of CURATED) {
    const result = await ingest(item);
    problems.push(...result.problems);
    if (result.value) ingested.push(result.value);
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
  await writeElementsJson();
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
const patchProblems = applyPatches();
await generate();
if (patchProblems.length) {
  console.warn(`\n${patchProblems.length} patch problem(s):\n  ${patchProblems.join("\n  ")}`);
  process.exitCode = 1;
}
