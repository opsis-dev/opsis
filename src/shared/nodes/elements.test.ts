import { describe, expect, it } from "vitest";
import { emptyClip, groupOf } from "../compile/clip";
import { ELEMENTS } from "../elements";
import { fieldMeta } from "../elements/introspect";
import { compileClip, must, nodeCard } from "../test-fixtures";
import { getNodeDef } from "./registry";

const SPRING_SETTLE = "fx.remocn.spring-settle";

describe("Spring settle", () => {
  it("plays the connected clips as scenes with its settings, pause and scene colours", () => {
    const clip = compileClip(
      SPRING_SETTLE,
      { gapFrames: 4, enterScale: 1.3, backgrounds: [{ scene: "in3", color: "#112233" }] },
      { in1: emptyClip(30), in3: emptyClip(20) },
    );
    expect(clip.durationInFrames).toBe(54);
    const sequencer = must(groupOf(clip.tracks[0].items[0])?.sequencer, "sequencer");
    expect(sequencer.component).toBe("remocn/spring-settle");
    expect(sequencer.props).toMatchObject({ enterScale: 1.3, gapFrames: 4 });
    expect(sequencer.backgrounds).toEqual([null, "#112233"]);
  });

  it("counts its scenes and the items in each on its card", () => {
    const rows = nodeCard(
      SPRING_SETTLE,
      { gapFrames: 15 },
      { connected: ["in1", "in2"], inputItems: { in1: 1, in2: 3 }, clipFrames: 90 },
    ).map((r) => r.text);
    expect(rows[0]).toBe("2 scenes");
    expect(rows[1]).toBe("Items per scene: 1 · 3");
    expect(rows.some((text) => text.endsWith("between scenes"))).toBe(true);
  });

  it("warns when no scene is connected", () => {
    expect(nodeCard(SPRING_SETTLE)[0]).toMatchObject({ text: "0 scenes", tone: "warn" });
  });
});

const firstItem = (type: string, params: Record<string, unknown>) =>
  must(compileClip(type, params).tracks[0]?.items[0], `${type} item`);

describe("element settings", () => {
  it("makes files inside rows project files", () => {
    const item = firstItem("fx.remocn.logo-enter", {
      logos: [
        { image: "assets/a.svg", background: "#000" },
        { image: "app:placeholders/logo-mark.svg", background: "#111" },
      ],
    });
    expect(item.props.logos).toEqual([
      { image: "project:assets/a.svg", background: "#000" },
      { image: "app:placeholders/logo-mark.svg", background: "#111" },
    ]);
  });

  it("leaves out colours left on the theme", () => {
    expect(firstItem("fx.remotion-ui.stat-card", { backgroundColor: "" }).props).not.toHaveProperty(
      "backgroundColor",
    );
    expect(
      firstItem("fx.remotion-ui.stat-card", { backgroundColor: "#123456" }).props,
    ).toMatchObject({
      backgroundColor: "#123456",
    });
  });

  it("shows a palette on the card", () => {
    const palette = nodeCard("fx.remocn.shader-mesh-gradient").find((r) => r.icon === "palette");
    expect(palette).toMatchObject({
      text: "4 colours",
      swatches: ["#12121a", "#232338", "#3a3a5c", "#52527a"],
    });
  });
});

describe("element defaults", () => {
  it("pass their own schema", () => {
    const failing = ELEMENTS.flatMap((meta) => {
      const parsed = meta.schema.safeParse(meta.defaultProps);
      return parsed.success ? [] : [`${meta.key}: ${parsed.error.issues[0]?.path.join(".")}`];
    });
    expect(failing).toEqual([]);
  });
});

describe("element music", () => {
  it("plays through the mix as the node's own channel, never inside the element", () => {
    const clip = compileClip("fx.remocn.release-teaser", {
      music: "assets/song.mp3",
      musicVolume: 0.4,
    });
    expect(clip.audio).toEqual([
      expect.objectContaining({
        src: "project:assets/song.mp3",
        volume: 0.4,
        channel: "n",
        role: "music",
        durationInFrames: clip.durationInFrames,
      }),
    ]);
    const props = must(clip.tracks[0]?.items[0], "teaser item").props;
    expect(props).not.toHaveProperty("music");
    expect(props).not.toHaveProperty("musicVolume");
  });

  it("adds no sound while the music is None", () => {
    expect(compileClip("fx.remocn.release-teaser", {}).audio).toEqual([]);
  });
});

