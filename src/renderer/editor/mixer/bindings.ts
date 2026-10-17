import type { MixResult, StemLevels } from "@/shared/mix/result";
import type { Eq } from "@/shared/mix/types";
import { useAudioPreview, type Contribution } from "../../store/audio-preview";
import { usePlayback } from "../../store/playback";
import { useWorkflowStore } from "../../store/workflow";
import type { Change, DragPhase } from "./gesture";
import { share } from "./meter-levels";

const HIGHPASS_Q = 0.7;

export type PreviewTarget = { key: string; fraction: () => number };

export const MASTER_TARGET: PreviewTarget = { key: "master", fraction: () => 1 };

export const previewTarget = (
  key: string,
  result: MixResult | null,
  levels: Pick<StemLevels, "rms"> | undefined,
): PreviewTarget => ({
  key,
  fraction: () => share(result, levels, usePlayback.getState().frame),
});

function gesture(phase: DragPhase, apply: () => void): void {
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
  store.contribute(id, make(next - store.rememberBaseline(id, current)));
}

function gainContribution(fraction: number, deltaDb: number): Contribution {
  const p = Math.max(1e-9, 1 - fraction + fraction * 10 ** (deltaDb / 10));
  return { gainDb: 10 * Math.log10(p) };
}

export const bindGain = (
  target: PreviewTarget,
  name: string,
  current: number,
  apply: (v: number) => void,
): Change<number> =>
  bindChange((v) => {
    previewChange(`${target.key}:${name}`, current, v, (d) =>
      gainContribution(target.fraction(), d),
    );
    apply(v);
  });

export const bindPan = (
  target: PreviewTarget,
  current: number,
  apply: (v: number) => void,
): Change<number> =>
  bindChange((v) => {
    previewChange(`${target.key}:pan`, current, v, (d) => ({ pan: d * target.fraction() }));
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
              gain: d * target.fraction(),
              q: b.q,
            },
          ],
        }));
      }
      apply(eq);
    });
