import { afterEach, describe, expect, it, vi } from "vitest";
import type { MixPlan } from "@/generated/bindings";
import { mixerBook } from "@/store/catalog";
import { useMedia } from "./media";

const failingAnalyze = vi.fn(() => Promise.reject(new Error("worker gone")));

vi.stubGlobal("window", { opsis: { analyze: failingAnalyze } });

afterEach(() => vi.useRealTimers());

const WAVE = { src: "", durationMs: 1000, levels: [], beats: [], bpm: 0 };

describe("media requests", () => {
  it("marks a waveform as failed when the request itself fails", async () => {
    useMedia.getState().reset("/project");
    useMedia.getState().waveform("project:a.wav");
    expect(useMedia.getState().analyses.waveform["project:a.wav"]).toBe("loading");
    await vi.waitFor(() =>
      expect(useMedia.getState().analyses.waveform["project:a.wav"]).toBe("error"),
    );
  });

  it("stops showing a mix as in progress when the request itself fails", async () => {
    vi.useFakeTimers();
    useMedia.getState().reset("/project");
    const plan: MixPlan = {
      fps: 30,
      durationInFrames: 30,
      channels: [],
      buses: [],
      master: mixerBook().defaultMaster,
      key: "silent",
    };
    useMedia.getState().requestMix("film", plan);
    expect(useMedia.getState().mix.status).toEqual({ kind: "pending", key: plan.key });
    await vi.runAllTimersAsync();
    expect(useMedia.getState().mix.status).toEqual({
      kind: "failed",
      key: plan.key,
      error: "worker gone",
    });
  });

  it("loads a changed file's waveform again and keeps the old one meanwhile", async () => {
    useMedia.getState().reset("/project");
    useMedia.setState({
      analyses: { waveform: { "project:assets/a.wav": WAVE, "b.wav": WAVE }, filmstrip: {} },
    });
    failingAnalyze.mockClear();
    useMedia.getState().reloadChanged(["project:assets/a.wav"]);
    expect(failingAnalyze).toHaveBeenCalledOnce();
    expect(useMedia.getState().analyses.waveform["project:assets/a.wav"]).toBe(WAVE);
    await vi.waitFor(() =>
      expect(useMedia.getState().analyses.waveform["project:assets/a.wav"]).toBe("error"),
    );
    expect(useMedia.getState().analyses.waveform["b.wav"]).toBe(WAVE);
  });
});
