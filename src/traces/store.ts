import { existsSync } from "node:fs";
import { mkdir, readdir, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { constants, zstdCompressSync, zstdDecompressSync } from "node:zlib";
import { unzipSync } from "fflate";
import { z } from "zod";
import type { AppSettings } from "../shared/settings";
import type { TraceHeader, TraceModel } from "../shared/trace";
import { parseTrace, traceTextPart, type TraceTextPart } from "./model";
import { pruneDir } from "../engine/dir";
import { sha256Hex } from "../engine/hash";
import { atomicWrite, nullIfMissing } from "../engine/json";
import { projectPaths } from "../engine/project-paths";
import { projectRef } from "../shared/asset-ref";
import { PROJECT_DIRS } from "../shared/project-dirs";
import { parseJsonWith } from "../shared/guards";
import { safeName } from "../shared/safe-name";

const traceEntrySchema = z.object({
  name: z.string(),
  sha: z.string(),
  zstd: z.boolean(),
  size: z.number(),
});
type TraceEntry = z.infer<typeof traceEntrySchema>;

const manifestSchema = z.object({
  id: z.string(),
  nodeId: z.string(),
  label: z.string(),
  ok: z.boolean(),
  createdAt: z.number(),
  groups: z.array(z.string()),
  entries: z.array(traceEntrySchema),
}) satisfies z.ZodType<TraceHeader>;
type TraceManifest = z.infer<typeof manifestSchema>;

const BINARY = /\.(jpe?g|png|webp|gif|avif|woff2?|mp4|webm)$/i;
const MANIFEST_SUFFIX = ".json.zst";

const blobName = (e: TraceEntry) =>
  e.zstd ? `${e.sha}.zst` : `${e.sha}${path.extname(e.name).toLowerCase()}`;
const blobRel = (e: TraceEntry) => path.posix.join("blobs", e.sha.slice(0, 2), blobName(e));
const runsDir = (root: string) => path.join(root, "runs");
const manifestPath = (root: string, id: string) =>
  path.join(runsDir(root), `${id}${MANIFEST_SUFFIX}`);

const zstd = (data: Uint8Array) =>
  zstdCompressSync(data, { params: { [constants.ZSTD_c_compressionLevel]: 19 } });

async function writeManifest(root: string, manifest: TraceManifest): Promise<void> {
  await mkdir(runsDir(root), { recursive: true });
  await atomicWrite(manifestPath(root, manifest.id), zstd(Buffer.from(JSON.stringify(manifest))));
}

function parseManifest(data: Buffer): TraceManifest | null {
  let text: string;
  try {
    text = zstdDecompressSync(data).toString();
  } catch {
    return null;
  }
  return parseJsonWith(manifestSchema, text)?.data ?? null;
}

type Runs = { manifests: TraceManifest[]; broken: string[] };

async function readRuns(root: string): Promise<Runs> {
  const dir = runsDir(root);
  const runs: Runs = { manifests: [], broken: [] };
  for (const name of (await nullIfMissing(readdir(dir))) ?? []) {
    if (!name.endsWith(MANIFEST_SUFFIX)) continue;
    const file = path.join(dir, name);
    const data = await nullIfMissing(readFile(file));
    if (data === null) continue;
    const manifest = parseManifest(data);
    if (manifest) runs.manifests.push(manifest);
    else runs.broken.push(file);
  }
  runs.manifests.sort((a, b) => b.createdAt - a.createdAt);
  return runs;
}

export async function ingestTrace(
  projectDir: string,
  zipFile: string,
  meta: { nodeId: string; label: string; ok: boolean; groups: string[] },
): Promise<TraceManifest> {
  const root = projectPaths(projectDir).traces;
  const files = unzipSync(new Uint8Array(await readFile(zipFile)));
  const entries: TraceEntry[] = [];
  for (const [name, data] of Object.entries(files)) {
    const entry: TraceEntry = {
      name,
      sha: sha256Hex(data),
      zstd: !BINARY.test(name),
      size: data.length,
    };
    const file = path.join(root, blobRel(entry));
    if (!existsSync(file)) {
      await mkdir(path.dirname(file), { recursive: true });
      await writeFile(file, entry.zstd ? zstd(data) : data);
    }
    entries.push(entry);
  }
  const createdAt = Date.now();
  const manifest: TraceManifest = {
    id: `${safeName(meta.nodeId, { fallback: "node", lower: false, alphabet: "alnum" })}-${createdAt}`,
    ...meta,
    createdAt,
    entries,
  };
  await writeManifest(root, manifest);
  await rm(zipFile, { force: true });
  return manifest;
}

export async function listTraces(projectDir: string): Promise<TraceManifest[]> {
  return (await readRuns(projectPaths(projectDir).traces)).manifests;
}

export async function readTrace(projectDir: string, id: string): Promise<TraceModel> {
  if (!/^[\w-]+$/.test(id)) throw new Error("Invalid trace id");
  const root = projectPaths(projectDir).traces;
  const data = await nullIfMissing(readFile(manifestPath(root, id)));
  const manifest = data && parseManifest(data);
  if (!manifest) throw new Error("This trace can't be read.");
  const { groups, entries, ...header } = manifest;
  const parts: Record<TraceTextPart, string[]> = { trace: [], network: [] };
  const sources = new Map<string, string>();
  for (const e of entries.toSorted((x, y) => x.name.localeCompare(y.name))) {
    const kind = traceTextPart(e.name);
    if (kind) {
      const raw = await readFile(path.join(root, blobRel(e)));
      parts[kind].push((e.zstd ? zstdDecompressSync(raw) : raw).toString());
    } else if (!e.zstd) {
      sources.set(e.name, projectRef(path.posix.join(PROJECT_DIRS.traces, blobRel(e))));
    }
  }
  return {
    ...header,
    ...parseTrace({
      trace: parts.trace.join("\n"),
      network: parts.network.join("\n"),
      groups,
      frameSrc: (file) => sources.get(file),
    }),
  };
}

function keptTraces(all: TraceManifest[], policy: AppSettings["traces"]): TraceManifest[] {
  const byNode = new Map<string, TraceManifest[]>();
  for (const m of all) {
    const list = byNode.get(m.nodeId);
    if (list) list.push(m);
    else byNode.set(m.nodeId, [m]);
  }
  return [...byNode.values()].flatMap((list) =>
    list.filter((m, i) => {
      if (policy.retention === "always") return true;
      if (policy.retention === "failures") return !m.ok;
      return policy.retention === "last-n" && i < policy.keepLast;
    }),
  );
}

async function sweepBlobs(root: string, live: Set<string>): Promise<void> {
  const blobs = path.join(root, "blobs");
  for (const shard of (await nullIfMissing(readdir(blobs))) ?? [])
    await pruneDir(path.join(blobs, shard), { keep: (name) => live.has(name) });
}

export async function applyRetention(
  projectDir: string,
  policy: AppSettings["traces"],
): Promise<ReadonlySet<string>> {
  const root = projectPaths(projectDir).traces;
  const { manifests, broken } = await readRuns(root);
  const kept = keptTraces(manifests, policy);
  const keep = new Set(kept.map((m) => m.id));
  for (const m of manifests)
    if (!keep.has(m.id)) await rm(manifestPath(root, m.id), { force: true });
  for (const file of broken) await rm(file, { force: true });
  await sweepBlobs(root, new Set(kept.flatMap((m) => m.entries.map(blobName))));
  return keep;
}
