import type { MixResult, StemLevels } from "@/shared/mix/result";
import type { Eq, Mix } from "@/shared/mix/types";
import { useAudioPreview, type Contribution } from "../../store/audio-preview";
import { useWorkflowStore } from "../../store/workflow";
import type { DragPhase } from "./knob";
import { frameIndex } from "./meter";

const SILENCE_DB = -120;
const HIGHPASS_Q = 0.7;

export type PreviewTarget = { key: string; fraction: number };

type Change<T> = (value: T, phase: DragPhase) => void;

export function gesture(phase: DragPhase, apply: () => void): void {
  const store = useWorkflowStore.getState();
  if (phase === "start") store.beginGesture();
  else if (phase === "move") apply();
  else store.endGesture();
}

export const bindChange =
  <T>(apply: (value: T) => void): Change<T> =>
  (value, phase) =>
    gesture(phase, () => apply(value));

function previewChange(
  id: string,
  current: number,
  next: number,
  make: (delta: number) => Contribution,
): void {
  const store = useAudioPreview.getState();
  store.contribute(id, make(next - store.baseline(id, current)));
}

function gainContribution(fraction: number, deltaDb: number): Contribution {
  const p = Math.max(1e-9, 1 - fraction + fraction * 10 ** (deltaDb / 10));
  return { gainDb: 10 * Math.log10(p) };
}

export function share(
  result: MixResult | null,
  levels: Pick<StemLevels, "rms"> | undefined,
  frame: number,
): number {
  const master = result?.levels.master;
  if (!master?.rms.length || !levels) return 1;
  const f = frameIndex(frame, master.rms.length);
  const pc = 10 ** ((levels.rms[f] ?? SILENCE_DB) / 10);
  const pm = 10 ** ((master.rms[f] ?? SILENCE_DB) / 10);
  return pm <= 1e-12 ? 1 : Math.min(1, pc / pm);
}

export const bindGain = (
  target: PreviewTarget,
  name: string,
  current: number,
  apply: (v: number) => void,
): Change<number> =>
  bindChange((v) => {
    previewChange(`${target.key}:${name}`, current, v, (d) => gainContribution(target.fraction, d));
    apply(v);
  });

export const bindPan = (
  target: PreviewTarget,
  current: number,
  apply: (v: number) => void,
): Change<number> =>
  bindChange((v) => {
    previewChange(`${target.key}:pan`, current, v, (d) => ({ pan: d * target.fraction }));
    apply(v);
  });

export const bindHighpass = (
  target: PreviewTarget,
  apply: (freq: number) => void,
): Change<number> =>
  bindChange((freq) => {
    useAudioPreview.getState().contribute(`${target.key}:hp`, {
      filters: [{ type: "highpass", freq, gain: 0, q: HIGHPASS_Q }],
    });
    apply(freq);
  });

export const bindEq =
  (target: PreviewTarget, before: Eq, apply: (eq: Eq) => void) =>
  (eq: Eq, phase: DragPhase, band?: number) =>
    gesture(phase, () => {
      if (band !== undefined) {
        const b = eq.bands[band];
        previewChange(`${target.key}:eq${band}`, before.bands[band].gain, b.gain, (d) => ({
          filters: [
            {
              type: b.type === "peak" ? "peaking" : b.type,
              freq: b.freq,
              gain: d * target.fraction,
              q: b.q,
            },
          ],
        }));
      }
      apply(eq);
    });

export const setMix = (fn: (mix: Mix) => Mix): void => useWorkflowStore.getState().setMix(fn);
