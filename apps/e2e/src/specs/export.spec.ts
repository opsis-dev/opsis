import { statSync } from "node:fs";
import path from "node:path";
import { $, browser, expect } from "@wdio/globals";
import { api, runEvents } from "../support/api";
import { PROJECTS } from "../support/paths";

const EXPORT_MS = 600_000;

const EXPORTED_VIDEO = "changelog-clip.mp4";

const PANEL_MS = 15_000;

type Artifact = { path: string };

describe("exports", function () {
  this.timeout(EXPORT_MS);

  it("runs the changelog example and writes its exports", async () => {
    const dir = path.join(PROJECTS, "changelog");
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
    expect(artifacts.length).toBeGreaterThan(0);
    for (const artifact of artifacts)
      expect(statSync(path.join(dir, artifact.path)).size).toBeGreaterThan(0);
  });

  it("plays the exported video in the Run panel", async () => {
    await browser.keys(["Control", "4"]);
    await $(`video[src$="${EXPORTED_VIDEO}"]`).waitForExist({ timeout: PANEL_MS });
    const played = await browser.executeAsync((name: string, done: (answer: string) => void) => {
      const element = document.querySelector<HTMLVideoElement>(`video[src$="${name}"]`);
      if (element === null) return done("no video");
      const answer = () =>
        done(
          element.error === null
            ? `duration ${element.duration}`
            : `error ${element.error.code} ${element.error.message}`,
        );
      if (element.readyState >= HTMLMediaElement.HAVE_METADATA || element.error !== null)
        return answer();
      element.addEventListener("loadedmetadata", answer);
      element.addEventListener("error", answer);
      element.load();
    }, EXPORTED_VIDEO);
    expect(played).toMatch(/^duration \d/);
  });
});
