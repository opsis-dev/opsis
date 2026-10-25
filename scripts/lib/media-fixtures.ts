import { compileWorkflow } from "@opsis/core/compile/compile";
import {
  busJob,
  channelJob,
  jobArgs,
  loudnessMeasureJob,
  loudnormFilter,
  masterJob,
  measureArgs,
  parseLoudnorm,
} from "@opsis/core/mix/graph";
import { buildMixPlan, type MixPlan } from "@opsis/core/mix/plan";
import { atempoChain } from "@opsis/core/mix/tempo";
import { mixSchema, STRIP_PRESETS } from "@opsis/core/mix/types";
import { defaultParams } from "@opsis/core/nodes/registry";
import type { Workflow } from "@opsis/core/schema/workflow";
import { detectBeats, waveformLevels } from "../../src/worker/media/analysis";
import {
  AUDIO_FORMATS,
  audioFormatFor,
  CODECS,
  convertArgs,
  editedFile,
  ffmpegSeconds,
  frameGrabArgs,
  gradeSoundArgs,
  gradeVideoArgs,
  h264Args,
  soundFile,
} from "../../src/worker/media/codecs";
import { canStreamCopy, parseProbe } from "../../src/worker/media/ffmpeg";
import {
  ffmpegColor,
  fitFilter,
  gifFilter,
  joinFilter,
  speedArgs,
  speedSoundArgs,
} from "../../src/worker/media/filters";
import { filterPath, gradeFilters, lookKindOf, type GradeSize } from "../../src/worker/media/grade";
import {
  frameBytes,
  joinReversedArgs,
  pieceLengthMs,
  reverseArgs,
  reversedPartsList,
  reversePieces,
  reverseSoundArgs,
} from "../../src/worker/media/reverse";
import { friendlyMixError } from "../../src/worker/mix/render";
import { frameLevels } from "../../src/worker/mix/wav";
import { finishArgs, muxArgs } from "../../src/worker/render/picture";

const CONVERT = ["h264", "h265", "vp9", "prores"] as const;
const PRESETS = ["ultrafast", "fast", "medium", "slow", "veryslow"] as const;
const SOUNDS = [undefined, "aac", "mp3", "opus", "vorbis", "pcm_s16le", "flac", "alac"];
const EXTS = [".mp4", ".webm", ".mov", ".mp3", ".wav", ".m4a", ".ogg", ".flac", ".gif", ".png", ""];

function codecCases() {
  return {
    table: Object.fromEntries(
      Object.entries(CODECS).map(([name, c]) => [
        name,
        {
          ext: c.ext,
          audio: c.audio,
          faststart: c.faststart,
          crf: c.crf,
          video: c.video ? PRESETS.map((p) => c.video(23, p)) : null,
        },
      ]),
    ),
    convert: CONVERT.flatMap((codec) =>
      PRESETS.flatMap((preset) =>
        [18, 23.5].map((crf) => ({
          codec,
          preset,
          crf,
          convert: convertArgs(codec, crf, preset),
          gradeVideo: gradeVideoArgs(codec, crf, preset),
        })),
      ),
    ),
    gradeSound: CONVERT.flatMap((codec) =>
      SOUNDS.map((sound) => ({ codec, sound: sound ?? null, args: gradeSoundArgs(codec, sound) })),
    ),
    h264: [
      { crf: 18 },
      { crf: 20, preset: "slow" },
      { crf: 23, keyframeEvery: 15 },
      { crf: 30, preset: "fast", keyframeEvery: 30, faststart: true },
    ].map((opts) => ({ opts, args: h264Args(opts) })),
    edited: EXTS.flatMap((ext) =>
      [
        { video: true, audio: true },
        { video: true, audio: false },
        { video: false, audio: true },
        { video: false, audio: false },
      ].map((streams) => ({ ext, streams, out: editedFile(ext, streams) })),
    ),
    soundFiles: EXTS.map((ext) => ({ ext, out: soundFile(ext) })),
    audioFormats: AUDIO_FORMATS,
    audioFormatFor: ["m4a", "mp3", "wav", "ogg", "", "MP3"].map((ext) => ({
      ext,
      format: audioFormatFor(ext),
    })),
    seconds: [0, 1, 33.4, 1500, 1234.5, 0.5, 2.5, 999999.5, 60000].map((ms) => ({
      ms,
      text: ffmpegSeconds(ms),
    })),
    frameGrabs: [
      { atMs: 0, width: 320 },
      { atMs: 1234.5, width: 640, evenHeight: true },
      { atMs: 5000, width: 960, jpegQuality: 4 },
    ].map((opts) => ({ opts, args: frameGrabArgs("in.mp4", "out.jpg", opts) })),
    mux: (["h264", "h265", "vp8", "vp9", "prores"] as const).map((codec) => ({
      codec,
      args: muxArgs(codec, "p.mp4", "m.wav", "o.mp4"),
    })),
    finish: [
      { faststart: true, chapters: "c.txt" },
      { faststart: true, chapters: null },
      { faststart: false, chapters: "c.txt" },
      { faststart: false, chapters: null },
    ].map((opts) => ({ opts, args: finishArgs("a.mp4", "b.mp4", opts) })),
  };
}

