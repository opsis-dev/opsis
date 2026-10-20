import { z } from "zod";
import { parseJsonWith } from "../guards";
import { clamp } from "../math";
import type { Keyframe } from "../schema/timeline";
import { samplePoints } from "../timeline/envelope";
import { framesToMs } from "../timeline/time";
import type { BusPlan, ChannelPlan, MixPlan, SoundPlacement } from "./plan";
import { atempoChain } from "./tempo";
import { LOUDNESS_TARGETS, SAMPLE_RATE, dbToGain, type Comp, type Eq, type Master } from "./types";

const MIN_COMP_THRESHOLD = 0.000976563;
const MAX_COMP_MAKEUP = 64;
const MIN_LIMITER_CEILING = 0.0625;
const LIMITER_OVERSAMPLING = 4;
const LOUDNESS_RANGE = 11;
const ENDLESS_LOOP = 2147483647;
const MIN_LEVEL_DB = 0.01;
const MIN_EQ_GAIN_DB = 0.05;
const MIN_PAN = 0.005;
const DUCK_THRESHOLD_DB = -40;
const DUCK_REFERENCE_DB = -12;
const DUCK_RANGE_DB = DUCK_REFERENCE_DB - DUCK_THRESHOLD_DB;
const MIN_DUCK_AMOUNT_DB = 0.5;
const MAX_DUCK_AMOUNT_DB = DUCK_RANGE_DB - 1;
const MAX_DUCK_RATIO = 20;

const loudnessMeasureSchema = z.object({
  input_i: z.string().min(1),
  input_tp: z.string(),
  input_lra: z.string(),
  input_thresh: z.string(),
  target_offset: z.string(),
});

type LoudnessMeasure = z.infer<typeof loudnessMeasureSchema>;

type Taps = { pre: string; post: string };

export type Job = {
  inputs: string[];
  filter: string;
  output: string;
  taps?: Taps;
};

type Plan = Pick<MixPlan, "fps" | "durationInFrames">;

type MasterPlan = Pick<MixPlan, "fps" | "durationInFrames" | "master">;

const STEM_TAPS: Taps = { pre: "pre", post: "post" };

const DIGITS = {
  gainDb: 2,
  freq: 1,
  q: 3,
  pan: 4,
  ratio: 3,
  compTime: 3,
  duckTime: 2,
  seconds: 3,
} as const;

const num = (n: number, digits = 6) => {
  const s = n.toFixed(digits).replace(/\.?0+$/, "");
  return s === "-0" ? "0" : s;
};

const secs = (frames: number, fps: number) => num(frames / fps);

const dbStep = (db: number): string[] =>
  Math.abs(db) >= MIN_LEVEL_DB ? [`volume=${num(db, DIGITS.gainDb)}dB`] : [];

