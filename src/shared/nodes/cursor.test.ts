import { describe, expect, it } from "vitest";
import { recordingProps } from "../compile/clip";
import { DEFAULT_CURSOR_LOOK, DEFAULT_KEYCAP } from "../compile/recording";
import { compileClip, nodeCard, zoomed, type StyleStepSpec } from "../test-fixtures";
import { parseParams } from "./params";
import { getNodeDef } from "./registry";

const propsAfter = (type: string, params: Record<string, unknown>) =>
  recordingProps(zoomed({ style: [[type, params]] }))[0];

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

  it("draws the Mac cursor without colours of its own", () => {
    expect(propsAfter("cursor.style", { look: "mac", color: "#ff0000" }).cursorLook).toEqual({
      shape: "mac",
    });
    expect(nodeCard("cursor.style", { look: "mac", size: 1.5 })[0]).toMatchObject({
      text: "Mac · 1.5×",
    });
    expect(nodeCard("cursor.style", { look: "mac" })[0]).not.toHaveProperty("swatches");
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

describe("camera and marks in a style", () => {
  const styled = (style: StyleStepSpec[], own: StyleStepSpec[] = []) =>
    recordingProps(zoomed({ style, own }))[0];

  it("keeps marks in the order their nodes are chained", () => {
    const props = styled([
      ["cursor.spotlight"],
      ["cursor.highlightBox"],
      ["text.tooltip", { text: "Save" }],
    ]);
    expect(props.annotations.map((a) => a.kind)).toEqual(["spotlight", "highlight", "tooltip"]);
  });

  it("applies the cursor, backdrop and zoom whichever order they come in", () => {
    const style: StyleStepSpec[] = [
      ["cursor.style", { size: 2 }],
      ["camera.backdrop", { kind: "solid", color: "#ff0000" }],
    ];
    const zoom: StyleStepSpec[] = [["camera.manualZoom", { atMs: 500 }]];
    const pick = (p: ReturnType<typeof styled>) => ({
      scale: p.cursorScale,
      backdrop: p.backdrop,
      beats: p.extraBeats.length,
    });
    const forward = pick(styled(style, zoom));
    expect(forward).toEqual({ scale: 2, backdrop: "#ff0000", beats: 2 });
    expect(pick(styled(style.toReversed(), zoom))).toEqual(forward);
  });

  it("keeps zooms and marks made before a laptop frame on its screen", () => {
    const clip = zoomed({
      style: [["cursor.spotlight"]],
      own: [["camera.manualZoom", { atMs: 500 }]],
    });
    const props = recordingProps(
      compileClip("camera.deviceFrame", { device: "laptop" }, { clip }),
    )[0];
    expect(props.screen).toBe(true);
    expect(props.extraBeats).toHaveLength(2);
    expect(props.annotations).toHaveLength(1);
  });
});