const COLOURS = [
  "black",
  "white",
  "",
  "rgb(255, 0, 0)",
  "rgb(1, 2, 3)",
  "hsl(120, 100%, 50%)",
  "oklch(0.628 0.258 29.2)",
  "#00000080",
  "#abc",
  "transparent",
  "not a colour",
  " #FF8800 ",
];

function filterCases() {
  return {
    colours: COLOURS.map((color) => ({ color, out: ffmpegColor(color) })),
    fit: (["pad", "crop", "stretch"] as const).flatMap((mode) =>
      COLOURS.slice(0, 5).map((background) => {
        const p = { mode, width: 641, height: 360, background };
        return { p, out: fitFilter(p) };
      }),
    ),
    gif: [
      { fps: 15, width: 960, dither: "bayer" },
      { fps: 10, width: 480, dither: "none" },
      { fps: 24, width: 1280, dither: "floyd_steinberg" },
    ].map((p) => ({ p, out: gifFilter(p as Parameters<typeof gifFilter>[0]) })),
    join: [
      [
        [
          { video: true, audio: true, durationMs: 2000 },
          { video: true, audio: true, durationMs: 2000 },
        ],
        { width: 1920, height: 1080, fps: 60 },
      ],
      [
        [
          { video: true, audio: true, durationMs: 2000 },
          { video: true, audio: false, durationMs: 1234 },
        ],
        { width: 1920, height: 1080, fps: 29.97003 },
      ],
      [
        [
          { video: false, audio: true, durationMs: 1500 },
          { video: true, audio: true, durationMs: 2000 },
        ],
        { width: 1920, height: 1080, fps: 60 },
      ],
      [
        [
          { video: false, audio: true, durationMs: 1000 },
          { video: false, audio: true, durationMs: 1000 },
        ],
        { width: 1920, height: 1080, fps: 30 },
      ],
      [
        [
          { video: true, audio: false, durationMs: 1000 },
          { video: true, audio: false, durationMs: 1000.5 },
          { video: true, audio: false, durationMs: 3000 },
        ],
        { width: 321, height: 241, fps: 30 },
      ],
    ].map(([parts, to]) => ({
      parts,
      to,
      out: joinFilter(
        parts as Parameters<typeof joinFilter>[0],
        to as Parameters<typeof joinFilter>[1],
      ),
    })),
    speed: [
      { rate: 64, fps: 29.97003, sound: false },
      { rate: 2, fps: 30, sound: true },
      { rate: 0.25, fps: 60, sound: true },
    ].map((opts) => ({ opts, args: speedArgs("in.mp4", "out.mp4", opts) })),
    speedSound: [0.25, 1.5, 3, 64].map((rate) => ({
      rate,
      args: speedSoundArgs("in.mp3", "out.mp3", { rate, codecs: ["-vn", "-c:a", "aac"] }),
    })),
    atempo: [64, 0.25, 3, 1, 1.5, 0.1, 1.000049, 2.00005, 0.3, 7].map((rate) => ({
      rate,
      chain: atempoChain(rate),
    })),
    loudnorm: [
      [-14, -1, 11],
      [-23, -2, 7],
      [-16.5, -1.5, 10],
    ].map(([lufs, truePeak, range]) => ({
      lufs,
      truePeak,
      range,
      filter: loudnormFilter(lufs, truePeak, range),
    })),
  };
}

type GradeCase = { type: string; params?: Record<string, unknown> };

const gradeEntry = ({ type, params = {} }: GradeCase, i: number) => ({
  nodeId: `g${i}`,
  type,
  params: { ...defaultParams(type), ...params },
});

