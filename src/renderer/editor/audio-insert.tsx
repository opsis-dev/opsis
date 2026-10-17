import { useContext, useEffect } from "react";
import { Internals } from "remotion";
import { clamp } from "@/shared/math";
import { dbToGain } from "@/shared/mix/types";
import type { TimelineSpec } from "@/shared/schema/timeline";
import { DemoTimeline } from "@/video/demo-timeline";
import { MAX_PREVIEW_FILTERS, previewTotals, useAudioPreview } from "../store/audio-preview";

const SMOOTHING_S = 0.02;

type Chain = { filters: BiquadFilterNode[]; panner: StereoPannerNode; gain: GainNode };

function buildChain(ac: AudioContext, master: GainNode): Chain {
  const filters = Array.from({ length: MAX_PREVIEW_FILTERS }, () => {
    const f = ac.createBiquadFilter();
    f.type = "peaking";
    f.gain.value = 0;
    return f;
  });
  const panner = ac.createStereoPanner();
  const gain = ac.createGain();
  const nodes: AudioNode[] = [...filters, panner, gain];
  for (let i = 0; i < nodes.length - 1; i++) nodes[i].connect(nodes[i + 1]);
  master.disconnect();
  master.connect(filters[0]);
  gain.connect(ac.destination);
  return { filters, panner, gain };
}

function applyPreview(chain: Chain, now: number): void {
  const { gainDb, pan, filters: wanted } = previewTotals(useAudioPreview.getState());
  chain.gain.gain.setTargetAtTime(dbToGain(gainDb), now, SMOOTHING_S);
  chain.panner.pan.setTargetAtTime(clamp(pan, -1, 1), now, SMOOTHING_S);
  chain.filters.forEach((node, i) => {
    const spec = wanted[i];
    if (!spec) {
      node.type = "peaking";
      node.gain.setTargetAtTime(0, now, SMOOTHING_S);
      return;
    }
    node.type = spec.type;
    node.frequency.setTargetAtTime(spec.freq, now, SMOOTHING_S);
    node.Q.setTargetAtTime(spec.q, now, SMOOTHING_S);
    node.gain.setTargetAtTime(spec.gain, now, SMOOTHING_S);
  });
}

function restore(master: GainNode, ac: AudioContext): void {
  master.disconnect();
  master.connect(ac.destination);
}

function AudioInsert() {
  const shared = useContext(Internals.SharedAudioContext);
  const ac = shared?.audioContext ?? null;
  const master = shared?.gainNode ?? null;

  useEffect(() => {
    if (!ac || !master) return;
    const chain = buildChain(ac, master);
    const apply = () => applyPreview(chain, ac.currentTime);
    apply();
    const unsubscribe = useAudioPreview.subscribe(apply);
    return () => {
      unsubscribe();
      restore(master, ac);
      for (const n of [...chain.filters, chain.panner, chain.gain]) n.disconnect();
    };
  }, [ac, master]);

  return null;
}

export const PreviewComposition: React.FC<TimelineSpec> = (spec) => (
  <>
    <DemoTimeline {...spec} />
    <AudioInsert />
  </>
);
