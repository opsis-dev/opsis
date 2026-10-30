import { afterEach, describe, expect, it, vi } from "vitest";
import { useLiveStore } from "./live";

vi.stubGlobal("URL", {
  ...URL,
  createObjectURL: () => "blob:frame",
  revokeObjectURL: () => undefined,
});

afterEach(() => {
  useLiveStore.getState().reset();
  useLiveStore.setState({ pipOpen: false, windowWatching: false });
});

function frame(width: number, height: number) {
  return {
    type: "frame" as const,
    captureId: "c1",
    nodeId: "record",
    jpeg: new Uint8Array(new ArrayBuffer(1)),
    width,
    height,
    t: 0,
  };
}

describe("the live view", () => {
  it("brings the picture-in-picture back when the live window stops watching", () => {
    useLiveStore.setState({ pipOpen: false });
    useLiveStore.getState().setWindowWatching(true);
    expect(useLiveStore.getState().pipOpen).toBe(false);
    useLiveStore.getState().setWindowWatching(false);
    expect(useLiveStore.getState().pipOpen).toBe(true);
  });

  it("takes a capture's size from its first frame when it joined mid-capture", () => {
    useLiveStore.getState().apply(frame(1280, 720));
    const capture = useLiveStore.getState().captures.c1;
    expect(capture?.viewport).toEqual({ width: 1280, height: 720 });
    expect(capture?.frameUrl).toBe("blob:frame");
  });
});
