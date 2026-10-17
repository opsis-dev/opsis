import { afterEach, describe, expect, it, vi } from "vitest";
import { buildMixPlan } from "@/shared/mix/plan";
import { useMedia } from "./media";

const failingAnalyze = vi.fn(() => Promise.reject(new Error("worker gone")));

vi.stubGlobal("window", { demoflow: { analyze: failingAnalyze } });

afterEach(() => vi.useRealTimers());

describe("media requests", () => {
  it("marks a waveform as failed when the request itself fails", async () => {
    useMedia.getState().reset("/project");
    useMedia.getState().waveform("project:a.wav");
    expect(useMedia.getState().waveforms["project:a.wav"]).toBe("loading");
    await vi.waitFor(() => expect(useMedia.getState().waveforms["project:a.wav"]).toBe("error"));
  });

  it("stops showing a mix as in progress when the request itself fails", async () => {
    vi.useFakeTimers();
    useMedia.getState().reset("/project");
    const plan = buildMixPlan({ tracks: [], audio: [], durationInFrames: 30, fps: 30 }, undefined);
    useMedia.getState().requestMix("film", plan);
    expect(useMedia.getState().mixing).toBe(plan.key);
    await vi.runAllTimersAsync();
    expect(useMedia.getState().mixing).toBeNull();
    expect(useMedia.getState().mixError).toBe("worker gone");
  });
});
