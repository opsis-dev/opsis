import { createHash } from "node:crypto";
import { existsSync } from "node:fs";
import { mkdir, readdir, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { constants, zstdCompressSync, zstdDecompressSync } from "node:zlib";
import { unzipSync } from "fflate";
import type { AppSettings } from "../shared/settings";
import type { TraceModel } from "../shared/trace";
import { parseTrace } from "./model";

export type TraceEntry = { name: string; sha: string; zstd: boolean; size: number };

export type TraceManifest = {
  id: string;
  nodeId: string;
  label: string;
  ok: boolean;
  createdAt: number;
  groups: string[];
  entries: TraceEntry[];
};

const BINARY = /\.(jpe?g|png|webp|gif|avif|woff2?|mp4|webm)$/i;

export const traceRoot = (projectDir: string) => path.join(projectDir, ".demoflow", "traces");
const blobName = (e: Pick<TraceEntry, "sha" | "zstd" | "name">) =>
  e.zstd ? `${e.sha}.zst` : `${e.sha}${path.extname(e.name).toLowerCase()}`;
const blobRel = (e: Pick<TraceEntry, "sha" | "zstd" | "name">) =>
  path.posix.join("blobs", e.sha.slice(0, 2), blobName(e));
const manifestPath = (root: string, id: string) => path.join(root, "runs", `${id}.json.zst`);

const zstd = (data: Uint8Array) =>
  zstdCompressSync(data, { params: { [constants.ZSTD_c_compressionLevel]: 19 } });

export async function ingestTrace(
  projectDir: string,
  zipFile: string,
  meta: { nodeId: string; label: string; ok: boolean; groups: string[] },
): Promise<TraceManifest> {
  const root = traceRoot(projectDir);
  const files = unzipSync(new Uint8Array(await readFile(zipFile)));
  const entries: TraceEntry[] = [];
  for (const [name, data] of Object.entries(files)) {
    const sha = createHash("sha256").update(data).digest("hex");
    const entry: TraceEntry = { name, sha, zstd: !BINARY.test(name), size: data.length };
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
  await mkdir(path.join(root, "runs"), { recursive: true });
  await writeFile(manifestPath(root, manifest.id), zstd(Buffer.from(JSON.stringify(manifest))));
  await rm(zipFile, { force: true });
  return manifest;
}

export async function listTraces(projectDir: string): Promise<TraceManifest[]> {
  const dir = path.join(traceRoot(projectDir), "runs");
  if (!existsSync(dir)) return [];
  const out: TraceManifest[] = [];
  for (const f of await readdir(dir)) {
    if (!f.endsWith(".json.zst")) continue;
    out.push(
      JSON.parse(zstdDecompressSync(await readFile(path.join(dir, f))).toString()) as TraceManifest,
    );
  }
  return out.sort((a, b) => b.createdAt - a.createdAt);
}

const TEXT_PART = /^(?:\d+-)?trace\.(trace|network)$/;

async function readManifest(root: string, id: string): Promise<TraceManifest> {
  return JSON.parse(
    zstdDecompressSync(await readFile(manifestPath(root, id))).toString(),
  ) as TraceManifest;
}

export async function readTrace(projectDir: string, id: string): Promise<TraceModel> {
  if (!/^[\w-]+$/.test(id)) throw new Error("Invalid trace id");
  const root = traceRoot(projectDir);
  const manifest = await readManifest(root, id);
  const parts = { trace: [] as string[], network: [] as string[] };
  const sources = new Map<string, string>();
  for (const e of [...manifest.entries].sort((x, y) => x.name.localeCompare(y.name))) {
    const kind = TEXT_PART.exec(e.name)?.[1] as "trace" | "network" | undefined;
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
      groups: manifest.groups ?? [],
      frameSrc: (file) => sources.get(file),
    }),
  };
}

export async function applyRetention(
  projectDir: string,
  policy: AppSettings["traces"],
): Promise<void> {
  const root = traceRoot(projectDir);
  const all = await listTraces(projectDir);
  const keep = new Set<string>();
  const byNode = new Map<string, TraceManifest[]>();
  for (const m of all) byNode.set(m.nodeId, [...(byNode.get(m.nodeId) ?? []), m]);
  for (const list of byNode.values()) {
    for (const [i, m] of list.entries()) {
      if (policy.retention === "always") keep.add(m.id);
      else if (policy.retention === "failures" && !m.ok) keep.add(m.id);
      else if (policy.retention === "last-n" && i < policy.keepLast) keep.add(m.id);
    }
  }
  for (const m of all) if (!keep.has(m.id)) await rm(manifestPath(root, m.id), { force: true });
  const live = new Set(
    all.filter((m) => keep.has(m.id)).flatMap((m) => m.entries.map((e) => blobName(e))),
  );
  const blobs = path.join(root, "blobs");
  if (!existsSync(blobs)) return;
  for (const shard of await readdir(blobs)) {
    for (const name of await readdir(path.join(blobs, shard))) {
      if (!live.has(name)) await rm(path.join(blobs, shard, name), { force: true });
    }
  }
}
