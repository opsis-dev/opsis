import { copyFileSync, existsSync, readFileSync, rmSync, statSync } from "node:fs";
import path from "node:path";
import { $, browser, expect } from "@wdio/globals";
import { api, runEvents } from "../support/api";
import { engineReady } from "../support/engine";
import { PROJECTS, ROOT, RUN_DIR } from "../support/paths";

const RUN_MS = 600_000;
const WINDOW_MS = 60_000;

type Artifact = { path: string; label?: string };

function runUntilDone(
  body: unknown,
): Promise<{ done: Record<string, unknown>; artifacts: Artifact[] }> {
  const artifacts: Artifact[] = [];
  let done: Record<string, unknown> = {};
  const finished = runEvents((event) => {
    if (event.type === "node:done") {
      const result = event.result as { artifacts?: Artifact[] } | undefined;
      artifacts.push(...(result?.artifacts ?? []));
    }
    if (event.type !== "run:done") return false;
    done = event;
    return true;
  });
  return api("POST", "/project/run", body).then(async (answer) => {
    expect(answer.status).toBe(200);
    await finished;
    return { done, artifacts };
  });
}

async function typeIntoTheTerminalWindow(main: string, text: string): Promise<void> {
  await browser.waitUntil(async () => (await browser.getWindowHandles()).length > 1, {
    timeout: WINDOW_MS,
  });
  const window = (await browser.getWindowHandles()).find((h) => h !== main);
  if (!window) throw new Error("no terminal window");
  await browser.switchToWindow(window);
  await $('[aria-label="Terminal"]').waitForExist({ timeout: WINDOW_MS });
  await browser.waitUntil(
    () => browser.execute(() => document.body.textContent?.includes("opsis") ?? false),
    { timeout: WINDOW_MS },
  );
  await browser.keys([...text, "Enter"]);
  await browser.waitUntil(
    () =>
      browser.execute(
        (typed: string) => (document.body.textContent?.split(typed).length ?? 0) > 2,
        text.replace("echo ", ""),
      ),
    { timeout: WINDOW_MS },
  );
  await $("button=Save tape").click();
  await browser.switchToWindow(main);
}

