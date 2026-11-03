import { readFileSync, statSync } from "node:fs";
import path from "node:path";
import { expect } from "@wdio/globals";
import { api, runEvents } from "../support/api";
import { engineReady } from "../support/engine";
import { PROJECTS } from "../support/paths";

const RUN_MS = 600_000;

type Artifact = { path: string; label?: string };

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
});
