import type { Keyframe } from "../schema/timeline";
import { samplePoints } from "../timeline/envelope";
import type { BusPlan, ChannelPlan, MixPlan, SoundPlacement } from "./plan";
import { LOUDNESS_TARGETS, dbToGain, type Comp, type Eq, type Master } from "./types";
import { clamp } from "../math";
import { framesToMs } from "../timeline/time";

export const SAMPLE_RATE = 48_000;

export type LoudnessMeasure = {
  input_i: string;
  input_tp: string;
  input_lra: string;
  input_thresh: string;
  target_offset: string;
};

export type Job = {
  inputs: string[];
  filter: string;
  output: string;
  taps?: { pre: string; post: string };
};

const num = (n: number, digits = 6) => {
  const s = n.toFixed(digits).replace(/\.?0+$/, "");
  return s === "-0" ? "0" : s;
};

const secs = (frames: number, fps: number) => num(frames / fps);

export function envelopeExpr(
  keys: Keyframe[],
  fps: number,
  map: (v: number) => number = (v) => v,
): string {
  const points = samplePoints(keys).map((p) => ({ t: p.frame / fps, v: map(p.value) }));
  if (points.length === 0) return "1";
  if (points.length === 1) return num(points[0].v);
  let expr = num(points[points.length - 1].v);
  for (let i = points.length - 2; i >= 0; i--) {
    const a = points[i];
    const b = points[i + 1];
    const span = b.t - a.t;
    const seg =
      span <= 0 || a.v === b.v
        ? num(a.v)
        : `${num(a.v)}+(${num(b.v - a.v)})*(t-${num(a.t)})/${num(span)}`;
    expr = `if(lt(t,${num(b.t)}),${seg},${expr})`;
  }
  return `if(lt(t,${num(points[0].t)}),${num(points[0].v)},${expr})`;
}

export function soundChain(
  sound: SoundPlacement,
  input: number,
  label: string,
  fps: number,
): string {
  const steps: string[] = [
    `aresample=${SAMPLE_RATE}`,
    "aformat=sample_fmts=fltp:channel_layouts=stereo",
  ];
  if (sound.offset > 0)
    steps.push(`atrim=start=${secs(sound.offset, fps)}`, "asetpts=PTS-STARTPTS");
  if (sound.loop) steps.push("aloop=loop=-1:size=2147483647");
  steps.push(`atrim=end=${secs(sound.length, fps)}`, "asetpts=PTS-STARTPTS");
  if (sound.volume !== 1) steps.push(`volume=${num(sound.volume)}`);
  if (sound.fadeIn > 0)
    steps.push(`afade=t=in:st=0:d=${secs(Math.min(sound.fadeIn, sound.length), fps)}`);
  if (sound.fadeOut > 0) {
    const d = Math.min(sound.fadeOut, sound.length);
    steps.push(`afade=t=out:st=${secs(sound.length - d, fps)}:d=${secs(d, fps)}`);
  }
  const volumeKeys = sound.envelope?.volume;
  if (volumeKeys?.length) steps.push(`volume=eval=frame:volume='${envelopeExpr(volumeKeys, fps)}'`);
  const panKeys = sound.envelope?.pan;
  const delay = framesToMs(sound.at, fps);
  if (!panKeys?.length) {
    if (delay > 0) steps.push(`adelay=delays=${delay}:all=1`);
    return `[${input}:a]${steps.join(",")}[${label}]`;
  }
  const pan = envelopeExpr(panKeys, fps);
  const pre = `[${input}:a]${steps.join(",")},channelsplit=channel_layout=stereo[${label}l][${label}r]`;
  const left = `[${label}l]volume=eval=frame:volume='min(1,1-(${pan}))'[${label}L]`;
  const right = `[${label}r]volume=eval=frame:volume='min(1,1+(${pan}))'[${label}R]`;
  const join = `[${label}L][${label}R]join=inputs=2:channel_layout=stereo${delay > 0 ? `,adelay=delays=${delay}:all=1` : ""}[${label}]`;
  return [pre, left, right, join].join(";");
}

