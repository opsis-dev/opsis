import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import type { ProcMemory } from "./procfs";

export type Sample = {
  from: number;
  at: number;
  cpu: Record<string, number>;
  memory: Record<string, ProcMemory>;
};

type NodeRun = {
  id: string;
  type: string;
  startedAt: number;
  endedAt: number;
  ok: boolean;
  error: string | null;
  ran: string[];
  cached: string[];
};

export type NodeCost = {
  id: string;
  type: string;
  ok: boolean;
  error: string | null;
  wallMs: number;
  cpuSeconds: Record<string, number>;
  peakPssMib: Record<string, number>;
  totalCpuSeconds: number;
  peakTotalPssMib: number;
};

const KIB_PER_MIB = 1024;

export function overlapShare(sample: Sample, start: number, end: number): number {
  const overlap = Math.min(sample.at, end) - Math.max(sample.from, start);
  if (overlap <= 0) return 0;
  return sample.at === sample.from ? 1 : overlap / (sample.at - sample.from);
}

function nodeCosts(runs: NodeRun[], samples: Sample[]): NodeCost[] {
  return runs.map((run) => {
    const cpuSeconds: Record<string, number> = {};
    const peakPssMib: Record<string, number> = {};
    let peakTotalPssMib = 0;
    for (const sample of samples) {
      const share = overlapShare(sample, run.startedAt, run.endedAt);
      if (share === 0) continue;
      for (const [tag, seconds] of Object.entries(sample.cpu))
        cpuSeconds[tag] = (cpuSeconds[tag] ?? 0) + seconds * share;
      let total = 0;
      for (const [tag, memory] of Object.entries(sample.memory)) {
        const mib = memory.pssKib / KIB_PER_MIB;
        peakPssMib[tag] = Math.max(peakPssMib[tag] ?? 0, mib);
        total += mib;
      }
      peakTotalPssMib = Math.max(peakTotalPssMib, total);
    }
    return {
      id: run.id,
      type: run.type,
      ok: run.ok,
      error: run.error,
      wallMs: run.endedAt - run.startedAt,
      cpuSeconds,
      peakPssMib,
      totalCpuSeconds: Object.values(cpuSeconds).reduce((sum, value) => sum + value, 0),
      peakTotalPssMib,
    };
  });
}

