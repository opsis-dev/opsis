import { describe, expect, it } from "vitest";
import { acceleratorLabel, appMenu, isPrimaryModifier, matchesAccelerator } from "#core/menu";
import { keyEvent } from "#core/test-fixtures";

describe("app menu", () => {
  it("matches shortcuts per platform", () => {
    expect(matchesAccelerator("CmdOrCtrl+B", keyEvent("b", { ctrlKey: true }), "linux")).toBe(true);
    expect(matchesAccelerator("CmdOrCtrl+B", keyEvent("b", { metaKey: true }), "darwin")).toBe(
      true,
    );
    expect(
      matchesAccelerator("CmdOrCtrl+B", keyEvent("b", { ctrlKey: true, altKey: true }), "linux"),
    ).toBe(false);
    expect(
      matchesAccelerator(
        "CmdOrCtrl+Alt+B",
        keyEvent("b", { ctrlKey: true, altKey: true }),
        "linux",
      ),
    ).toBe(true);
    expect(
      matchesAccelerator(
        "CmdOrCtrl+Shift+P",
        keyEvent("P", { ctrlKey: true, shiftKey: true }),
        "win32",
      ),
    ).toBe(true);
    expect(
      matchesAccelerator("CmdOrCtrl+=", keyEvent("+", { ctrlKey: true, shiftKey: true }), "linux"),
    ).toBe(true);
    expect(matchesAccelerator("F11", keyEvent("F11"), "linux")).toBe(true);
    expect(matchesAccelerator("CmdOrCtrl+,", keyEvent(",", { ctrlKey: true }), "linux")).toBe(true);
  });

  it("matches letter shortcuts with Caps Lock on", () => {
    expect(matchesAccelerator("CmdOrCtrl+C", keyEvent("C", { ctrlKey: true }), "linux")).toBe(true);
    expect(matchesAccelerator("CmdOrCtrl+D", keyEvent("D", { metaKey: true }), "darwin")).toBe(
      true,
    );
  });

  it("labels shortcuts the way each platform writes them", () => {
    expect(acceleratorLabel("CmdOrCtrl+Shift+P", "linux")).toBe("Ctrl+Shift+P");
    expect(acceleratorLabel("CmdOrCtrl+Shift+P", "darwin")).toBe("⌘⇧P");
    expect(acceleratorLabel("CmdOrCtrl+Enter", "win32")).toBe("Ctrl+Enter");
  });

  it("puts Quit in the menu only where there is no app menu", () => {
    const labels = (p: NodeJS.Platform) =>
      appMenu(p).flatMap((s) => s.items.map((i) => ("label" in i ? i.label : "")));
    expect(labels("linux")).toContain("Quit");
    expect(labels("darwin")).not.toContain("Quit");
  });

  it("gives every shortcut to only one item", () => {
    for (const platform of ["linux", "win32", "darwin"] as const) {
      const accels = appMenu(platform)
        .flatMap((s) => s.items)
        .flatMap((i) => (i.kind !== "separator" && i.accelerator ? [i.accelerator] : []));
      expect(new Set(accels).size).toBe(accels.length);
    }
  });
});

describe("isPrimaryModifier", () => {
  it("is Cmd on macOS and Ctrl elsewhere, like CmdOrCtrl in a menu accelerator", () => {
    const ctrl = keyEvent("=", { ctrlKey: true });
    const cmd = keyEvent("=", { metaKey: true });
    expect(isPrimaryModifier(cmd, "darwin")).toBe(true);
    expect(isPrimaryModifier(ctrl, "darwin")).toBe(false);
    expect(isPrimaryModifier(ctrl, "linux")).toBe(true);
    expect(isPrimaryModifier(cmd, "win32")).toBe(false);
    for (const platform of ["darwin", "linux", "win32"] as const)
      for (const e of [ctrl, cmd])
        expect(isPrimaryModifier(e, platform)).toBe(matchesAccelerator("CmdOrCtrl+=", e, platform));
  });
});
