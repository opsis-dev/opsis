import type { MixResult, Eq, EqType } from "@/generated/bindings";
import { useAudioPreview, type Contribution } from "../../store/audio-preview";
import { usePlayback } from "../../store/playback";
import { workflowEdits } from "../../store/workflow";
import type { Gesture } from "./gesture";
import { share, type Levels } from "./meter-levels";

const HIGHPASS_Q = 0.7;
const MIN_POWER = 1e-9;

const FILTER_TYPES: Record<EqType, BiquadFilterType> = {
  lowshelf: "lowshelf",
  peak: "peaking",
  highshelf: "highshelf",
};

export type PreviewTarget = { key: string; fraction: () => number };

export type EqGestures = (band?: number) => Gesture<Eq>;

export const MASTER_TARGET: PreviewTarget = { key: "master", fraction: () => 1 };

export const previewTarget = (
  key: string,
  result: MixResult | null,
  levels: Levels,
): PreviewTarget => ({
  key,
  fraction: () => share(result, levels, usePlayback.getState().frame),
});

export const bindChange = <T>(apply: (value: T) => void): Gesture<T> => ({
  begin: () => workflowEdits.beginGesture(),
  set: apply,
  end: () => workflowEdits.endGesture(),
});

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
  const powerRatio = Math.max(MIN_POWER, 1 - fraction + fraction * 10 ** (deltaDb / 10));
  return { gainDb: 10 * Math.log10(powerRatio) };
}

export const bindGain = (
  target: PreviewTarget,
  name: string,
  current: number,
  apply: (v: number) => void,
): Gesture<number> =>
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
): Gesture<number> =>
  bindChange((v) => {
    previewChange(`${target.key}:pan`, current, v, (d) => ({ pan: d * target.fraction() }));
    apply(v);
  });

export const bindHighpass = (
  target: PreviewTarget,
  apply: (freq: number) => void,
): Gesture<number> =>
  bindChange((freq) => {
    useAudioPreview.getState().contribute(`${target.key}:hp`, {
      filters: [{ type: "highpass", freq, gain: 0, q: HIGHPASS_Q }],
    });
    apply(freq);
  });

export const bindEq =
  (target: PreviewTarget, before: Eq, apply: (eq: Eq) => void): EqGestures =>
  (band) =>
    bindChange((eq) => {
      if (band !== undefined) {
        const b = eq.bands[band];
        previewChange(`${target.key}:eq${band}`, before.bands[band].gain, b.gain, (d) => ({
          filters: [
            { type: FILTER_TYPES[b.type], freq: b.freq, gain: d * target.fraction(), q: b.q },
          ],
        }));
      }
      apply(eq);
    });
