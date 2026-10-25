import { describe, expect, it } from "vitest";
import { defaultRecordingProps } from "../compile/recording";
import { timelineSpecSchema } from "./timeline";

const log = {
  viewport: { width: 1280, height: 720 },
  dpr: 1,
  t0: 0,
  durationMs: 1000,
  actions: [],
};
const recordingProps = { ...defaultRecordingProps(), src: "project:take.mp4", name: "Take", log };
const place = { id: "i", from: 0, durationInFrames: 30 };

const specWith = (item: Record<string, unknown>) => ({
  width: 1920,
  height: 1080,
  fps: 30,
  durationInFrames: 30,
  tracks: [{ id: "t", items: [item] }],
});

const accepts = (item: Record<string, unknown>) =>
  timelineSpecSchema.safeParse(specWith(item)).success;

describe("timeline items", () => {
  it("reads a group, a recording and an element by their kind", () => {
    const recording = { ...place, kind: "recording", props: recordingProps };
    const element = { ...place, kind: "element", component: "core/title", props: { title: "Hi" } };
    const group = { ...place, kind: "group", props: { tracks: [{ id: "n", items: [recording] }] } };
    expect(accepts(group)).toBe(true);
    expect(accepts(element)).toBe(true);
    expect(timelineSpecSchema.parse(specWith(group)).tracks[0].items[0]).toEqual(group);
  });

  it("refuses props that don't fit the kind", () => {
    expect(accepts({ ...place, kind: "group", props: recordingProps })).toBe(false);
    expect(
      accepts({ ...place, kind: "recording", props: { ...recordingProps, log: "take" } }),
    ).toBe(false);
    expect(accepts({ ...place, kind: "element", props: {} })).toBe(false);
    expect(accepts({ ...place, component: "core/group", props: { tracks: [] } })).toBe(false);
  });

  it("checks items inside groups too", () => {
    const bad = { ...place, kind: "recording", props: { ...recordingProps, style: "fancy" } };
    expect(
      accepts({ ...place, kind: "group", props: { tracks: [{ id: "n", items: [bad] }] } }),
    ).toBe(false);
  });
});