describe("terminal takes", function () {
  this.timeout(RUN_MS);

  before(engineReady);

  it("records the terminal example in a real shell and exports its film", async () => {
    const dir = path.join(PROJECTS, "terminal");
    expect((await api("POST", "/open", { dir })).status).toBe(200);
    const artifacts: Artifact[] = [];
    let done: Record<string, unknown> | null = null;
    const finished = runEvents((event) => {
      if (event.type === "node:done") {
        const result = event.result as { artifacts?: Artifact[] } | undefined;
        artifacts.push(...(result?.artifacts ?? []));
      }
      if (event.type !== "run:done") return false;
      done = event;
      return true;
    });
    expect((await api("POST", "/project/run", {})).status).toBe(200);
    await finished;
    expect(done).toMatchObject({ ok: true, cancelled: false });
    const cast = artifacts.find((a) => a.path.endsWith("take.cast"));
    const film = artifacts.find((a) => a.path.endsWith(".mp4"));
    if (!cast || !film) throw new Error(`missing artifacts: ${JSON.stringify(artifacts)}`);
    const recorded = readFileSync(path.join(dir, cast.path), "utf8");
    expect(recorded.split("\n")[0]).toContain('"version":3');
    expect(recorded).toContain("Opsis records real terminals");
    expect(statSync(path.join(dir, film.path)).size).toBeGreaterThan(0);
  });

  it("plays a terminal take in Replay and lists the themes with their colours", async () => {
    await browser.execute(() => {
      const page = window as unknown as {
        __TAURI_INTERNALS__: { invoke: (cmd: string, args: unknown) => Promise<unknown> };
      };
      void page.__TAURI_INTERNALS__.invoke("show_replay", { nodeId: "rec", tab: "replay" });
    });
    await $('[role="dialog"] [aria-label="Terminal"]').waitForExist({ timeout: WINDOW_MS });
    await $('[role="dialog"] button[aria-label="Play"]').click();
    await browser.waitUntil(
      () =>
        browser.execute(
          () =>
            document
              .querySelector('[role="dialog"] [aria-label="Terminal"]')
              ?.textContent?.includes("Opsis records real terminals") ?? false,
        ),
      { timeout: WINDOW_MS, timeoutMsg: "the terminal player never played the take" },
    );
    await browser.saveScreenshot(path.join(RUN_DIR, "terminal-replay.png"));
    await browser.keys("Escape");
    await $("button:has(svg.lucide-maximize)").click();
    await $('[data-id="look"]').waitForExist({ timeout: WINDOW_MS });
    await browser.execute(() => document.querySelector<HTMLElement>('[data-id="look"]')?.click());
    const theme = await $("button*=Catppuccin Mocha");
    await theme.waitForExist({ timeout: WINDOW_MS });
    expect(await theme.getText()).toContain("Catppuccin Mocha");
    await theme.click();
    await browser.waitUntil(
      () =>
        browser.execute(() => document.querySelectorAll('[data-slot="command-item"]').length > 300),
      { timeout: WINDOW_MS, timeoutMsg: "the theme list never opened" },
    );
    await browser.keys("Escape");
  });

  it("records a tape live in its own window, then plays it into a film", async () => {
    const dir = path.join(PROJECTS, "terminal");
    const tape = path.join(dir, "live.tape");
    rmSync(tape, { force: true });
    const main = await browser.getWindowHandle();
    const nodes = [
      { type: "terminal.playTape", id: "tape", params: { file: "project:live.tape" } },
      { type: "camera.take", id: "tapeTake" },
      { type: "output.render", id: "tapeFilm", params: { fileName: "tape film" } },
    ];
    for (const node of nodes) expect((await api("POST", "/project/nodes", node)).status).toBe(200);
    const edges = [
      ["tape", "recording", "tapeTake", "recording"],
      ["tape", "style", "tapeTake", "style"],
      ["tapeTake", "clip", "tapeFilm", "clip"],
    ];
    for (const [source, sourceHandle, target, targetHandle] of edges)
      expect(
        (await api("POST", "/project/edges", { source, sourceHandle, target, targetHandle }))
          .status,
      ).toBe(200);
    try {
      const live = runUntilDone({ targetNodeIds: ["tape"] });
      await typeIntoTheTerminalWindow(main, "echo live-tape");
      expect((await live).done).toMatchObject({ ok: true, cancelled: false });
      expect(existsSync(tape)).toBe(true);
      expect(readFileSync(tape, "utf8")).toContain('Type "echo live-tape"');
      const played = await runUntilDone({ targetNodeIds: ["tapeFilm"] });
      expect(played.done).toMatchObject({ ok: true, cancelled: false });
      const cast = played.artifacts.find((a) => a.path.endsWith("take.cast"));
      const film = played.artifacts.find((a) => a.path.endsWith("tape-film.mp4"));
      if (!cast || !film) throw new Error(`missing artifacts: ${JSON.stringify(played.artifacts)}`);
      expect(readFileSync(path.join(dir, cast.path), "utf8")).toContain("live-tape");
      expect(statSync(path.join(dir, film.path)).size).toBeGreaterThan(0);
    } finally {
      for (const node of nodes) await api("DELETE", `/project/nodes/${node.id}`);
      rmSync(tape, { force: true });
    }
  });
  it("imports a cast file as a take and exports its film", async () => {
    const dir = path.join(PROJECTS, "terminal");
    const cast = path.join(dir, "imported.cast");
    copyFileSync(path.join(ROOT, "crates/terminal/tests/casts/demo.cast"), cast);
    const nodes = [
      { type: "terminal.castFile", id: "cast", params: { file: "project:imported.cast" } },
      { type: "camera.take", id: "castTake" },
      { type: "output.render", id: "castFilm", params: { fileName: "cast film" } },
    ];
    for (const node of nodes) expect((await api("POST", "/project/nodes", node)).status).toBe(200);
    for (const [source, sourceHandle, target, targetHandle] of [
      ["cast", "recording", "castTake", "recording"],
      ["castTake", "clip", "castFilm", "clip"],
    ])
      expect(
        (await api("POST", "/project/edges", { source, sourceHandle, target, targetHandle }))
          .status,
      ).toBe(200);
    try {
      const played = await runUntilDone({ targetNodeIds: ["castFilm"] });
      expect(played.done).toMatchObject({ ok: true, cancelled: false });
      const film = played.artifacts.find((a) => a.path.endsWith("cast-film.mp4"));
      if (!film) throw new Error(`missing artifacts: ${JSON.stringify(played.artifacts)}`);
      expect(statSync(path.join(dir, film.path)).size).toBeGreaterThan(0);
    } finally {
      for (const node of nodes) await api("DELETE", `/project/nodes/${node.id}`);
      rmSync(cast, { force: true });
    }
  });
});