function csvCell(value: unknown): string {
  const text = String(value ?? "");
  return /[",\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text;
}

function csv(rows: unknown[][]): string {
  return `${rows.map((row) => row.map(csvCell).join(",")).join("\n")}\n`;
}

function tagsOf(records: Record<string, number>[]): string[] {
  return [...new Set(records.flatMap((record) => Object.keys(record)))].sort();
}

const PALETTE = [
  "#4e79a7",
  "#f28e2b",
  "#e15759",
  "#76b7b2",
  "#59a14f",
  "#edc948",
  "#b07aa1",
  "#ff9da7",
  "#9c755f",
  "#bab0ac",
  "#86bcb6",
  "#d37295",
];

function stackedArea(
  samples: Sample[],
  tags: string[],
  value: (s: Sample, tag: string) => number,
  unit: string,
): string {
  const width = 1200;
  const height = 260;
  if (samples.length === 0) return "";
  const start = samples[0].from;
  const span = Math.max(1, samples.at(-1)!.at - start);
  const totals = samples.map((s) => tags.reduce((sum, tag) => sum + value(s, tag), 0));
  const top = Math.max(1e-9, ...totals);
  function x(s: Sample) {
    return ((s.at - start) / span) * width;
  }
  function y(v: number) {
    return height - (v / top) * height;
  }
  const base = samples.map(() => 0);
  const layers = tags.map((tag, index) => {
    const upper = samples.map((s, i) => base[i] + value(s, tag));
    const forward = samples.map((s, i) => `${x(s).toFixed(1)},${y(upper[i]).toFixed(1)}`);
    const back = samples.map((s, i) => `${x(s).toFixed(1)},${y(base[i]).toFixed(1)}`).toReversed();
    upper.forEach((v, i) => {
      base[i] = v;
    });
    return `<polygon fill="${PALETTE[index % PALETTE.length]}" points="${[...forward, ...back].join(" ")}"><title>${tag}</title></polygon>`;
  });
  const legend = tags
    .map(
      (tag, index) =>
        `<span class="key"><i style="background:${PALETTE[index % PALETTE.length]}"></i>${tag}</span>`,
    )
    .join("");
  return `<div class="legend">${legend}</div><svg viewBox="0 0 ${width} ${height}" preserveAspectRatio="none">${layers.join("")}</svg><p class="axis">peak ${top.toFixed(1)} ${unit}, ${(span / 1000).toFixed(0)} s</p>`;
}

function html(costs: NodeCost[], samples: Sample[], title: string): string {
  const cpuTags = tagsOf(samples.map((s) => s.cpu));
  const memoryTags = tagsOf(
    samples.map((s) => Object.fromEntries(Object.keys(s.memory).map((k) => [k, 0]))),
  );
  function seconds(s: Sample) {
    return Math.max(0.001, (s.at - s.from) / 1000);
  }
  const cpu = stackedArea(samples, cpuTags, (s, tag) => (s.cpu[tag] ?? 0) / seconds(s), "cores");
  const memory = stackedArea(
    samples,
    memoryTags,
    (s, tag) => (s.memory[tag]?.pssKib ?? 0) / KIB_PER_MIB / 1024,
    "GiB",
  );
  const rows = [...costs]
    .sort((a, b) => b.wallMs - a.wallMs)
    .map(
      (c) =>
        `<tr class="${c.ok ? "" : "failed"}"><td>${c.type}</td><td>${c.id}</td><td>${(c.wallMs / 1000).toFixed(1)}</td><td>${c.totalCpuSeconds.toFixed(1)}</td><td>${c.peakTotalPssMib.toFixed(0)}</td><td>${Object.entries(
          c.cpuSeconds,
        )
          .filter(([, v]) => v >= 0.05)
          .sort((a, b) => b[1] - a[1])
          .map(([tag, v]) => `${tag} ${v.toFixed(1)}`)
          .join(", ")}</td><td>${c.error ?? ""}</td></tr>`,
    )
    .join("");
  return `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>${title}</title>
<style>
:root { color-scheme: light dark; --bg: #fff; --fg: #111; --muted: #666; --line: #ddd; --bad: #fde2e2; }
@media (prefers-color-scheme: dark) { :root { --bg: #111; --fg: #eee; --muted: #999; --line: #333; --bad: #4a1d1d; } }
body { background: var(--bg); color: var(--fg); font: 14px system-ui, sans-serif; margin: 0 auto; padding: 16px; max-width: 1240px; }
svg { width: 100%; height: 260px; border: 1px solid var(--line); }
.legend { display: flex; flex-wrap: wrap; gap: 12px; margin: 8px 0; }
.key i { display: inline-block; width: 10px; height: 10px; margin-right: 4px; }
.axis { color: var(--muted); margin: 4px 0 24px; }
table { border-collapse: collapse; width: 100%; font-variant-numeric: tabular-nums; }
td, th { border-bottom: 1px solid var(--line); padding: 4px 6px; text-align: left; vertical-align: top; }
tr.failed { background: var(--bad); }
.wrap { overflow-x: auto; }
</style></head><body>
<h1>${title}</h1>
<h2>CPU by process</h2>${cpu}
<h2>Memory (PSS) by process</h2>${memory}
<h2>Nodes, longest first</h2>
<div class="wrap"><table><thead><tr><th>Node</th><th>Id</th><th>Wall s</th><th>CPU s</th><th>Peak PSS MiB</th><th>CPU by process</th><th>Error</th></tr></thead><tbody>${rows}</tbody></table></div>
</body></html>`;
}

function readSamples(file: string): Sample[] {
  return readFileSync(file, "utf8")
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line));
}

export function writeReport(out: string, title: string): NodeCost[] {
  const runs: NodeRun[] = JSON.parse(readFileSync(path.join(out, "runs.json"), "utf8"));
  const samples = readSamples(path.join(out, "samples.jsonl"));
  const costs = nodeCosts(runs, samples);
  const cpuTags = tagsOf(costs.map((c) => c.cpuSeconds));
  const memoryTags = tagsOf(costs.map((c) => c.peakPssMib));
  writeFileSync(
    path.join(out, "nodes.csv"),
    csv([
      [
        "type",
        "id",
        "ok",
        "wall_s",
        "cpu_s",
        "peak_pss_mib",
        ...cpuTags.map((t) => `cpu_s ${t}`),
        ...memoryTags.map((t) => `peak_pss_mib ${t}`),
        "error",
      ],
      ...costs.map((c) => [
        c.type,
        c.id,
        c.ok,
        (c.wallMs / 1000).toFixed(3),
        c.totalCpuSeconds.toFixed(3),
        c.peakTotalPssMib.toFixed(1),
        ...cpuTags.map((t) => (c.cpuSeconds[t] ?? 0).toFixed(3)),
        ...memoryTags.map((t) => (c.peakPssMib[t] ?? 0).toFixed(1)),
        c.error ?? "",
      ]),
    ]),
  );
  const sampleTags = tagsOf(samples.map((s) => s.cpu));
  writeFileSync(
    path.join(out, "samples.csv"),
    csv([
      ["at", ...sampleTags.map((t) => `cpu_s ${t}`), ...sampleTags.map((t) => `pss_mib ${t}`)],
      ...samples.map((s) => [
        new Date(s.at).toISOString(),
        ...sampleTags.map((t) => (s.cpu[t] ?? 0).toFixed(3)),
        ...sampleTags.map((t) => ((s.memory[t]?.pssKib ?? 0) / KIB_PER_MIB).toFixed(1)),
      ]),
    ]),
  );
  writeFileSync(path.join(out, "report.html"), html(costs, samples, title));
  return costs;
}
