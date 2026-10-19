import type { BrowserWindow } from "electron";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { tempDirs } from "../engine/test-temp-dirs";
import { must } from "../shared/test-fixtures";
import { closeDatabase, IN_MEMORY, openDatabase } from "./db";
import { applyProjectWindow, rememberProjectWindow, shownProjectDir } from "./project-window";
import { createProjectDir } from "./projects";
import { readViewState } from "./view-state";

vi.mock("electron", () => ({
  app: { getPath: () => "" },
  screen: { getAllDisplays: () => [{ workArea: { x: 0, y: 0, width: 1920, height: 1080 } }] },
}));

const tempDir = tempDirs("opsis-project-window-");

beforeEach(() => {
  openDatabase(IN_MEMORY);
  return closeDatabase;
});

function fakeWindow(url = "opsis-app://app/index.html#/") {
  const handlers = new Map<string, () => void>();
  const state = {
    maximized: false,
    bounds: { x: 10, y: 20, width: 1600, height: 1000 },
    calls: [] as string[],
  };
  const win = {
    isDestroyed: () => false,
    isMinimized: () => false,
    isFullScreen: () => false,
    isMaximized: () => state.maximized,
    maximize: () => {
      state.maximized = true;
      state.calls.push("maximize");
    },
    unmaximize: () => {
      state.maximized = false;
      state.calls.push("unmaximize");
    },
    setBounds: (b: typeof state.bounds) => {
      state.bounds = b;
      state.calls.push("setBounds");
    },
    getNormalBounds: () => state.bounds,
    webContents: { getURL: () => url },
    on: (event: string, handler: () => void) => handlers.set(event, handler),
    close: () => handlers.get("close")?.(),
  };
  return { win, asWindow: win as unknown as BrowserWindow, state };
}

const editorUrl = (dir: string) =>
  `opsis-app://app/index.html#/editor?dir=${encodeURIComponent(dir)}`;

describe("shownProjectDir", () => {
  it("reads the project folder from the editor's address only", () => {
    expect(shownProjectDir(editorUrl("/home/a b/demo"))).toBe("/home/a b/demo");
    expect(shownProjectDir("http://localhost:5173/#/")).toBeNull();
    expect(shownProjectDir("http://localhost:5173/#/panel/mixer")).toBeNull();
  });
});

describe("applyProjectWindow", () => {
  it("maximizes for a project with no saved window", () => {
    const { asWindow, state } = fakeWindow();
    applyProjectWindow(asWindow, {});
    expect(state.calls).toEqual(["maximize"]);
  });

  it("puts the window back the way the project left it", () => {
    const { asWindow, state } = fakeWindow();
    state.maximized = true;
    const saved = { x: 100, y: 80, width: 1200, height: 800 };
    applyProjectWindow(asWindow, { mainWindow: { ...saved, maximized: false } });
    expect(state.calls).toEqual(["unmaximize", "setBounds"]);
    expect(state.bounds).toEqual(saved);
  });

  it("maximizes again over the saved size when the project was maximized", () => {
    const { asWindow, state } = fakeWindow();
    applyProjectWindow(asWindow, {
      mainWindow: { x: 100, y: 80, width: 1200, height: 800, maximized: true },
    });
    expect(state.calls).toEqual(["setBounds", "maximize"]);
  });
});

describe("rememberProjectWindow", () => {
  it("saves the window for the project the editor shows", async () => {
    const dir = must(
      await createProjectDir(null, { name: "Window", template: "blank", dir: await tempDir() }),
      "project",
    ).dir;
    const { win, asWindow } = fakeWindow(editorUrl(dir));
    rememberProjectWindow(asWindow);
    win.close();
    expect(readViewState(dir).mainWindow).toEqual({
      x: 10,
      y: 20,
      width: 1600,
      height: 1000,
      maximized: false,
    });
  });

  it("saves nothing while the start screen shows", () => {
    const { win, asWindow, state } = fakeWindow();
    rememberProjectWindow(asWindow);
    win.close();
    expect(state.calls).toEqual([]);
  });
});
