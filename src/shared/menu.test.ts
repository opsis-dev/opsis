import { describe, expect, it } from "vitest";
import { acceleratorLabel, appMenu, matchesAccelerator } from "./menu";

const key = (
  k: string,
  mods: Partial<Record<"ctrlKey" | "metaKey" | "altKey" | "shiftKey", boolean>> = {},
) => ({
  key: k,
  ctrlKey: false,
  metaKey: false,
  altKey: false,
  shiftKey: false,
  ...mods,
});

describe("app menu", () => {
  it("matches shortcuts per platform", () => {
    expect(matchesAccelerator("CmdOrCtrl+B", key("b", { ctrlKey: true }), "linux")).toBe(true);
    expect(matchesAccelerator("CmdOrCtrl+B", key("b", { metaKey: true }), "darwin")).toBe(true);
    expect(
      matchesAccelerator("CmdOrCtrl+B", key("b", { ctrlKey: true, altKey: true }), "linux"),
    ).toBe(false);
    expect(
      matchesAccelerator("CmdOrCtrl+Alt+B", key("b", { ctrlKey: true, altKey: true }), "linux"),
    ).toBe(true);
    expect(
      matchesAccelerator("CmdOrCtrl+Shift+P", key("P", { ctrlKey: true, shiftKey: true }), "win32"),
    ).toBe(true);
    expect(
      matchesAccelerator("CmdOrCtrl+=", key("+", { ctrlKey: true, shiftKey: true }), "linux"),
    ).toBe(true);
    expect(matchesAccelerator("F11", key("F11"), "linux")).toBe(true);
    expect(matchesAccelerator("CmdOrCtrl+,", key(",", { ctrlKey: true }), "linux")).toBe(true);
  });

  it("labels shortcuts the way each platform writes them", () => {
    expect(acceleratorLabel("CmdOrCtrl+Shift+P", "linux")).toBe("Ctrl+Shift+P");
    expect(acceleratorLabel("CmdOrCtrl+Shift+P", "darwin")).toBe("⌘⇧P");
    expect(acceleratorLabel("CmdOrCtrl+Enter", "win32")).toBe("Ctrl+Enter");
  });

  it("puts Quit in the menu only where there is no app menu", () => {
    const labels = (p: string) =>
      appMenu(p).flatMap((s) => s.items.map((i) => ("label" in i ? i.label : "")));
    expect(labels("linux")).toContain("Quit");
    expect(labels("darwin")).not.toContain("Quit");
  });

  it("gives every shortcut to only one item", () => {
    for (const platform of ["linux", "win32", "darwin"]) {
      const accels = appMenu(platform)
        .flatMap((s) => s.items)
        .flatMap((i) => (i.kind !== "separator" && i.accelerator ? [i.accelerator] : []));
      expect(new Set(accels).size).toBe(accels.length);
    }
  });
});
