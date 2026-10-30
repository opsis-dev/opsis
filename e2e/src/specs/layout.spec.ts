import path from "node:path";
import { $, $$, browser, expect } from "@wdio/globals";
import { findById } from "../../../packages/video/src/common/lists";
import { api } from "../support/api";
import { PROJECTS } from "../support/paths";

const WAIT_MS = 30_000;

type Placed = { id: string; position: { x: number; y: number }; parentId?: string };
type Joined = { id: string; target: string; targetHandle: string };
type Project = { workflow: { nodes: Placed[]; edges: Joined[] } };

async function project() {
  return ((await api("GET", "/project")).body as Project).workflow;
}

async function positions() {
  return JSON.stringify(Object.fromEntries((await project()).nodes.map((n) => [n.id, n.position])));
}

function cardBoxes() {
  return browser.execute(() =>
    [...document.querySelectorAll<HTMLElement>(".react-flow__node-opsis")].map((el) => {
      const box = el.getBoundingClientRect();
      return { id: el.dataset.id ?? "", x: box.x, y: box.y, w: box.width, h: box.height };
    }),
  );
}

function edgePaths() {
  return browser.execute(() =>
    Object.fromEntries(
      [...document.querySelectorAll<SVGGElement>(".react-flow__edge")].map((g) => [
        g.dataset.id ?? "",
        g.querySelector("path.react-flow__edge-path")?.getAttribute("d") ?? "",
      ]),
    ),
  );
}

function verticalRuns(d: string): Array<{ x: number; top: number; bottom: number }> {
  const numbers = (d.match(/-?\d+(?:\.\d+)?/g) ?? []).map(Number);
  const points: Array<[number, number]> = [];
  for (let i = 0; i + 1 < numbers.length; i += 2) points.push([numbers[i], numbers[i + 1]]);
  const runs = [];
  for (let i = 1; i < points.length; i++) {
    const [ax, ay] = points[i - 1];
    const [bx, by] = points[i];
    if (Math.abs(ax - bx) < 0.5 && Math.abs(ay - by) > 8)
      runs.push({ x: ax, top: Math.min(ay, by), bottom: Math.max(ay, by) });
  }
  return runs;
}

describe("tidy up layout", function () {
  this.timeout(120_000);

  it("lays out a flow whose sequence takes a third clip, with no cards or edge runs on top of each other", async () => {
    const opened = await api("POST", "/open", { dir: path.join(PROJECTS, "feature-launch") });
    expect(opened.status).toBe(200);
    const { workflow } = opened.body as Project;
    await browser.waitUntil(
      async () => (await $$(".react-flow__node").length) === workflow.nodes.length,
      { timeout: WAIT_MS, timeoutMsg: "the canvas never showed every node" },
    );
    expect(workflow.edges.some((e) => /^in[3-9]$/.test(e.targetHandle))).toBe(true);
    const before = await positions();
    await $('button[aria-label="Tidy up layout"]').click();
    await browser.waitUntil(async () => (await positions()) !== before, {
      timeout: WAIT_MS,
      timeoutMsg: "tidying moved no node",
    });
    await browser.pause(500);

    const boxes = (await cardBoxes()).filter((b) => {
      const n = findById(workflow.nodes, b.id);
      return n && !n.parentId;
    });
    for (const a of boxes)
      for (const b of boxes) {
        if (a.id >= b.id) continue;
        const apart = a.x + a.w <= b.x || b.x + b.w <= a.x || a.y + a.h <= b.y || b.y + b.h <= a.y;
        expect({ cards: [a.id, b.id], apart }).toEqual({ cards: [a.id, b.id], apart: true });
      }

    const paths = await edgePaths();
    const tidied = await project();
    const targets = new Set(tidied.edges.map((e) => e.target));
    for (const target of targets) {
      const runs = tidied.edges
        .filter((e) => e.target === target)
        .flatMap((e) => verticalRuns(paths[e.id] ?? "").map((r) => ({ ...r, id: e.id })));
      for (const a of runs)
        for (const b of runs) {
          if (a.id >= b.id) continue;
          const overlap = a.top < b.bottom && b.top < a.bottom;
          if (overlap)
            expect({ edges: [a.id, b.id], gap: Math.abs(a.x - b.x) > 0.5 }).toEqual({
              edges: [a.id, b.id],
              gap: true,
            });
        }
    }

    expect((await api("POST", "/project/undo")).status).toBe(200);
    await browser.waitUntil(async () => (await positions()) === before, {
      timeout: WAIT_MS,
      timeoutMsg: "undo didn't put the nodes back",
    });
  });
});