export function eqSteps(eq: Eq): string[] {
  if (!eq.on) return [];
  return eq.bands
    .filter((b) => Math.abs(b.gain) >= 0.05)
    .map((b) => {
      const name = b.type === "peak" ? "equalizer" : b.type;
      return `${name}=f=${num(b.freq, 1)}:t=q:w=${num(b.q, 3)}:g=${num(b.gain, 2)}`;
    });
}

export function compSteps(comp: Comp): string[] {
  if (!comp.on) return [];
  const threshold = clamp(dbToGain(comp.threshold), 0.000976563, 1);
  const makeup = clamp(dbToGain(comp.makeup), 1, 64);
  return [
    `acompressor=threshold=${num(threshold)}:ratio=${num(comp.ratio, 3)}:attack=${num(comp.attack, 3)}:release=${num(comp.release, 3)}:makeup=${num(makeup)}`,
  ];
}

export function panStep(pan: number): string[] {
  if (Math.abs(pan) < 0.005) return [];
  const left = Math.min(1, 1 - pan);
  const right = Math.min(1, 1 + pan);
  return [`pan=stereo|c0=${num(left, 4)}*c0|c1=${num(right, 4)}*c1`];
}

export function preCompSteps(s: ChannelPlan["strip"]): string[] {
  return [
    ...(Math.abs(s.trim) >= 0.01 ? [`volume=${num(s.trim, 2)}dB`] : []),
    ...(s.highpass.on ? [`highpass=f=${num(s.highpass.freq, 1)}`] : []),
    ...eqSteps(s.eq),
  ];
}

export function postCompSteps(s: ChannelPlan["strip"]): string[] {
  return [...panStep(s.pan), ...(Math.abs(s.fader) >= 0.01 ? [`volume=${num(s.fader, 2)}dB`] : [])];
}

export function stripSteps(channel: Pick<ChannelPlan, "strip">): string[] {
  const s = channel.strip;
  return [...preCompSteps(s), ...compSteps(s.comp), ...postCompSteps(s)];
}

const PINNED = `aformat=sample_fmts=fltp:sample_rates=${SAMPLE_RATE}:channel_layouts=stereo`;

function tapped(sum: string, comp: string[], post: string[]): string {
  const tail = post.length ? post.join(",") : "anull";
  return `[${sum}]${PINNED},asplit=2[pre][c];[c]${comp.join(",")},${PINNED},asplit=2[post][m];[m]${tail}[out]`;
}

const silence = (total: number, fps: number) =>
  `anullsrc=r=${SAMPLE_RATE}:cl=stereo,atrim=end=${secs(total, fps)}`;

const fit = (total: number, fps: number) =>
  `apad,atrim=end=${secs(total, fps)},asetpts=PTS-STARTPTS`;

function mixOf(labels: string[], out: string, then: string[]): string {
  const tail = then.length ? `,${then.join(",")}` : "";
  if (labels.length === 1) return `[${labels[0]}]anull${tail}[${out}]`;
  return `${labels.map((l) => `[${l}]`).join("")}amix=inputs=${labels.length}:normalize=0:duration=longest:dropout_transition=0${tail}[${out}]`;
}