const GRADE_CHAINS: GradeCase[][] = [
  [],
  [{ type: "grade.colour" }],
  [
    {
      type: "grade.colour",
      params: {
        curve: "vintage",
        hue: 20,
        vibrance: 0.5,
        warmth: 1,
        gamma: 1.2,
        saturation: 1.4,
        contrast: 1.3,
        brightness: 0.1,
        exposure: -0.5,
      },
    },
  ],
  [{ type: "grade.colour", params: { contrast: 1.2 } }],
  [{ type: "grade.colour", params: { warmth: -1 } }],
  [{ type: "grade.colour", params: { warmth: 0.333 } }],
  [
    { type: "grade.colour", params: { shadows: "#ff0000", shadowsAmount: 1 } },
    { type: "grade.colour", params: { highlights: "#808080", highlightsAmount: 1 } },
    { type: "grade.colour", params: { midtones: "", midtonesAmount: 1 } },
  ],
  [
    {
      type: "grade.colour",
      params: { shadows: "#3366cc", shadowsAmount: 0.5, highlights: "orange", midtones: "#0f0" },
    },
  ],
  [
    { type: "grade.texture", params: { vignette: 0.5 } },
    { type: "grade.colour", params: { contrast: 1.2 } },
  ],
  [
    {
      type: "grade.texture",
      params: {
        grain: 0.5,
        vignette: 0.5,
        vignetteX: 0.25,
        vignetteY: 0.75,
        soften: 0.5,
        sharpen: 0.5,
      },
    },
  ],
  [{ type: "grade.texture", params: { soften: 1 } }],
  [{ type: "grade.lens", params: { fringe: 1 } }],
  [{ type: "grade.lens", params: { fringe: 0.01 } }],
  [{ type: "grade.lens", params: { trails: 0.5, bulge: -1, fringe: 0.5 } }],
  [
    { type: "grade.texture", params: { grain: 1 } },
    { type: "grade.lens", params: { trails: 1 } },
  ],
  [{ type: "grade.mono" }],
  [{ type: "grade.mono", params: { kind: "sepia", strength: 0.5 } }],
  [{ type: "grade.mono", params: { kind: "tint", tint: "#3366ff", strength: 0.75 } }],
  [{ type: "grade.mono", params: { kind: "tint", tint: "", strength: 1 } }],
  [{ type: "grade.mono", params: { strength: 0 } }],
  [{ type: "grade.autoFix" }],
  [
    {
      type: "grade.autoFix",
      params: { whiteBalance: true, levels: true, levelsStrength: 0.5, deband: true },
    },
  ],
  [{ type: "grade.lookFile", params: { file: "looks/film.cube" } }],
  [{ type: "grade.lookFile", params: { file: "looks/it's [a],b;c:d.3dl", strength: 0.5 } }],
  [
    { type: "grade.colour", params: { contrast: 1.1 } },
    { type: "grade.lookFile", params: { file: "looks/hald.png", strength: 0.8 } },
    { type: "grade.lookFile", params: { file: "looks/second.png" } },
    { type: "grade.mono", params: { kind: "sepia", strength: 0.3 } },
    { type: "grade.texture", params: { sharpen: 0.2 } },
  ],
  [{ type: "grade.lookFile", params: { file: "", strength: 1 } }],
];

const GRADE_SIZES: GradeSize[] = [
  { width: 1920, height: 1080, still: false },
  { width: 1920, height: 1080, still: true },
  { width: 960, height: 540, still: false },
];

function gradeCases() {
  return {
    chains: GRADE_CHAINS.flatMap((chain) =>
      GRADE_SIZES.map((size) => {
        const entries = chain.map(gradeEntry);
        return { entries, size, out: gradeFilters(entries, size, (file) => `/project/${file}`) };
      }),
    ),
    lookKinds: ["a.cube", "a.3DL", "a.png", "a.jpg", "noext"].map((file) => ({
      file,
      kind: lookKindOf(file),
    })),
    paths: ["/p/look.cube", "C:\\looks\\it's [a],b;c:d.cube"].map((file) => ({
      file,
      out: filterPath(file),
    })),
  };
}

