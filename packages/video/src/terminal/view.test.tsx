import { readFileSync } from "node:fs";
import path from "node:path";
import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { decodeFrames } from "#video/terminal/frames.ts";
import { isSprite } from "#video/terminal/sprites.ts";
import { cellBox, screenSize, TERMINAL_FONTS, TerminalView } from "#video/terminal/view.tsx";

const ROOT = path.resolve(import.meta.dirname, "../../../..");
const frames = decodeFrames(readFileSync(path.join(ROOT, "fixtures/terminal/take.frames")));
const look = {
  fontSize: 20,
  lineHeight: 1.2,
  letterSpacing: 0,
  padding: 24,
  cursor: "program" as const,
};

describe("terminal view", () => {
  it("sizes the screen from its cells and padding like Rust does", () => {
    expect(cellBox(look)).toEqual({ width: 12, height: 24 });
    expect(screenSize(look, 80, 20)).toEqual({ width: 1008, height: 528 });
  });

  it("draws one box per text run, every sprite in one layer, and the cursor while it is on", () => {
    const frame = frames.frames.at(-1);
    if (!frame) throw new Error("no frames");
    const all = frame.rowIndices.flatMap((i) => frames.rowTable[i] ?? []);
    function drawnAsSprite(text: string) {
      return [...text].length === 1 && isSprite(text.codePointAt(0) ?? 0);
    }
    const runs = all.filter((r) => !drawnAsSprite(r.text) || r.bg !== frame.bg).length + 1;
    const on = render(<TerminalView frames={frames} frame={frame} look={look} cursorOn />);
    const root = on.container.firstElementChild as HTMLElement;
    expect(root.style.width).toBe("1008px");
    expect(root.style.fontFamily).toContain("Opsis Terminal");
    expect(root.children.length).toBe(runs + (frame.cursor.visible ? 1 : 0));
    expect(on.container.querySelectorAll("svg")).toHaveLength(1);
    expect(on.container.querySelector("svg path")).not.toBeNull();
    on.unmount();
    const off = render(<TerminalView frames={frames} frame={frame} look={look} cursorOn={false} />);
    expect((off.container.firstElementChild as HTMLElement).children.length).toBe(runs);
  });

  it("names the hidden bundled families as its stack, in order", () => {
    const bundled = JSON.parse(
      readFileSync(path.join(ROOT, "crates/compile/src/fonts/bundled.generated.json"), "utf8"),
    ) as Array<{ family: string; hidden?: boolean }>;
    expect(bundled.filter((f) => f.hidden).map((f) => f.family)).toEqual([...TERMINAL_FONTS]);
  });
});