export function channelJob(
  plan: Pick<MixPlan, "fps" | "durationInFrames">,
  channel: Pick<ChannelPlan, "sounds" | "strip">,
  resolve: (ref: string) => string,
): Job {
  const { fps, durationInFrames: total } = plan;
  if (channel.sounds.length === 0)
    return { inputs: [], filter: `${silence(total, fps)}[out]`, output: "out" };
  const chains = channel.sounds.map((s, i) => soundChain(s, i, `s${i}`, fps));
  const labels = channel.sounds.map((_, i) => `s${i}`);
  const inputs = channel.sounds.map((s) => resolve(s.src));
  const comp = compSteps(channel.strip.comp);
  if (comp.length === 0) {
    const mixed = mixOf(labels, "out", [fit(total, fps), ...stripSteps(channel)]);
    return { inputs, filter: [...chains, mixed].join(";"), output: "out" };
  }
  const mixed = mixOf(labels, "sum", [fit(total, fps), ...preCompSteps(channel.strip)]);
  return {
    inputs,
    filter: [...chains, mixed, tapped("sum", comp, postCompSteps(channel.strip))].join(";"),
    output: "out",
    taps: { pre: "pre", post: "post" },
  };
}

export function busJob(
  plan: Pick<MixPlan, "fps" | "durationInFrames">,
  bus: BusPlan,
  stem: (channelId: string) => string,
): Job {
  const { fps, durationInFrames: total } = plan;
  const fader = Math.abs(bus.fader) >= 0.01 ? [`volume=${num(bus.fader, 2)}dB`] : [];
  const proc = bus.processing ? busSteps(bus.processing) : [];
  if (bus.channels.length === 0)
    return { inputs: [], filter: `${silence(total, fps)}[out]`, output: "out" };
  const inputs = bus.channels.map(stem);
  const labels = inputs.map((_, i) => `${i}:a`);
  if (!bus.duck)
    return {
      inputs,
      filter: mixOf(labels, "out", [fit(total, fps), ...proc, ...fader]),
      output: "out",
    };
  const keyInputs = bus.duck.keys.map(stem);
  const keyLabels = keyInputs.map((_, i) => `${inputs.length + i}:a`);
  const ratio = duckRatio(bus.duck.amount);
  const sidechain = `sidechaincompress=threshold=${num(dbToGain(DUCK_THRESHOLD_DB))}:ratio=${num(ratio, 3)}:attack=${num(bus.duck.attack, 2)}:release=${num(bus.duck.release, 2)}:makeup=1`;
  return {
    inputs: [...inputs, ...keyInputs],
    filter: [
      mixOf(labels, "main", [fit(total, fps), ...proc, ...fader]),
      mixOf(keyLabels, "key", [fit(total, fps)]),
      `[main][key]${sidechain},${fit(total, fps)}[out]`,
    ].join(";"),
    output: "out",
  };
}

export function busSteps(p: NonNullable<BusPlan["processing"]>): string[] {
  return [
    ...(p.highpass.on ? [`highpass=f=${num(p.highpass.freq, 1)}`] : []),
    ...eqSteps(p.eq),
    ...compSteps(p.comp),
  ];
}

export const DUCK_THRESHOLD_DB = -40;
export const DUCK_REFERENCE_DB = -12;

export function duckRatio(amount: number): number {
  const range = DUCK_REFERENCE_DB - DUCK_THRESHOLD_DB;
  const a = clamp(amount, 0.5, range - 1);
  return Math.min(20, 1 / (1 - a / range));
}

function masterTail(master: Master, measured?: LoudnessMeasure | null): string[] {
  const steps = Math.abs(master.fader) >= 0.01 ? [`volume=${num(master.fader, 2)}dB`] : [];
  const target = LOUDNESS_TARGETS[master.loudness];
  if (target && measured)
    steps.push(
      `loudnorm=I=${target.lufs}:TP=${target.truePeak}:LRA=11:measured_I=${measured.input_i}:measured_TP=${measured.input_tp}:measured_LRA=${measured.input_lra}:measured_thresh=${measured.input_thresh}:offset=${measured.target_offset}:linear=true:print_format=summary`,
      `aresample=${SAMPLE_RATE}`,
    );
  if (master.limiter.on)
    steps.push(
      `aresample=${SAMPLE_RATE * 4}`,
      `alimiter=limit=${num(Math.max(0.0625, dbToGain(master.limiter.ceiling)))}:level=false`,
      `aresample=${SAMPLE_RATE}`,
    );
  return steps;
}