const PIX_FMTS = [
  "yuv420p",
  "yuvj420p",
  "yuv420p10le",
  "nv12",
  "p010le",
  "yuv422p",
  "yuv422p10le",
  "yuyv422",
  "yuv444p",
  "yuv444p12le",
  "gbrp",
  "rgb24",
  "rgba",
  "bgr0",
  "rgb48le",
  "rgba64le",
  "gray",
  "gray16le",
  "pal8",
  "",
];

function reverseCases() {
  const plans = [
    { durationMs: 10_000, fps: 30, frameBytes: 1920 * 1080 * 1.5, budgetBytes: 512 * 1024 ** 2 },
    { durationMs: 60_000, fps: 30, frameBytes: 3840 * 2160 * 1.5, budgetBytes: 512 * 1024 ** 2 },
    { durationMs: 12_345, fps: 29.97, frameBytes: 1920 * 1080 * 3, budgetBytes: 100 * 1024 ** 2 },
    { durationMs: 1000, fps: 60, frameBytes: 1e9, budgetBytes: 512 * 1024 ** 2 },
  ];
  return {
    frameBytes: PIX_FMTS.map((pixFmt) => ({
      pixFmt,
      bytes: frameBytes({ width: 1921, height: 1081, pixFmt }),
    })),
    plans: plans.map((plan) => {
      const pieces = reversePieces(plan);
      return {
        plan,
        pieces,
        lengths: pieces.map((p) => pieceLengthMs(p, plan.durationMs)),
        args: pieces.flatMap((piece) => [
          reverseArgs("in.mp4", "out.mkv", {
            piece,
            sound: true,
            soundCodec: ["-c:a", "pcm_s16le"],
          }),
          reverseArgs("in.mp4", "out.mkv", { piece, sound: false }),
        ]),
      };
    }),
    whole: [true, false].map((sound) => reverseArgs("in.mp4", "out.mp4", { sound })),
    sound: reverseSoundArgs("in.mp3", "out.mp3", ["-vn", "-c:a", "libmp3lame", "-q:a", "2"]),
    list: reversedPartsList([
      { file: "/tmp/part-0.mkv", lengthMs: 1234.5 },
      { file: "/tmp/it's part-1.mkv", lengthMs: 999 },
    ]),
    join: [true, false].map((sound) => joinReversedArgs("parts.txt", "out.mp4", sound)),
  };
}

const MEASURED = `[Parsed_loudnorm_0 @ 0x1]
{
	"input_i" : "-19.52",
	"input_tp" : "-3.10",
	"input_lra" : "5.20",
	"input_thresh" : "-29.87",
	"output_i" : "-14.02",
	"output_tp" : "-1.00",
	"output_lra" : "4.90",
	"output_thresh" : "-24.35",
	"normalization_type" : "dynamic",
	"target_offset" : "0.02"
}`;

function edited(mix: Workflow["mix"]): Workflow["mix"] {
  const base = mixSchema.parse(mix ?? {});
  const [voice, bed, punchy] = STRIP_PRESETS;
  const channels = Object.fromEntries(
    Object.entries(base.channels).map(([id, strip], i) => [
      id,
      {
        ...strip,
        ...[voice, bed, punchy][i % 3].strip,
        pan: [-0.5, 0.75, 0.003][i % 3],
        trim: [3, -2.5, 0][i % 3],
      },
    ]),
  );
  return mixSchema.parse({
    channels,
    buses: base.buses.map((bus, i) => ({
      ...bus,
      highpass: { on: i === 0, freq: 120 },
      comp: { on: i === 1, threshold: -20, ratio: 2.5, attack: 5, release: 100, makeup: 4 },
      duck: bus.duck.on
        ? { ...bus.duck, amount: 22.5, attack: 12.345, release: 678 }
        : { on: false, keys: [] },
    })),
    master: {
      fader: -1.5,
      eq: { on: true, bands: voice.strip.eq?.bands },
      limiter: { on: true, ceiling: -1 },
      loudness: "web",
    },
  });
}

function mixPlans(name: string, wf: Workflow): Array<{ name: string; plan: MixPlan }> {
  const plans: Array<{ name: string; plan: MixPlan }> = [];
  const compiled = compileWorkflow(wf);
  for (const [id, spec] of Object.entries(compiled.specs)) {
    plans.push({ name: `${name}/${id}`, plan: buildMixPlan(spec, wf.mix) });
    if (wf.mix)
      plans.push({ name: `${name}/${id}/edited`, plan: buildMixPlan(spec, edited(wf.mix)) });
  }
  return plans;
}

