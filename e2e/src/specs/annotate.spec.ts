import fs from "node:fs";
import path from "node:path";
import { $, browser, expect } from "@wdio/globals";
import { findById } from "../../../packages/video/src/common/lists";
import { api } from "../support/api";
import { PROJECTS, ROOT, RUN_DIR } from "../support/paths";

const WAIT_MS = 30_000;

type Box = { left: number; top: number; width: number; height: number };
type Mark = { kind: string; x: number; y: number; x2: number; y2: number };
type Project = { workflow: { nodes: Array<{ id: string; params?: { marks?: Mark[] } }> } };

const marksOf = async (): Promise<Mark[]> => {
  const project = (await api("GET", "/project")).body as Project;
  return findById(project.workflow.nodes, "notes")?.params?.marks ?? [];
};

describe("annotate", function () {
  this.timeout(120_000);

  it("draws a box on the picture in the Inspector and saves it as one mark", async () => {
    const dir = path.join(PROJECTS, "changelog");
    fs.copyFileSync(
      path.join(ROOT, "public", "backdrops", "aurora.jpg"),
      path.join(dir, "assets", "shot.jpg"),
    );
    expect((await api("POST", "/open", { dir })).status).toBe(200);
    const added = [
      {
        id: "shot",
        type: "media.import",
        position: { x: 0, y: 0 },
        params: { path: "assets/shot.jpg" },
      },
      { id: "notes", type: "image.annotate", position: { x: 320, y: 0 }, params: { marks: [] } },
    ];
    for (const node of added) expect((await api("POST", "/project/nodes", node)).status).toBe(200);
    const edge = { source: "shot", sourceHandle: "media", target: "notes", targetHandle: "image" };
    expect((await api("POST", "/project/edges", edge)).status).toBe(200);

    await $('[data-id="notes"]').waitForExist({ timeout: WAIT_MS });
    await browser.execute(() => {
      const card = document.querySelector<HTMLElement>('[data-id="notes"]');
      card?.click();
    });
    const boxTool = $('button[aria-label="Box"]');
    await boxTool.waitForExist({ timeout: WAIT_MS });
    await boxTool.click();
    await browser.waitUntil(
      () => browser.execute(() => !!document.querySelector(".konvajs-content canvas")),
      { timeout: WAIT_MS },
    );
    const stage = await browser.execute((): Box | null => {
      const canvas = document.querySelector(".konvajs-content");
      if (!canvas) return null;
      const b = canvas.getBoundingClientRect();
      return { left: b.left, top: b.top, width: b.width, height: b.height };
    });
    if (!stage) throw new Error("no marks canvas");
    const at = (fx: number, fy: number) => ({
      x: Math.round(stage.left + stage.width * fx),
      y: Math.round(stage.top + stage.height * fy),
    });
    const [from, to] = [at(0.25, 0.25), at(0.6, 0.7)];
    await browser.performActions([
      {
        type: "pointer",
        id: "mouse",
        parameters: { pointerType: "mouse" },
        actions: [
          { type: "pointerMove", origin: "viewport", x: from.x, y: from.y, duration: 0 },
          { type: "pointerDown", button: 0 },
          { type: "pointerMove", origin: "viewport", x: to.x, y: to.y, duration: 300 },
          { type: "pointerUp", button: 0 },
        ],
      },
    ]);
    await browser.releaseActions();
    await browser.waitUntil(async () => (await marksOf()).length > 0, { timeout: WAIT_MS });
    const marks = await marksOf();
    expect(marks).toHaveLength(1);
    const [box] = marks;
    expect(box?.kind).toBe("rectangle");
    expect(Math.abs((box?.x ?? 0) - 0.25)).toBeLessThan(0.02);
    expect(Math.abs((box?.y2 ?? 0) - 0.7)).toBeLessThan(0.02);
    await browser.saveScreenshot(path.join(RUN_DIR, "annotate.png"));
  });
});