export function masterSteps(master: Master, measured?: LoudnessMeasure | null): string[] {
  return [...eqSteps(master.eq), ...compSteps(master.comp), ...masterTail(master, measured)];
}

export function masterJob(
  plan: Pick<MixPlan, "fps" | "durationInFrames" | "master">,
  buses: string[],
  measured: LoudnessMeasure | null,
): Job {
  const { fps, durationInFrames: total } = plan;
  if (buses.length === 0)
    return { inputs: [], filter: `${silence(total, fps)}[out]`, output: "out" };
  const labels = buses.map((_, i) => `${i}:a`);
  const comp = compSteps(plan.master.comp);
  if (comp.length === 0)
    return {
      inputs: buses,
      filter: mixOf(labels, "out", [fit(total, fps), ...masterSteps(plan.master, measured)]),
      output: "out",
    };
  return {
    inputs: buses,
    filter: [
      mixOf(labels, "sum", [fit(total, fps), ...eqSteps(plan.master.eq)]),
      tapped("sum", comp, masterTail(plan.master, measured)),
    ].join(";"),
    output: "out",
    taps: { pre: "pre", post: "post" },
  };
}

export function loudnessMeasureJob(
  plan: Pick<MixPlan, "fps" | "durationInFrames" | "master">,
  buses: string[],
): Job | null {
  const target = LOUDNESS_TARGETS[plan.master.loudness];
  if (!target || buses.length === 0) return null;
  const { fps, durationInFrames: total } = plan;
  const labels = buses.map((_, i) => `${i}:a`);
  const pre = masterSteps({ ...plan.master, loudness: "off", limiter: { on: false, ceiling: 0 } });
  return {
    inputs: buses,
    filter: mixOf(labels, "out", [
      fit(total, fps),
      ...pre,
      `loudnorm=I=${target.lufs}:TP=${target.truePeak}:LRA=11:print_format=json`,
    ]),
    output: "out",
  };
}

export function jobArgs(
  job: Job,
  output: string,
  codec: "pcm_f32le" | "pcm_s16le",
  taps?: { pre: string; post: string },
  maxSeconds?: number,
): string[] {
  const limit = maxSeconds ? ["-t", num(maxSeconds, 3)] : [];
  const tap = (label: string, file: string) => [
    "-map",
    `[${label}]`,
    ...limit,
    "-ar",
    String(SAMPLE_RATE),
    "-c:a",
    "pcm_f32le",
    file,
  ];
  const tapArgs =
    job.taps && taps ? [...tap(job.taps.pre, taps.pre), ...tap(job.taps.post, taps.post)] : [];
  return [
    ...job.inputs.flatMap((file) => ["-i", file]),
    "-filter_complex",
    job.filter,
    "-map",
    `[${job.output}]`,
    ...limit,
    "-ac",
    "2",
    "-ar",
    String(SAMPLE_RATE),
    "-c:a",
    codec,
    output,
    ...tapArgs,
  ];
}

export function measureArgs(job: Job): string[] {
  return [
    ...job.inputs.flatMap((file) => ["-i", file]),
    "-filter_complex",
    job.filter,
    "-map",
    `[${job.output}]`,
    "-f",
    "null",
    "-",
  ];
}

export function parseLoudnorm(stderr: string): LoudnessMeasure | null {
  const start = stderr.lastIndexOf("{");
  const end = stderr.lastIndexOf("}");
  if (start === -1 || end < start) return null;
  try {
    const json = JSON.parse(stderr.slice(start, end + 1)) as Record<string, string>;
    if (!json.input_i || json.input_i === "-inf") return null;
    return {
      input_i: json.input_i,
      input_tp: json.input_tp,
      input_lra: json.input_lra,
      input_thresh: json.input_thresh,
      target_offset: json.target_offset,
    };
  } catch {
    return null;
  }
}
