import fs from "node:fs";
import path from "node:path";
import { $, browser, expect } from "@wdio/globals";
import { api, runEvents } from "../support/api";
import { engineReady } from "../support/engine";
import { PROJECTS, ROOT, RUN_DIR } from "../support/paths";

const WAIT_MS = 30_000;

type Artifact = { path: string };

const DIR = path.join(PROJECTS, "changelog");

async function runNode(id: string): Promise<Artifact[]> {
  const artifacts: Artifact[] = [];
  let ok = false;
  const finished = runEvents((event) => {
    if (event.type === "node:done") {
      const result = event.result as { artifacts?: Artifact[] } | undefined;
      artifacts.push(...(result?.artifacts ?? []));
    }
    if (event.type !== "run:done") return false;
    ok = event.ok === true;
    return true;
  });
  expect((await api("POST", "/project/run", { targetNodeIds: [id] })).status).toBe(200);
  await finished;
  expect(ok).toBe(true);
  return artifacts;
}

async function select(id: string): Promise<void> {
  await $("button:has(svg.lucide-maximize)").click();
  await $(`[data-id="${id}"]`).waitForExist({ timeout: WAIT_MS });
  await browser.execute((nodeId: string) => {
    document.querySelector<HTMLElement>(`[data-id="${nodeId}"]`)?.click();
  }, id);
}

function cardText(id: string) {
  return browser.execute(
    (nodeId: string) => document.querySelector(`[data-id="${nodeId}"]`)?.textContent ?? "",
    id,
  );
}

describe("inspector", function () {
  this.timeout(180_000);

  before(async () => {
    await engineReady();
    expect((await api("POST", "/open", { dir: DIR })).status).toBe(200);
  });

  after(async () => {
    for (const id of ["made", "split", "pair"]) await api("DELETE", `/project/nodes/${id}`);
  });

  it("plays a sound the node made below its settings", async () => {
    const node = {
      id: "made",
      type: "audio.generate",
      position: { x: 0, y: 1600 },
      params: { preset: "ding" },
    };
    expect((await api("POST", "/project/nodes", node)).status).toBe(200);
    const [sound] = await runNode("made");
    if (!sound) throw new Error("the node made no file");
    const name = path.basename(sound.path);
    await select("made");
    await browser.waitUntil(
      () =>
        browser.execute((file: string) => {
          const files = [...document.querySelectorAll("legend")].find(
            (el) => el.textContent?.trim() === "Files",
          );
          const audio = files?.closest("fieldset")?.querySelector(`audio[src*="${file}"]`);
          const player = audio?.closest('[data-slot="media-player"]');
          player?.setAttribute("data-e2e-player", "made");
          if (audio instanceof HTMLAudioElement) audio.muted = true;
          return audio instanceof HTMLAudioElement && audio.duration > 1;
        }, name),
      { timeout: WAIT_MS, timeoutMsg: "the Inspector never loaded the made sound" },
    );
    await $('[data-e2e-player="made"] button[aria-label="Play"]').click();
    await browser.waitUntil(
      () =>
        browser.execute(
          () =>
            (document.querySelector<HTMLAudioElement>('[data-e2e-player="made"] audio')
              ?.currentTime ?? 0) > 0.2,
        ),
      { timeout: WAIT_MS, timeoutMsg: "the made sound never played" },
    );
    await browser.saveScreenshot(path.join(RUN_DIR, "inspector-sound.png"));
  });

  it("shows a change made by typing only once the field is left", async () => {
    fs.copyFileSync(
      path.join(ROOT, "public", "backdrops", "aurora.jpg"),
      path.join(DIR, "assets", "pair.jpg"),
    );
    const added = [
      {
        id: "pair",
        type: "media.import",
        position: { x: 0, y: 1900 },
        params: { path: "assets/pair.jpg" },
      },
      { id: "split", type: "image.beforeAfter", position: { x: 320, y: 1900 }, params: {} },
    ];
    for (const node of added) expect((await api("POST", "/project/nodes", node)).status).toBe(200);
    for (const targetHandle of ["image", "after"]) {
      const edge = { source: "pair", sourceHandle: "media", target: "split", targetHandle };
      expect((await api("POST", "/project/edges", edge)).status).toBe(200);
    }
    await runNode("split");
    await select("split");
    await browser.waitUntil(async () => !(await cardText("split")).includes("Changed"), {
      timeout: WAIT_MS,
      timeoutMsg: "the card still shows a change after the run",
    });
    await browser.waitUntil(
      () =>
        browser.execute(() => {
          const label = [...document.querySelectorAll("label")].find(
            (el) => el.textContent?.trim() === "Before label",
          );
          const field = label?.htmlFor ? document.getElementById(label.htmlFor) : null;
          field?.setAttribute("data-e2e-field", "before");
          return field?.textContent?.includes("Before") === true;
        }),
      { timeout: WAIT_MS, timeoutMsg: "the Inspector never showed the Before label field" },
    );
    await $('[data-e2e-field="before"]').click();
    await browser.keys(["X"]);
    const typed = await browser.execute(
      () => document.querySelector('[data-e2e-field="before"]')?.textContent?.trim() ?? "",
    );
    expect(typed).toMatch(/^(XBefore|BeforeX)$/);
    await browser.waitUntil(async () => (await cardText("split")).includes(`“${typed}”`), {
      timeout: WAIT_MS,
      timeoutMsg: "the card never showed the typed label",
    });
    expect(await cardText("split")).not.toContain("Changed");
    await browser.execute(() => {
      const active = document.activeElement;
      if (active instanceof HTMLElement) active.blur();
    });
    await browser.waitUntil(async () => (await cardText("split")).includes("Changed"), {
      timeout: WAIT_MS,
      timeoutMsg: "the card never showed the change after the field was left",
    });
    await browser.saveScreenshot(path.join(RUN_DIR, "inspector-typing.png"));
  });
});