const resolve = (ref: string) => `/project/${ref}`;
const stemOf = (id: string) => `/stems/${id}.wav`;

function mixJobs(plan: MixPlan) {
  const channels = plan.channels.map((channel) => {
    const job = channelJob(plan, channel, resolve);
    return {
      id: channel.id,
      job,
      args: jobArgs(
        job,
        "/stems/out.wav",
        "pcm_f32le",
        { pre: "/t/pre.wav", post: "/t/post.wav" },
        10.5,
      ),
    };
  });
  const buses = plan.buses.map((bus) => {
    const job = busJob(plan, bus, stemOf);
    return { id: bus.id, job, args: jobArgs(job, "/stems/bus.wav", "pcm_f32le") };
  });
  const busFiles = plan.buses.map((b) => stemOf(b.id));
  const measure = loudnessMeasureJob(plan, busFiles);
  return {
    channels,
    buses,
    master: [null, parseLoudnorm(MEASURED)].map((measured) => {
      const job = masterJob(plan, busFiles, measured);
      return {
        measured,
        job,
        args: jobArgs(
          job,
          "/stems/master.wav",
          "pcm_s16le",
          { pre: "/t/a.wav", post: "/t/b.wav" },
          plan.durationInFrames / plan.fps + 1,
        ),
      };
    }),
    measure: measure ? { job: measure, args: measureArgs(measure) } : null,
  };
}

function shaped(plan: MixPlan): MixPlan {
  const [first, ...rest] = plan.channels;
  const sound = first.sounds[0];
  const envelope = {
    volume: [
      { frame: 0, value: 0, ease: "linear" as const },
      { frame: 30, value: 1, ease: "in-out" as const },
      { frame: 90, value: 1, ease: "linear" as const },
      { frame: 120, value: 0.25, ease: "out" as const },
    ],
    pan: [
      { frame: 10, value: -1, ease: "linear" as const },
      { frame: 70, value: 0.6, ease: "linear" as const },
    ],
  };
  const sounds = [
    { ...sound, rate: 1.5, offset: 7, loop: false, fadeIn: 400, fadeOut: 500, envelope },
    {
      ...sound,
      at: 0,
      rate: 0.3,
      volume: 1.25,
      envelope: { volume: [{ frame: 5, value: 0.5, ease: "linear" as const }] },
    },
    {
      ...sound,
      at: 3,
      rate: 3,
      offset: 0,
      loop: true,
      length: 1,
      envelope: { pan: [{ frame: 0, value: 0.5, ease: "linear" as const }] },
    },
  ];
  return { ...plan, channels: [{ ...first, sounds }, ...rest] };
}

function mixCases(workflows: Array<[string, Workflow]>) {
  const plans = workflows
    .flatMap(([name, wf]) => mixPlans(name, wf))
    .filter(({ plan }) => plan.channels.length);
  const withShapes = plans.flatMap((p) => [p, { name: `${p.name}/shaped`, plan: shaped(p.plan) }]);
  return {
    plans: withShapes.map(({ name, plan }) => ({ name, plan, jobs: mixJobs(plan) })),
    loudnorm: [
      MEASURED,
      `${MEASURED.replace('"-19.52"', '"-inf"')}`,
      "no json here",
      `{ "input_i": "", "input_tp": "1", "input_lra": "1", "input_thresh": "1", "target_offset": "0" }`,
    ].map((stderr) => ({ stderr, measured: parseLoudnorm(stderr) })),
    errors: [
      "ffmpeg exited with 254: [in#0 @ 0x1] Error opening input: No such file or directory\nError opening input file /home/me/project/assets/my song.mp3.\nError opening input files: No such file or directory",
      "ffmpeg exited with 254: Error opening input: No such file or directory\n/home/me/project/assets/song.mp3: No such file or directory",
      "bad",
    ].map((output) => ({
      output,
      message: friendlyMixError(Object.assign(new Error("x"), { name: "ToolError", output }))
        .message,
    })),
  };
}

function lcg(seed: number): () => number {
  let state = seed >>> 0;
  return () => {
    state = (Math.imul(state, 1664525) + 1013904223) >>> 0;
    return state / 2 ** 32;
  };
}

