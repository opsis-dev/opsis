import { describe, expect, it, vi } from "vitest";
import { openRememberedWindow } from "./window-bounds";

const made = vi.hoisted(() => ({ windows: [] as unknown[], raised: [] as unknown[] }));

vi.mock("electron", () => ({ app: { getPath: () => "" }, screen: { getAllDisplays: () => [] } }));

vi.mock("./raise-window", () => ({
  raiseWindow: (win: unknown) => made.raised.push(win),
}));

vi.mock("./window-registry", () => ({
  rendererUrl: (hash: string) => `app://${hash}`,
  makeWindow: () => {
    const win = { on: () => undefined, loadURL: () => Promise.resolve() };
    made.windows.push(win);
    return win;
  },
}));

const open = () =>
  openRememberedWindow({
    key: "settings",
    existing: null,
    saved: () => Promise.resolve(null),
    save: () => undefined,
    options: { width: 100, height: 100, minWidth: 100, minHeight: 100, title: "Settings" },
    hash: "/settings",
    onClosed: () => undefined,
  });

describe("openRememberedWindow", () => {
  it("opens one window when asked twice before the saved bounds are read", async () => {
    const [first, second] = await Promise.all([open(), open()]);
    expect(made.windows).toHaveLength(1);
    expect(first).toBe(made.windows[0]);
    expect(second).toBeNull();
    expect(made.raised).toEqual([made.windows[0]]);
  });
});
