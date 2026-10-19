import { beforeEach, describe, expect, it, vi } from "vitest";
import { closeDatabase, IN_MEMORY, openDatabase } from "./db";
import { openRememberedWindow } from "./window-bounds";

const made = vi.hoisted(() => ({
  windows: [] as Array<{ options: unknown; close: () => void }>,
  raised: [] as unknown[],
}));

vi.mock("electron", () => ({
  app: { getPath: () => "" },
  screen: { getAllDisplays: () => [{ workArea: { x: 0, y: 0, width: 1920, height: 1080 } }] },
}));

vi.mock("./raise-window", () => ({
  raiseWindow: (win: unknown) => made.raised.push(win),
}));

vi.mock("./window-registry", () => ({
  rendererUrl: (hash: string) => `app://${hash}`,
  makeWindow: (options: unknown) => {
    const handlers = new Map<string, () => void>();
    const bounds = { x: 100, y: 50, width: 800, height: 600 };
    const win = {
      options,
      on: (event: string, handler: () => void) => handlers.set(event, handler),
      loadURL: () => Promise.resolve(),
      isDestroyed: () => false,
      isMinimized: () => false,
      isMaximized: () => false,
      isFullScreen: () => false,
      getBounds: () => bounds,
      close: () => handlers.get("close")?.(),
    };
    made.windows.push(win);
    return win;
  },
}));

const open = () =>
  openRememberedWindow({
    key: "settings",
    existing: null,
    options: { width: 100, height: 100, minWidth: 100, minHeight: 100, title: "Settings" },
    hash: "/settings",
    onClosed: () => undefined,
  });

beforeEach(() => {
  made.windows = [];
  made.raised = [];
  openDatabase(IN_MEMORY);
  return closeDatabase;
});

describe("openRememberedWindow", () => {
  it("opens one window when asked twice at once", async () => {
    const [first, second] = await Promise.all([open(), open()]);
    expect(made.windows).toHaveLength(1);
    expect(first).toBe(made.windows[0]);
    expect(second).toBeNull();
    expect(made.raised).toEqual([made.windows[0]]);
  });

  it("opens a window where it was when it closed", async () => {
    const first = await open();
    first?.close();
    await open();
    expect(made.windows[1]?.options).toMatchObject({ x: 100, y: 50, width: 800, height: 600 });
  });
});
