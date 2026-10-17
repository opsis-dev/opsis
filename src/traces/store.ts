import { existsSync } from "node:fs";
import { mkdir, readdir, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { constants, zstdCompressSync, zstdDecompressSync } from "node:zlib";
import { unzipSync } from "fflate";
import type { AppSettings } from "../shared/settings";
import type { TraceModel } from "../shared/trace";
import { parseTrace } from "./model";
import { sha256Hex } from "../engine/hash";

type TraceEntry = { name: string; sha: string; zstd: boolean; size: number };

type TraceManifest = {
  id: string;
  nodeId: string;
  label: string;
  ok: boolean;
  createdAt: number;
  groups: string[];
  entries: TraceEntry[];
};

type TextPart = "trace" | "network";

const BINARY = /\.(jpe?g|png|webp|gif|avif|woff2?|mp4|webm)$/i;
const MANIFEST_SUFFIX = ".json.zst";
const TEXT_PART = /^(?:\d+-)?trace\.(trace|network)$/;

export const traceRoot = (projectDir: string) => path.join(projectDir, ".demoflow", "traces");
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
  await writeFile(manifestPath(root, manifest.id), zstd(Buffer.from(JSON.stringify(manifest))));
}

async function readManifestFile(file: string): Promise<TraceManifest> {
  return JSON.parse(zstdDecompressSync(await readFile(file)).toString()) as TraceManifest;
}

function textPart(name: string): TextPart | undefined {
  const kind = TEXT_PART.exec(name)?.[1];
  return kind === "trace" || kind === "network" ? kind : undefined;
}

export async function ingestTrace(
  projectDir: string,
  zipFile: string,
  meta: { nodeId: string; label: string; ok: boolean; groups: string[] },
): Promise<TraceManifest> {
  const root = traceRoot(projectDir);
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
    id: `${meta.nodeId}-${createdAt}`,
    ...meta,
    createdAt,
    entries,
  };
  await writeManifest(root, manifest);
  await rm(zipFile, { force: true });
  return manifest;
}

export async function listTraces(projectDir: string): Promise<TraceManifest[]> {
  const dir = runsDir(traceRoot(projectDir));
  if (!existsSync(dir)) return [];
  const out: TraceManifest[] = [];
  for (const f of await readdir(dir))
    if (f.endsWith(MANIFEST_SUFFIX)) out.push(await readManifestFile(path.join(dir, f)));
  return out.sort((a, b) => b.createdAt - a.createdAt);
}

export async function readTrace(projectDir: string, id: string): Promise<TraceModel> {
  if (!/^[\w-]+$/.test(id)) throw new Error("Invalid trace id");
  const root = traceRoot(projectDir);
  const manifest = await readManifestFile(manifestPath(root, id));
  const parts: Record<TextPart, string[]> = { trace: [], network: [] };
  const sources = new Map<string, string>();
  for (const e of manifest.entries.toSorted((x, y) => x.name.localeCompare(y.name))) {
    const kind = textPart(e.name);
    if (kind) {
      const raw = await readFile(path.join(root, blobRel(e)));
      parts[kind].push((e.zstd ? zstdDecompressSync(raw) : raw).toString());
    } else if (!e.zstd) {
      sources.set(e.name, `project:${path.posix.join(".demoflow", "traces", blobRel(e))}`);
    }
  }
  return {
    id: manifest.id,
    nodeId: manifest.nodeId,
    label: manifest.label,
    ok: manifest.ok,
    createdAt: manifest.createdAt,
    ...parseTrace({
      trace: parts.trace.join("\n"),
      network: parts.network.join("\n"),
      groups: manifest.groups,
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
  if (!existsSync(blobs)) return;
  for (const shard of await readdir(blobs))
    for (const name of await readdir(path.join(blobs, shard)))
      if (!live.has(name)) await rm(path.join(blobs, shard, name), { force: true });
}

export async function applyRetention(
  projectDir: string,
  policy: AppSettings["traces"],
): Promise<ReadonlySet<string>> {
  const root = traceRoot(projectDir);
  const all = await listTraces(projectDir);
  const kept = keptTraces(all, policy);
  const keep = new Set(kept.map((m) => m.id));
  for (const m of all) if (!keep.has(m.id)) await rm(manifestPath(root, m.id), { force: true });
  await sweepBlobs(root, new Set(kept.flatMap((m) => m.entries.map(blobName))));
  return keep;
}