function envelopeExpr(keys: Keyframe[], fps: number): string {
  const points = samplePoints(keys).map((p) => ({ t: p.frame / fps, v: p.value }));
  const last = points.at(-1);
  if (!last) return "1";
  if (points.length === 1) return num(last.v);
  let expr = num(last.v);
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

const LEFT = -1;
const RIGHT = 1;

type Side = typeof LEFT | typeof RIGHT;

const sideGain = (side: Side, pan: number): string => num(Math.min(1, 1 + side * pan), DIGITS.pan);

const sideGainExpr = (side: Side, pan: string): string =>
  `min(1,1${side === LEFT ? "-" : "+"}(${pan}))`;

function panChain({
  label,
  steps,
  pan,
  after,
}: {
  label: string;
  steps: string[];
  pan: string;
  after: string[];
}): string {
  const [left, right, leftOut, rightOut] = ["l", "r", "L", "R"].map((side) => `${label}${side}`);
  return [
    `${steps.join(",")},channelsplit=channel_layout=stereo[${left}][${right}]`,
    `[${left}]volume=eval=frame:volume='${sideGainExpr(LEFT, pan)}'[${leftOut}]`,
    `[${right}]volume=eval=frame:volume='${sideGainExpr(RIGHT, pan)}'[${rightOut}]`,
    `[${leftOut}][${rightOut}]${["join=inputs=2:channel_layout=stereo", ...after].join(",")}[${label}]`,
  ].join(";");
}

function placementSteps(sound: SoundPlacement, fps: number): string[] {
  const steps: string[] = [
    `aresample=${SAMPLE_RATE}`,
    "aformat=sample_fmts=fltp:channel_layouts=stereo",
  ];
  if (sound.offset > 0)
    steps.push(`atrim=start=${secs(sound.offset, fps)}`, "asetpts=PTS-STARTPTS");
  if (sound.loop) steps.push(`aloop=loop=-1:size=${ENDLESS_LOOP}`);
  if (sound.rate !== 1) steps.push(atempoChain(sound.rate), "asetpts=PTS-STARTPTS");
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
  return steps;
}

function soundChain(sound: SoundPlacement, input: number, label: string, fps: number): string {
  const steps = placementSteps(sound, fps);
  const delay = framesToMs(sound.at, fps);
  const after = delay > 0 ? [`adelay=delays=${delay}:all=1`] : [];
  const panKeys = sound.envelope?.pan;
  const chain = panKeys?.length
    ? panChain({ label, steps, pan: envelopeExpr(panKeys, fps), after })
    : `${[...steps, ...after].join(",")}[${label}]`;
  return `[${input}:a]${chain}`;
}

function eqSteps(eq: Eq): string[] {
  if (!eq.on) return [];
  return eq.bands
    .filter((b) => Math.abs(b.gain) >= MIN_EQ_GAIN_DB)
    .map((b) => {
      const name = b.type === "peak" ? "equalizer" : b.type;
      return `${name}=f=${num(b.freq, DIGITS.freq)}:t=q:w=${num(b.q, DIGITS.q)}:g=${num(b.gain, DIGITS.gainDb)}`;
    });
}

function compSteps(comp: Comp): string[] {
  if (!comp.on) return [];
  const threshold = clamp(dbToGain(comp.threshold), MIN_COMP_THRESHOLD, 1);
  const makeup = clamp(dbToGain(comp.makeup), 1, MAX_COMP_MAKEUP);
  const options = [
    `threshold=${num(threshold)}`,
    `ratio=${num(comp.ratio, DIGITS.ratio)}`,
    `attack=${num(comp.attack, DIGITS.compTime)}`,
    `release=${num(comp.release, DIGITS.compTime)}`,
    `makeup=${num(makeup)}`,
  ];
  return [`acompressor=${options.join(":")}`];
}

function panStep(pan: number): string[] {
  if (Math.abs(pan) < MIN_PAN) return [];
  return [`pan=stereo|c0=${sideGain(LEFT, pan)}*c0|c1=${sideGain(RIGHT, pan)}*c1`];
}

const highpassStep = (highpass: { on: boolean; freq: number }): string[] =>
  highpass.on ? [`highpass=f=${num(highpass.freq, DIGITS.freq)}`] : [];

type Stages = { pre: string[]; comp: string[]; post: string[] };

const stripStages = (s: ChannelPlan["strip"]): Stages => ({
  pre: [...dbStep(s.trim), ...highpassStep(s.highpass), ...eqSteps(s.eq)],
  comp: compSteps(s.comp),
  post: [...panStep(s.pan), ...dbStep(s.fader)],
});

function busSteps(p: BusPlan["processing"]): string[] {
  return [...highpassStep(p.highpass), ...eqSteps(p.eq), ...compSteps(p.comp)];
}

export const loudnormFilter = (lufs: number, truePeak: number, range: number): string =>
  `loudnorm=I=${lufs}:TP=${truePeak}:LRA=${range}`;

const loudnormBase = (target: { lufs: number; truePeak: number }) =>
  loudnormFilter(target.lufs, target.truePeak, LOUDNESS_RANGE);

function masterTail(master: Master, measured?: LoudnessMeasure | null): string[] {
  const steps = dbStep(master.fader);
  const target = LOUDNESS_TARGETS[master.loudness];
  if (target && measured) {
    const options = [
      loudnormBase(target),
      `measured_I=${measured.input_i}`,
      `measured_TP=${measured.input_tp}`,
      `measured_LRA=${measured.input_lra}`,
      `measured_thresh=${measured.input_thresh}`,
      `offset=${measured.target_offset}`,
      "linear=true",
      "print_format=summary",
    ];
    steps.push(options.join(":"), `aresample=${SAMPLE_RATE}`);
  }
  if (master.limiter.on)
    steps.push(
      `aresample=${SAMPLE_RATE * LIMITER_OVERSAMPLING}`,
      `alimiter=limit=${num(Math.max(MIN_LIMITER_CEILING, dbToGain(master.limiter.ceiling)))}:level=false`,
      `aresample=${SAMPLE_RATE}`,
    );
  return steps;
}

const masterStages = (master: Master, measured?: LoudnessMeasure | null): Stages => ({
  pre: eqSteps(master.eq),
  comp: compSteps(master.comp),
  post: masterTail(master, measured),
});

const PINNED = `aformat=sample_fmts=fltp:sample_rates=${SAMPLE_RATE}:channel_layouts=stereo`;

function tapped(sum: string, comp: string[], post: string[]): string {
  const tail = post.length ? post.join(",") : "anull";
  const split = `[${sum}]${PINNED},asplit=2[${STEM_TAPS.pre}][c]`;
  const compressed = `[c]${comp.join(",")},${PINNED},asplit=2[${STEM_TAPS.post}][m]`;
  const finished = `[m]${tail}[out]`;
  return [split, compressed, finished].join(";");
}

const silenceJob = ({ fps, durationInFrames }: Plan): Job => ({
  inputs: [],
  filter: `anullsrc=r=${SAMPLE_RATE}:cl=stereo,atrim=end=${secs(durationInFrames, fps)}[out]`,
  output: "out",
});

const fit = ({ fps, durationInFrames }: Plan) =>
  `apad,atrim=end=${secs(durationInFrames, fps)},asetpts=PTS-STARTPTS`;

function mixOf(labels: string[], out: string, then: string[]): string {
  const tail = then.length ? `,${then.join(",")}` : "";
  if (labels.length === 1) return `[${labels[0]}]anull${tail}[${out}]`;
  const amix = `amix=inputs=${labels.length}:normalize=0:duration=longest:dropout_transition=0`;
  return `${labels.map((l) => `[${l}]`).join("")}${amix}${tail}[${out}]`;
}

const streamLabels = (inputs: string[], first = 0) => inputs.map((_, i) => `${first + i}:a`);

function stripJob(
  plan: Plan,
  inputs: string[],
  sources: { chains: string[]; labels: string[] },
  { pre, comp, post }: Stages,
): Job {
  if (comp.length === 0) {
    const mixed = mixOf(sources.labels, "out", [fit(plan), ...pre, ...post]);
    return { inputs, filter: [...sources.chains, mixed].join(";"), output: "out" };
  }
  const mixed = mixOf(sources.labels, "sum", [fit(plan), ...pre]);
  return {
    inputs,
    filter: [...sources.chains, mixed, tapped("sum", comp, post)].join(";"),
    output: "out",
    taps: { ...STEM_TAPS },
  };
}

export function channelJob(
  plan: Plan,
  channel: Pick<ChannelPlan, "sounds" | "strip">,
  resolve: (ref: string) => string,
): Job {
  if (channel.sounds.length === 0) return silenceJob(plan);
  const labels = channel.sounds.map((_, i) => `s${i}`);
  const chains = channel.sounds.map((s, i) => soundChain(s, i, labels[i], plan.fps));
  return stripJob(
    plan,
    channel.sounds.map((s) => resolve(s.src)),
    { chains, labels },
    stripStages(channel.strip),
  );
}

function duckRatio(amount: number): number {
  const a = clamp(amount, MIN_DUCK_AMOUNT_DB, MAX_DUCK_AMOUNT_DB);
  return Math.min(MAX_DUCK_RATIO, 1 / (1 - a / DUCK_RANGE_DB));
}

function sidechain(duck: NonNullable<BusPlan["duck"]>): string {
  const options = [
    `threshold=${num(dbToGain(DUCK_THRESHOLD_DB))}`,
    `ratio=${num(duckRatio(duck.amount), DIGITS.ratio)}`,
    `attack=${num(duck.attack, DIGITS.duckTime)}`,
    `release=${num(duck.release, DIGITS.duckTime)}`,
    "makeup=1",
  ];
  return `sidechaincompress=${options.join(":")}`;
}

export function busJob(plan: Plan, bus: BusPlan, stem: (channelId: string) => string): Job {
  if (bus.channels.length === 0) return silenceJob(plan);
  const inputs = bus.channels.map(stem);
  const main = [fit(plan), ...busSteps(bus.processing), ...dbStep(bus.fader)];
  if (!bus.duck) return { inputs, filter: mixOf(streamLabels(inputs), "out", main), output: "out" };
  const keyInputs = bus.duck.keys.map(stem);
  return {
    inputs: [...inputs, ...keyInputs],
    filter: [
      mixOf(streamLabels(inputs), "main", main),
      mixOf(streamLabels(keyInputs, inputs.length), "key", [fit(plan)]),
      `[main][key]${sidechain(bus.duck)},${fit(plan)}[out]`,
    ].join(";"),
    output: "out",
  };
}

export function masterJob(
  plan: MasterPlan,
  buses: string[],
  measured: LoudnessMeasure | null,
): Job {
  if (buses.length === 0) return silenceJob(plan);
  return stripJob(
    plan,
    buses,
    { chains: [], labels: streamLabels(buses) },
    masterStages(plan.master, measured),
  );
}

export function loudnessMeasureJob(plan: MasterPlan, buses: string[]): Job | null {
  const target = LOUDNESS_TARGETS[plan.master.loudness];
  if (!target || buses.length === 0) return null;
  const { pre, comp, post } = masterStages({
    ...plan.master,
    loudness: "off",
    limiter: { on: false, ceiling: 0 },
  });
  return {
    inputs: buses,
    filter: mixOf(streamLabels(buses), "out", [
      fit(plan),
      ...pre,
      ...comp,
      ...post,
      `${loudnormBase(target)}:print_format=json`,
    ]),
    output: "out",
  };
}

const inputArgs = (job: Job) => [
  ...job.inputs.flatMap((file) => ["-i", file]),
  "-filter_complex",
  job.filter,
];

const outputArgs = (label: string, limit: string[], format: string[], file: string) => [
  "-map",
  `[${label}]`,
  ...limit,
  ...format,
  file,
];

type PcmCodec = "pcm_f32le" | "pcm_s16le";

const TAP_CODEC: PcmCodec = "pcm_f32le";

const stereoPcm = (codec: PcmCodec) => ["-ac", "2", "-ar", String(SAMPLE_RATE), "-c:a", codec];

export function jobArgs(
  job: Job,
  output: string,
  codec: PcmCodec,
  taps?: Taps,
  maxSeconds?: number,
): string[] {
  const limit = maxSeconds ? ["-t", num(maxSeconds, DIGITS.seconds)] : [];
  const stem = (label: string, file: string, format: PcmCodec) =>
    outputArgs(label, limit, stereoPcm(format), file);
  const tapStems =
    taps && job.taps
      ? [...stem(job.taps.pre, taps.pre, TAP_CODEC), ...stem(job.taps.post, taps.post, TAP_CODEC)]
      : [];
  return [...inputArgs(job), ...stem(job.output, output, codec), ...tapStems];
}

export function measureArgs(job: Job): string[] {
  return [...inputArgs(job), ...outputArgs(job.output, [], ["-f", "null"], "-")];
}

export function parseLoudnorm(stderr: string): LoudnessMeasure | null {
  const start = stderr.lastIndexOf("{");
  const end = stderr.lastIndexOf("}");
  if (start === -1 || end < start) return null;
  const parsed = parseJsonWith(loudnessMeasureSchema, stderr.slice(start, end + 1));
  if (!parsed?.success || parsed.data.input_i === "-inf") return null;
  return parsed.data;
}
