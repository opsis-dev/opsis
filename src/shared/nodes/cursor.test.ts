import { describe, expect, it } from "vitest";
import { recordingProps } from "../compile/clip";
import { itemClip } from "../compile/clip";
import { DEFAULT_CURSOR_LOOK, DEFAULT_KEYCAP, defaultRecordingProps } from "../compile/recording";
import { compileClip, makeLog, nodeCard } from "../test-fixtures";
import { parseParams } from "./params";
import { getNodeDef } from "./registry";

const take = () => {
  const log = makeLog([], 3000);
  const props = { ...defaultRecordingProps(), src: "rec.mp4", log, name: "Take" };
  return itemClip("rec", "core/recording", props, 90);
};

const propsAfter = (type: string, params: Record<string, unknown>) =>
  recordingProps(compileClip(type, params, { clip: take() }))[0];

describe("cursor colours", () => {
  it("uses the default colour when a colour is cleared", () => {
    expect(propsAfter("cursor.style", { color: "", outline: " " }).cursorLook).toEqual(
      DEFAULT_CURSOR_LOOK,
    );
    expect(propsAfter("cursor.keystrokeChips", { keycap: "", ink: "" }).keystrokes).toMatchObject(
      DEFAULT_KEYCAP,
    );
    expect(propsAfter("cursor.spotlight", { color: "" }).annotations[0]?.color).toBe(
      "rgba(0,0,0,0.55)",
    );
  });

  it("refuses text that isn't a colour", () => {
    const def = getNodeDef("cursor.style");
    expect(def && parseParams(def.params, { color: "not a colour" })).toEqual({
      ok: false,
      problem: "Pick a colour",
    });
  });
});

describe("marks on actions", () => {
  it("asks for the text of a tooltip or callout, and offers none on marks that draw no text", () => {
    const problem = (type: string, text: string) => {
      const def = getNodeDef(type);
      const parsed = def && parseParams(def.params, { text });
      return parsed && !parsed.ok ? parsed.problem : null;
    };
    expect(problem("text.tooltip", " ")).toBe("Add the text to show");
    expect(problem("text.calloutArrow", "")).toBe("Add the text to show");
    expect(problem("cursor.spotlight", "")).toBeNull();
    expect(getNodeDef("cursor.highlightBox")?.params.shape.text.meta()?.hidden).toBe(true);
  });

  it("says a mark with no name to match shows on every action, not only clicks", () => {
    expect(nodeCard("cursor.spotlight", { match: "" })[0].text).toBe("On every action");
  });
});
