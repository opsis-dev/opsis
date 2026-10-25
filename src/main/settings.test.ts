import { DEFAULT_SETTINGS } from "@opsis/core/settings";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { closeDatabase, db, IN_MEMORY, openDatabase } from "./db";
import { settings } from "./db/schema";
import { onSettingsChange, patchSettings, readSettings } from "./settings";

vi.mock("electron", () => ({ app: { getPath: () => "" } }));

beforeEach(() => {
  openDatabase(IN_MEMORY);
  return closeDatabase;
});

describe("settings", () => {
  it("starts with the defaults", () => {
    expect(readSettings()).toEqual(DEFAULT_SETTINGS);
  });

  it("keeps a patch and tells listeners what changed", () => {
    const seen: boolean[] = [];
    const off = onSettingsChange((next, previous) =>
      seen.push(previous.app.confirmExit, next.app.confirmExit),
    );
    patchSettings({ app: { confirmExit: false } });
    off();
    expect(readSettings().app).toEqual({ ...DEFAULT_SETTINGS.app, confirmExit: false });
    expect(seen).toEqual([true, false]);
  });

  it("forgets the last folder when the default folder changes", () => {
    patchSettings({ projects: { lastDir: "/a" } });
    patchSettings({ projects: { defaultDir: "/b" } });
    expect(readSettings().projects).toEqual({ defaultDir: "/b", lastDir: "" });
  });

  it("falls back to the defaults when a saved section can't be read", () => {
    db()
      .insert(settings)
      .values({ section: "app", value: { onStart: "nowhere" } })
      .run();
    expect(readSettings()).toEqual(DEFAULT_SETTINGS);
  });
});