describe("element look rows", () => {
  it("shows a gradient's stops while no file replaces the gradient", () => {
    const rows = nodeCard("fx.remotion-ui.text-mask-video");
    expect(rows.find((r) => r.icon === "blend")).toMatchObject({
      text: "Straight gradient",
      swatches: ["#e8b86d", "#f472b6", "#2dd4bf"],
    });
    expect(
      nodeCard("fx.remotion-ui.text-mask-video", { src: "project:assets/clip.mp4" }).some(
        (r) => r.icon === "blend",
      ),
    ).toBe(false);
  });

  it("shows the theme and its changed colours", () => {
    expect(nodeCard("fx.snapcn.hero-launch").find((r) => r.icon === "moon")?.text).toBe(
      "Dark theme",
    );
    const light = nodeCard("fx.snapcn.hero-launch", {
      theme: { mode: "light", colors: { background: "#fff" } },
    });
    expect(light.find((r) => r.icon === "sun")).toMatchObject({
      text: "Light theme · 1 colour changed",
      swatches: ["#fff"],
    });
  });
});

describe("element timing", () => {
  it("passes times set in seconds to the element as frames of 30 a second", () => {
    expect(firstItem("fx.snapcn.status-cycle", { statusHold: 1.5 }).props.statusHold).toBe(45);
    expect(firstItem("fx.snapcn.punch-lines", { holds: "1, 2.5" }).props.holds).toBe("30, 75");
    const steps = firstItem("fx.remocn.keystroke", { steps: [{ keys: "mod+k", at: 2, hold: 0.5 }] })
      .props.steps;
    expect(steps).toEqual([{ keys: "mod+k", at: 60, hold: 15 }]);
  });
});

describe("element layout", () => {
  it("draws a fixed-size element on the frame it was laid out for", () => {
    expect(firstItem("fx.remotion-ui.funnel-chart", {}).design).toEqual({
      width: 1920,
      height: 1080,
    });
    expect(firstItem("fx.remocn.soft-blur-in", { text: "Hi" }).design).toBeUndefined();
  });
});

describe("Device mockup", () => {
  const DEVICE_MOCKUP = "fx.remotion-ui.device-mockup-zoom";

  it("runs without a clip, showing its own screen", () => {
    expect(must(getNodeDef(DEVICE_MOCKUP), "node").inputs.clip?.optional).toBe(true);
    const item = firstItem(DEVICE_MOCKUP, { src: "project:assets/screen.png" });
    expect(item.component).toBe("remotion-ui/device-mockup-zoom");
    expect(item.props).toMatchObject({ src: "project:assets/screen.png" });
  });

  it("says how long it plays with and without a clip", () => {
    const length = must(getNodeDef(DEVICE_MOCKUP), "node").params.shape.clipDuration;
    expect(fieldMeta(length)).toMatchObject({
      zeroLabel: "Match the clip",
      unconnectedZeroLabel: "Usual length",
    });
    const clock = (connected: string[]) =>
      nodeCard(DEVICE_MOCKUP, {}, { connected }).find((r) => r.icon === "clock")?.text;
    expect(clock(["clip"])).toBe("Matches the clip");
    expect(clock([])).toBe("3 s");
    expect(
      nodeCard("fx.remotion-ui.multi-device-lineup").find((r) => r.icon === "clock")?.text,
    ).toBe("Matches the clip");
  });

  it("puts a connected clip on the screen", () => {
    const clip = compileClip(DEVICE_MOCKUP, {}, { clip: emptyClip(45) });
    expect(clip.durationInFrames).toBe(45);
    expect(groupOf(must(clip.tracks[0]?.items[0], "item"))?.wrapper?.component).toBe(
      "remotion-ui/device-mockup-zoom",
    );
  });
});
