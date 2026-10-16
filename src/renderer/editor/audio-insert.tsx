import { useContext, useEffect, useRef } from "react";
import { Internals } from "remotion";
import type { TimelineSpec } from "@/shared/schema/timeline";
import { DemoTimeline } from "@/video/demo-timeline";
import { useAudioPreview } from "../store/audio-preview";
import { clamp } from "@/shared/math";

type Chain = {
  filters: BiquadFilterNode[];
  panner: StereoPannerNode;
  gain: GainNode;
};

const MAX_FILTERS = 6;

function AudioInsert() {
  const shared = useContext(Internals.SharedAudioContext);
  const chain = useRef<Chain | null>(null);
  const ac = shared?.audioContext ?? null;
  const master = shared?.gainNode ?? null;

  useEffect(() => {
    if (!ac || !master) return;
    const filters = Array.from({ length: MAX_FILTERS }, () => {
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
    chain.current = { filters, panner, gain };
    const apply = () => {
      const { gainDb, pan, filters: wanted } = useAudioPreview.getState();
      const now = ac.currentTime;
      gain.gain.setTargetAtTime(10 ** (gainDb / 20), now, 0.02);
      panner.pan.setTargetAtTime(clamp(pan, -1, 1), now, 0.02);
      filters.forEach((node, i) => {
        const spec = wanted[i];
        if (!spec) {
          node.type = "peaking";
          node.gain.setTargetAtTime(0, now, 0.02);
          return;
        }
        node.type = spec.type;
        node.frequency.setTargetAtTime(spec.freq, now, 0.02);
        node.Q.setTargetAtTime(spec.q, now, 0.02);
        node.gain.setTargetAtTime(spec.gain, now, 0.02);
      });
    };
    apply();
    const unsubscribe = useAudioPreview.subscribe(apply);
    return () => {
      unsubscribe();
      restore(master, ac);
      for (const n of nodes) n.disconnect();
      chain.current = null;
    };
  }, [ac, master]);

  return null;
}

function restore(master: GainNode, ac: AudioContext): void {
  try {
    master.disconnect();
    master.connect(ac.destination);
  } catch {
    return;
  }
}

export const PreviewComposition: React.FC<TimelineSpec> = (spec) => (
  <>
    <DemoTimeline {...spec} />
    <AudioInsert />
  </>
);
