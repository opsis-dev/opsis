import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import { eventsIn } from "./api";
import {
  argsOf,
  CLOCK_TICKS,
  cpuByTag,
  parseSmapsRollup,
  parseStat,
  type ProcSample,
  type Snapshot,
} from "./procfs";

const ticks = fc.nat({ max: 1_000_000 });

const comm = fc.string({ maxLength: 15 }).filter((text) => !text.includes("\n"));

function statLine(pid: number, name: string, ppid: number, times: number[]): string {
  const middle = ["S", ppid, pid, pid, 0, -1, 4194560, 100, 0, 0, 0];
  return `${pid} (${name}) ${[...middle, ...times, 20, 0, 1, 0, 12345].join(" ")}\n`;
}

describe("reading /proc", () => {
  it("reads a stat line whatever the program's name holds", () => {
    fc.assert(
      fc.property(
        fc.nat(),
        comm,
        fc.nat(),
        fc.tuple(ticks, ticks, ticks, ticks),
        (pid, name, ppid, [u, s, cu, cs]) => {
          expect(parseStat(statLine(pid, name, ppid, [u, s, cu, cs]))).toEqual({
            pid,
            comm: name,
            ppid,
            ownTicks: u + s,
            reapedTicks: cu + cs,
          });
        },
      ),
    );
  });

  it("reads Rss and Pss from a rollup among its other lines", () => {
    fc.assert(
      fc.property(fc.nat(), fc.nat(), fc.nat(), (rss, pss, other) => {
        const text = `55d0-7ffd ---p 00000000 00:00 0 [rollup]\nRss: ${rss} kB\nPss: ${pss} kB\nPss_Anon: ${other} kB\nSwap: ${other} kB\n`;
        expect(parseSmapsRollup(text)).toEqual({ rssKib: rss, pssKib: pss });
      }),
    );
  });
});

describe("splitting a command line", () => {
  it("gives back every argument of a NUL-separated command line", () => {
    const arg = fc.string({ minLength: 1 }).filter((text) => !text.includes("\0"));
    fc.assert(
      fc.property(fc.array(arg, { minLength: 2, maxLength: 8 }), (args) => {
        expect(argsOf(`${args.join("\0")}\0`)).toEqual(args);
      }),
    );
  });
});

const TAGS = ["app", "ffmpeg", "browser", "shell"];

function tree(size: number) {
  return fc.array(fc.tuple(fc.nat(), fc.constantFrom(...TAGS), ticks, ticks), {
    minLength: size,
    maxLength: size,
  });
}

function processes(rows: [number, string, number, number][], offset: number): ProcSample[] {
  return rows.map(([parent, tag, own, reaped], index) => ({
    pid: index + 2 + offset,
    ppid: index === 0 ? 1 : (parent % (index + 1)) + 1 + offset,
    comm: tag,
    tag,
    program: tag,
    ownTicks: own,
    reapedTicks: reaped,
    rssKib: 0,
    pssKib: 0,
  }));
}

function root(own: number, reaped: number): ProcSample {
  return {
    pid: 1,
    ppid: 0,
    comm: "init",
    tag: "system",
    program: "tini",
    ownTicks: own,
    reapedTicks: reaped,
    rssKib: 0,
    pssKib: 0,
  };
}

function total(snapshot: Snapshot): number {
  return snapshot.processes.reduce((sum, p) => sum + p.ownTicks + p.reapedTicks, 0);
}

describe("CPU by process between two samples", () => {
  it("adds up to the change in every live and reaped counter, whoever reaped the processes that exited", () => {
    fc.assert(
      fc.property(
        tree(6),
        tree(6),
        fc.array(fc.boolean(), { minLength: 6, maxLength: 6 }),
        fc.tuple(ticks, ticks, ticks, ticks),
        (earlier, later, keep, [rootOwnA, rootReapedA, rootOwnB, rootReapedB]) => {
          const before: Snapshot = {
            at: 0,
            processes: [root(rootOwnA, rootReapedA), ...processes(earlier, 0)],
          };
          const survivors = before.processes.slice(1).filter((_, index) => keep[index]);
          const after: Snapshot = {
            at: 1000,
            processes: [root(rootOwnB, rootReapedB), ...survivors, ...processes(later, 100)],
          };
          const attributed = [...cpuByTag(before, after).values()].reduce(
            (sum, value) => sum + value,
            0,
          );
          expect(attributed * CLOCK_TICKS).toBeCloseTo(total(after) - total(before), 6);
        },
      ),
    );
  });
});

describe("the run event stream", () => {
  it("gives every event whole however the stream is cut into chunks", () => {
    const event = fc.record({
      type: fc.constantFrom("node:start", "node:done", "run:done"),
      runId: fc.string(),
      nodeId: fc.string(),
    });
    fc.assert(
      fc.property(
        fc.array(event, { maxLength: 8 }),
        fc.array(fc.nat(), { maxLength: 6 }),
        (events, cuts) => {
          const stream = `: following run events\n\n${events.map((e) => `data: ${JSON.stringify(e)}\n\n`).join("")}`;
          const points = [...new Set(cuts.map((cut) => cut % (stream.length + 1)))].sort(
            (a, b) => a - b,
          );
          let pending = "";
          const seen: unknown[] = [];
          for (const chunk of splitAt(stream, points)) {
            const parsed = eventsIn(pending + chunk);
            pending = parsed.rest;
            seen.push(...parsed.events);
          }
          expect(seen).toEqual(events);
        },
      ),
    );
  });
});

function splitAt(text: string, points: number[]): string[] {
  const ends = [...points, text.length];
  return ends.map((end, index) => text.slice(index === 0 ? 0 : ends[index - 1], end));
}
