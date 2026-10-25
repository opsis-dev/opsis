import { mkdir, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import type { ProjectFilesChange } from "@opsis/core/ipc";
import { afterEach, describe, expect, it, vi } from "vitest";
import { tempDirs } from "../engine/test-temp-dirs";
import { MISSING_FILE, watchLoadedFiles, type LoadedFilesWatch } from "./loaded-files-watch";

vi.mock("electron", () => ({ app: { getPath: () => "" } }));

const newDir = tempDirs("opsis-loaded-files-");
const open: LoadedFilesWatch[] = [];

afterEach(() => {
  for (const watch of open.splice(0)) watch.close();
});

const settle = () => new Promise((resolve) => setTimeout(resolve, 400));

async function watching(names: string[]) {
  const dir = await newDir();
  await mkdir(path.join(dir, "assets"));
  await mkdir(path.join(dir, ".opsis/cache"), { recursive: true });
  for (const name of names) await writeFile(path.join(dir, name), "one");
  const changes: ProjectFilesChange[] = [];
  const watch = watchLoadedFiles(dir, (change) => changes.push(change));
  open.push(watch);
  return { dir, changes, watch };
}

describe("watchLoadedFiles", () => {
  it("reports a loaded dotLottie file replaced in place with its new stamp", async () => {
    const { dir, changes, watch } = await watching(["assets/anim.lottie", "assets/other.png"]);
    watch.follow(["project:assets/anim.lottie"]);
    await writeFile(path.join(dir, "assets/other.png"), "two");
    await writeFile(path.join(dir, "assets/anim.lottie"), "three!");
    await vi.waitFor(() => expect(changes).toHaveLength(1));
    expect(Object.keys(changes[0].files)).toEqual(["assets/anim.lottie"]);
    expect(changes[0].files["assets/anim.lottie"]).toMatch(/^6:/);
  });

  it("reports a deleted file as missing and stops once it isn't loaded", async () => {
    const { dir, changes, watch } = await watching(["assets/a.wav"]);
    watch.follow(["project:assets/a.wav"]);
    await rm(path.join(dir, "assets/a.wav"));
    await vi.waitFor(() => expect(changes).toHaveLength(1));
    expect(changes[0].files).toEqual({ "assets/a.wav": MISSING_FILE });
    watch.follow([]);
    await writeFile(path.join(dir, "assets/a.wav"), "back");
    await settle();
    expect(changes).toHaveLength(1);
  });

  it("leaves the app's own files alone", async () => {
    const { dir, changes, watch } = await watching([".opsis/cache/take.mp4"]);
    watch.follow(["project:.opsis/cache/take.mp4"]);
    await writeFile(path.join(dir, ".opsis/cache/take.mp4"), "two");
    await settle();
    expect(changes).toHaveLength(0);
  });
});
