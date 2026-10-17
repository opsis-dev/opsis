import { describe, expect, it } from "vitest";
import { detectBeats, waveformLevels } from "./analysis";

const RATE = 8000;

function clicks(bpm: number, seconds: number, offset = 0.25): Float32Array {
  const samples = new Float32Array(RATE * seconds);
  let seed = 1;
  for (let i = 0; i < samples.length; i++) {
    seed = (seed * 16807) % 2147483647;
    samples[i] = (seed / 2147483647 - 0.5) * 0.05;
  }
  const gap = 60 / bpm;
  for (let t = offset; t < seconds; t += gap) {
    const start = Math.round(t * RATE);
    for (let i = 0; i < 200 && start + i < samples.length; i++)
      samples[start + i] += Math.sin(i / 3) * Math.exp(-i / 60);
  }
  return samples;
}

describe("detectBeats", () => {
  it("finds the tempo and the beats of a steady click", () => {
    const { bpm, beats } = detectBeats(clicks(120, 12), RATE);
    expect(bpm).toBe(120);
    expect(beats.length).toBeGreaterThan(20);
    for (const [i, t] of beats.slice(1).entries()) expect(t - beats[i]).toBeCloseTo(0.5, 1);
    expect(beats[0] % 0.5).toBeCloseTo(0.25, 1);
  });

  it("finds no beat in silence or in a short sound", () => {
    expect(detectBeats(new Float32Array(RATE * 12), RATE)).toEqual({ beats: [], bpm: null });
    expect(detectBeats(clicks(120, 2), RATE)).toEqual({ beats: [], bpm: null });
  });
});

describe("waveformLevels", () => {
  it("keeps the lowest and highest sample of each step", () => {
    const samples = Float32Array.from([0.5, -0.25, 0.125, -1]);
    expect(waveformLevels(samples, 4, [2])).toEqual([
      { rate: 2, min: [-0.25, -1], max: [0.5, 0.125] },
    ]);
  });
});
