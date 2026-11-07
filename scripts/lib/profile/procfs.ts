import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";

export const CLOCK_TICKS = 100;

export type ProcStat = {
  pid: number;
  comm: string;
  ppid: number;
  ownTicks: number;
  reapedTicks: number;
};

export type ProcMemory = { rssKib: number; pssKib: number };

export type ProcSample = ProcStat & ProcMemory & { tag: string; program: string };

export type Snapshot = { at: number; processes: ProcSample[] };

const STAT_PPID = 1;
const STAT_UTIME = 11;
const STAT_STIME = 12;
const STAT_CUTIME = 13;
const STAT_CSTIME = 14;

export function parseStat(text: string): ProcStat {
  const open = text.indexOf("(");
  const close = text.lastIndexOf(")");
  if (open < 0 || close < open) throw new Error(`not a stat line: ${text.slice(0, 80)}`);
  const fields = text
    .slice(close + 2)
    .trim()
    .split(" ");
  function field(index: number): number {
    return Number(fields[index]);
  }
  return {
    pid: Number(text.slice(0, open).trim()),
    comm: text.slice(open + 1, close),
    ppid: field(STAT_PPID),
    ownTicks: field(STAT_UTIME) + field(STAT_STIME),
    reapedTicks: field(STAT_CUTIME) + field(STAT_CSTIME),
  };
}

export function parseSmapsRollup(text: string): ProcMemory {
  function kib(name: string): number {
    return Number(text.match(new RegExp(`^${name}:\\s+(\\d+) kB`, "m"))?.[1] ?? 0);
  }
  return { rssKib: kib("Rss"), pssKib: kib("Pss") };
}

const CHROME_TYPES = ["renderer", "gpu-process", "utility", "zygote"];

function chromeTag(args: string[]): string {
  const type = args.find((arg) => arg.startsWith("--type="))?.slice("--type=".length);
  if (!type) return "browser";
  return CHROME_TYPES.includes(type) ? `browser ${type}` : "browser other";
}

function tagOf(args: string[]): string {
  const program = path.basename(args[0] ?? "");
  if (program === "opsis") return "app";
  if (program.startsWith("opsis-ffmpeg") || program === "ffmpeg") return "ffmpeg";
  if (program === "chrome-headless-shell") return chromeTag(args);
  if (program === "exe" && args.some((arg) => arg.startsWith("--type="))) return chromeTag(args);
  if (["bash", "sh", "zsh", "dash"].includes(program)) return "shell";
  if (["Xvfb", "dbus-daemon", "gnome-keyring-daemon", "tini", "docker-init"].includes(program))
    return "system";
  if (["samply", "perf", "heaptrack"].includes(program) || program.startsWith("heaptrack"))
    return "profiler";
  if (program === "bun") return "harness";
  return "other";
}

function readOr(file: string, fallback: string): string {
  try {
    return readFileSync(file, "utf8");
  } catch {
    return fallback;
  }
}

export function argsOf(cmdline: string): string[] {
  const args = cmdline.split("\0").filter(Boolean);
  return args.length === 1 && args[0].includes(" --") ? args[0].split(" ").filter(Boolean) : args;
}

function sampleOf(pid: number, knownTags: Map<number, string>): ProcSample | null {
  const dir = path.join("/proc", String(pid));
  const stat = readOr(path.join(dir, "stat"), "");
  if (!stat) return null;
  const parsed = parseStat(stat);
  const args = argsOf(readOr(path.join(dir, "cmdline"), ""));
  const tag = args.length > 0 ? tagOf(args) : (knownTags.get(pid) ?? tagOf([parsed.comm]));
  const type = args.find((arg) => arg.startsWith("--type="));
  return {
    ...parsed,
    ...parseSmapsRollup(readOr(path.join(dir, "smaps_rollup"), "")),
    tag,
    program:
      args.length > 0 ? [path.basename(args[0]), type].filter(Boolean).join(" ") : parsed.comm,
  };
}

export function snapshot(previous?: Snapshot): Snapshot {
  const knownTags = new Map((previous?.processes ?? []).map((p) => [p.pid, p.tag]));
  const pids = readdirSync("/proc")
    .filter((name) => /^\d+$/.test(name))
    .map(Number);
  const processes = pids.map((pid) => sampleOf(pid, knownTags)).filter((sample) => sample !== null);
  return { at: Date.now(), processes };
}

function totalTicks(sample: ProcStat): number {
  return sample.ownTicks + sample.reapedTicks;
}

const EXITED = "exited between samples";

export function cpuByTag(before: Snapshot, after: Snapshot): Map<string, number> {
  const earlier = new Map(before.processes.map((p) => [p.pid, p]));
  const later = new Set(after.processes.map((p) => p.pid));
  const ticks = new Map<string, number>();
  function add(tag: string, amount: number) {
    if (amount !== 0) ticks.set(tag, (ticks.get(tag) ?? 0) + amount);
  }
  let reaped = 0;
  for (const now of after.processes) {
    const then = earlier.get(now.pid);
    add(now.tag, now.ownTicks - (then?.ownTicks ?? 0));
    reaped += now.reapedTicks - (then?.reapedTicks ?? 0);
  }
  const counted = before.processes
    .filter((gone) => !later.has(gone.pid))
    .reduce((sum, gone) => sum + totalTicks(gone), 0);
  add(EXITED, reaped - counted);
  return new Map([...ticks].map(([tag, amount]) => [tag, amount / CLOCK_TICKS]));
}

export function memoryByTag(sample: Snapshot): Map<string, ProcMemory> {
  const memory = new Map<string, ProcMemory>();
  for (const process of sample.processes) {
    const sum = memory.get(process.tag) ?? { rssKib: 0, pssKib: 0 };
    memory.set(process.tag, {
      rssKib: sum.rssKib + process.rssKib,
      pssKib: sum.pssKib + process.pssKib,
    });
  }
  return memory;
}
