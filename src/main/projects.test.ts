import { rm } from "node:fs/promises";
import { must } from "@opsis/core/test-fixtures";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { tempDirs } from "../engine/test-temp-dirs";
import { closeDatabase, db, IN_MEMORY, openDatabase } from "./db";
import { projects } from "./db/schema";
import { createProjectDir, listRecent, openProjectDir } from "./projects";
import { readViewState, writeViewState } from "./view-state";

vi.mock("electron", () => ({ app: { getPath: () => "" } }));

const tempDir = tempDirs("opsis-projects-");

beforeEach(() => {
  openDatabase(IN_MEMORY);
  return closeDatabase;
});

const newProject = async (name: string) =>
  must(await createProjectDir(null, { name, template: "blank", dir: await tempDir() }), name).dir;

describe("recent projects", () => {
  it("lists the last opened first and skips folders without a project", async () => {
    const first = await newProject("First");
    const second = await newProject("Second");
    const gone = await newProject("Gone");
    await rm(gone, { recursive: true });
    await openProjectDir(first);
    expect(listRecent().map((p) => p.dir)).toEqual([first, second]);
  });

  it("keeps every project it has seen beyond the recent list", async () => {
    for (let i = 0; i < 21; i++) await newProject(`P${i}`);
    expect(listRecent()).toHaveLength(20);
    expect(db().select().from(projects).all()).toHaveLength(21);
  });
});

describe("view state", () => {
  it("returns what was saved for the project when it opens", async () => {
    const dir = await newProject("Views");
    expect(writeViewState(dir, "dockTab", "mixer")).toBe(true);
    writeViewState(dir, "dockTab", "timeline");
    writeViewState(dir, "mixerCollapsed", ["music"]);
    expect((await openProjectDir(dir)).view).toEqual({
      dockTab: "timeline",
      mixerCollapsed: ["music"],
    });
  });

  it("saves nothing for a folder that was never opened", async () => {
    const dir = await tempDir();
    expect(writeViewState(dir, "dockTab", "mixer")).toBe(false);
    expect(readViewState(dir)).toEqual({});
  });

  it("starts a project made again in the same folder without the old view", async () => {
    const parent = await tempDir();
    const made = must(
      await createProjectDir(null, { name: "Again", template: "blank", dir: parent }),
      "project",
    );
    writeViewState(made.dir, "dockTab", "mixer");
    await rm(made.dir, { recursive: true });
    await createProjectDir(null, { name: "Again", template: "blank", dir: parent });
    expect(readViewState(made.dir)).toEqual({});
  });
});