function clickTrack(rate: number, seconds: number, bpm: number, seed: number): Float32Array {
  const samples = new Float32Array(rate * seconds);
  const next = lcg(seed);
  const beat = Math.round((rate * 60) / bpm);
  for (let i = 0; i < samples.length; i++) {
    const sinceBeat = i % beat;
    const click = sinceBeat < 200 ? (sinceBeat % 16 < 8 ? 0.8 : -0.8) : 0;
    samples[i] = click + (next() - 0.5) * 0.02;
  }
  return samples;
}

function analysisCases() {
  const signals = [
    { rate: 8000, seconds: 10, bpm: 120, seed: 1 },
    { rate: 8000, seconds: 12, bpm: 97, seed: 2 },
    { rate: 8000, seconds: 3, bpm: 120, seed: 3 },
    { rate: 8000, seconds: 8, bpm: 0, seed: 4 },
  ];
  return {
    signals: signals.map((s) => {
      const samples =
        s.bpm > 0
          ? clickTrack(s.rate, s.seconds, s.bpm, s.seed)
          : Float32Array.from({ length: s.rate * s.seconds }, lcg(s.seed));
      return {
        signal: s,
        levels: waveformLevels(samples, s.rate),
        beats: detectBeats(samples, s.rate),
      };
    }),
    frames: [
      { sampleRate: 48000, channels: 2, fps: 30, frames: 20, seed: 5, gain: 0.5 },
      { sampleRate: 48000, channels: 2, fps: 29.97, frames: 45, seed: 6, gain: 1 },
      { sampleRate: 44100, channels: 1, fps: 24, frames: 10, seed: 7, gain: 0.001 },
      { sampleRate: 48000, channels: 2, fps: 30, frames: 30, seed: 8, gain: 0 },
    ].map((s) => {
      const next = lcg(s.seed);
      const count = Math.floor((s.sampleRate * (s.frames - 2)) / s.fps);
      const data = Array.from({ length: s.channels }, () =>
        Float32Array.from({ length: count }, () => (next() * 2 - 1) * s.gain),
      );
      return {
        signal: s,
        levels: frameLevels(
          { sampleRate: s.sampleRate, channels: s.channels, frames: count, data },
          s.fps,
          s.frames,
        ),
      };
    }),
  };
}

const RECORDING = {
  format: { duration: "12.480000", format_name: "mov,mp4,m4a,3gp,3g2,mj2" },
  streams: [
    {
      codec_type: "video",
      codec_name: "h264",
      width: 1920,
      height: 1080,
      r_frame_rate: "30/1",
      avg_frame_rate: "30000/1001",
      pix_fmt: "yuv420p",
      duration: "12.466667",
    },
    {
      codec_type: "audio",
      codec_name: "aac",
      sample_rate: "48000",
      channels: 2,
      duration: "12.480000",
    },
  ],
};

const PROBES = [
  RECORDING,
  { streams: [{ codec_type: "video", r_frame_rate: "25", avg_frame_rate: "0/0", duration: "2" }] },
  {
    format: { format_name: "png_pipe" },
    streams: [{ codec_type: "video", codec_name: "png", width: 10, height: 10 }],
  },
  {
    format: { duration: "3.000000", format_name: "mp3" },
    streams: [
      { codec_type: "audio", codec_name: "mp3", sample_rate: "44100", channels: 2 },
      {
        codec_type: "video",
        codec_name: "mjpeg",
        width: 600,
        height: 600,
        disposition: { attached_pic: 1 },
      },
    ],
  },
  { format: { duration: "N/A" }, streams: [] },
  {},
  { ...RECORDING, streams: [{ ...RECORDING.streams[0], width: 1280 }, RECORDING.streams[1]] },
  { ...RECORDING, streams: [RECORDING.streams[0]] },
];

function probeCases() {
  const parsed = PROBES.map((raw) => parseProbe(raw));
  return {
    probes: PROBES.map((raw, i) => ({ raw, probe: { ...parsed[i], raw: undefined } })),
    streamCopy: parsed.flatMap((a, i) =>
      parsed.map((b, j) => ({ a: i, b: j, copy: canStreamCopy(a, b) })),
    ),
  };
}

export function mediaFixtures(workflows: Array<[string, Workflow]>): Record<string, unknown> {
  return {
    "media/codecs.json": codecCases(),
    "media/filters.json": filterCases(),
    "media/grade.json": gradeCases(),
    "media/reverse.json": reverseCases(),
    "media/mix.json": mixCases(workflows),
    "media/analysis.json": analysisCases(),
    "media/probe.json": probeCases(),
  };
}
