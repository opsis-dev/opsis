import { describe, expect, it } from "vitest";
import { chordFromEvent } from "./keys";

const key = (
  k: string,
  mods: Partial<Record<"ctrlKey" | "metaKey" | "altKey" | "shiftKey", boolean>> = {},
) =>
  chordFromEvent({
    key: k,
    ctrlKey: false,
    metaKey: false,
    altKey: false,
    shiftKey: false,
    ...mods,
  });

describe("recording a shortcut", () => {
  it("writes chords the way the browser expects", () => {
    expect(key("k", { ctrlKey: true })).toBe("Control+K");
    expect(key("Enter")).toBe("Enter");
    expect(key(" ", { metaKey: true, shiftKey: true })).toBe("Meta+Shift+Space");
    expect(key("Shift", { shiftKey: true })).toBeNull();
    expect(key("ArrowUp", { altKey: true })).toBe("Alt+ArrowUp");
    expect(key("Escape")).toBe("Escape");
  });
});
