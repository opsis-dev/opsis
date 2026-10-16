import { describe, expect, it } from "vitest";
import { compileWorkflow, previewSpec } from "../compile/compile";
import { normalizeWorkflow } from "../nodes/registry";
import type { TimelineSpec } from "../schema/timeline";
import type { Workflow } from "../schema/workflow";
import { GraphBuilder } from "../templates/build";
import {
  busJob,
  channelJob,
  duckRatio,
  envelopeExpr,
  jobArgs,
  loudnessMeasureJob,
  masterJob,
  parseLoudnorm,
  soundChain,
  stripSteps,
} from "./graph";
import { applyMixLevels, channelGain, withMasterStem } from "./levels";
import { buildMixPlan, channelKey } from "./plan";
import { DEFAULT_STRIP, defaultMix, stripSchema, type Mix } from "./types";

function project(mix?: Mix): { wf: Workflow; spec: TimelineSpec } {
  const g = new GraphBuilder();
  g.node("t", "fx.core.title", 0, 0, { clipDuration: 300 });
  g.node("clipfile", "media.import", 0, 1, { path: "assets/clip.mp4" });
  g.node("music", "audio.music", 0, 2, { src: "assets/song.mp3", fadeInMs: 0, fadeOutMs: 0 });
  g.node("fx", "audio.sfx", 0, 3, { src: "app:sfx/whoosh.wav", atMs: 2000 });
  g.node("layers", "timeline.stack", 1, 0);
  g.node("mix", "audio.mix", 2, 0);
  g.node("out", "output.render", 3, 0);
  g.link("t", "layers", "clip", "in1")
    .link("clipfile", "layers", "clip", "in2")
    .link("layers", "mix")
    .link("music", "mix", "audio", "sounds")
    .link("fx", "mix", "audio", "sounds")
    .link("mix", "out", "clip", "timeline");
  const wf = { ...g.build({ id: "wf", name: "Mix" }), ...(mix ? { mix } : {}) };
  const spec = previewSpec(wf, compileWorkflow(wf), null)!.spec;
  return { wf, spec };
}

const strip = (patch: Record<string, unknown>) => stripSchema.parse(patch);

describe("mixer levels", () => {
  it("multiplies channel, bus and master faders", () => {
    const mix = defaultMix();
    mix.channels.fx = strip({ fader: -6 });
    mix.buses = mix.buses.map((b) => (b.id === "effects" ? { ...b, fader: -6 } : b));
    mix.master = { ...mix.master, fader: 6 };
    expect(channelGain(mix, "fx", "effect")).toBeCloseTo(10 ** (-6 / 20), 5);
    expect(channelGain(mix, "music", "music")).toBeCloseTo(10 ** (6 / 20), 5);
  });

  it("mutes and solos channels and buses", () => {
    const mix = defaultMix();
    mix.channels.fx = strip({ solo: true });
    expect(channelGain(mix, "music", "music")).toBe(0);
    expect(channelGain(mix, "fx", "effect")).toBe(1);
    mix.buses = mix.buses.map((b) => (b.id === "music" ? { ...b, solo: true } : b));
    expect(channelGain(mix, "music", "music")).toBe(1);
    mix.channels.music = strip({ mute: true });
    expect(channelGain(mix, "music", "music")).toBe(0);
  });

  it("applies gains to sounds and clip sound in the live preview", () => {
    const mix = defaultMix();
    mix.channels.music = strip({ fader: -20 });
    mix.channels.clipfile = strip({ mute: true });
    const { spec } = project(mix);
    const live = applyMixLevels(spec, mix);
    expect(live.audio.find((a) => a.nodeId === "music")!.gain).toBeCloseTo(0.1, 5);
    expect(live.audio.find((a) => a.nodeId === "fx")!.gain).toBe(1);
    const video = JSON.stringify(live.tracks);
    expect(video).toContain('"muted":true');
  });

  it("lets ducking ignore sounds shorter than a set length", () => {
    const mix = defaultMix();
    const duck = (ignoreShorterMs: number) => ({
      ...mix,
      buses: mix.buses.map((b) =>
        b.id === "music"
          ? {
              ...b,
              duck: {
                on: true,
                keys: ["effects"],
                amount: 12,
                attack: 100,
                release: 300,
                ignoreShorterMs,
              },
            }
          : b,
      ),
    });
    const { spec } = project(duck(0));
    const music = (m: ReturnType<typeof duck>) =>
      applyMixLevels(spec, m).audio.find((a) => a.nodeId === "music")!;
    expect(music(duck(0)).duck?.keys).toEqual(["fx"]);
    expect(music(duck(5000)).duck).toBeUndefined();
  });

  it("turns bus ducking into a live approximation keyed by channel", () => {
    const mix = defaultMix();
    mix.buses = mix.buses.map((b) =>
      b.id === "music"
        ? {
            ...b,
            duck: {
              on: true,
              keys: ["effects"],
              amount: 12,
              attack: 100,
              release: 300,
              ignoreShorterMs: 0,
            },
          }
        : b,
    );
    const { spec } = project(mix);
    const live = applyMixLevels(spec, mix);
    const music = live.audio.find((a) => a.nodeId === "music")!;
    expect(music.duck?.keys).toEqual(["fx"]);
    expect(music.duck?.level).toBeCloseTo(10 ** (-12 / 20), 5);
    expect(live.audio.find((a) => a.nodeId === "fx")!.duck).toBeUndefined();
  });

  it("plays only the master stem when one is ready", () => {
    const { spec } = project();
    const stem = withMasterStem(spec, { src: "project:.demoflow/cache/mix/m.wav", hash: "m" });
    expect(stem.audio).toEqual([]);
    expect(stem.mix?.hash).toBe("m");
    expect(JSON.stringify(stem.tracks)).not.toContain('"muted":false');
  });
});

describe("mix plan", () => {
  it("has a channel per sound source, routed to the default buses", () => {
    const { spec } = project();
    const plan = buildMixPlan(spec, undefined);
    expect(plan.channels.map((c) => [c.id, c.kind, c.bus])).toEqual([
      ["music", "music", "music"],
      ["fx", "effect", "effects"],
      ["clipfile", "clip", "clips"],
    ]);
    expect(plan.buses.find((b) => b.id === "effects")!.channels).toEqual(["fx"]);
    const fx = plan.channels.find((c) => c.id === "fx")!;
    expect(fx.sounds).toEqual([
      expect.objectContaining({ src: "app:sfx/whoosh.wav", at: 60, offset: 0, loop: false }),
    ]);
  });

  it("changes the plan key, but not the channel stem key, when only routing changes", () => {
    const { spec } = project();
    const a = buildMixPlan(spec, undefined);
    const mix = defaultMix();
    mix.channels.fx = strip({ mute: true });
    const b = buildMixPlan(spec, mix);
    expect(a.key).not.toBe(b.key);
    const fxA = a.channels.find((c) => c.id === "fx")!;
    const fxB = b.channels.find((c) => c.id === "fx")!;
    expect(channelKey(a, fxA)).toBe(channelKey(b, fxB));
    expect(b.buses.find((x) => x.id === "effects")!.channels).toEqual([]);
  });
});

describe("mix filter graphs", () => {
  const resolve = (ref: string) => `/files/${ref.split(":")[1]}`;

  it("places a sound in time with fades and volume", () => {
    expect(
      soundChain(
        {
          src: "a",
          at: 30,
          offset: 15,
          length: 60,
          loop: false,
          volume: 0.5,
          fadeIn: 6,
          fadeOut: 12,
        },
        2,
        "s2",
        30,
      ),
    ).toBe(
      "[2:a]aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo,atrim=start=0.5,asetpts=PTS-STARTPTS,atrim=end=2,asetpts=PTS-STARTPTS,volume=0.5,afade=t=in:st=0:d=0.2,afade=t=out:st=1.6:d=0.4,adelay=delays=1000:all=1[s2]",
    );
  });

  it("loops and pans with keyframes", () => {
    const chain = soundChain(
      {
        src: "a",
        at: 0,
        offset: 0,
        length: 90,
        loop: true,
        volume: 1,
        fadeIn: 0,
        fadeOut: 0,
        envelope: {
          pan: [
            { frame: 0, value: -1, ease: "linear" },
            { frame: 30, value: 1, ease: "linear" },
          ],
        },
      },
      0,
      "s0",
      30,
    );
    expect(chain).toContain("aloop=loop=-1:size=2147483647");
    expect(chain).toContain("channelsplit=channel_layout=stereo[s0l][s0r]");
    expect(chain).toContain("[s0L][s0R]join=inputs=2:channel_layout=stereo[s0]");
    expect(chain).toContain("min(1,1-(if(lt(t,0),-1,if(lt(t,1),-1+(2)*(t-0)/1,1))))");
  });

  it("builds volume keyframes as a piecewise expression", () => {
    expect(
      envelopeExpr(
        [
          { frame: 0, value: 1, ease: "linear" },
          { frame: 30, value: 0, ease: "linear" },
          { frame: 60, value: 0, ease: "linear" },
        ],
        30,
      ),
    ).toBe("if(lt(t,0),1,if(lt(t,1),1+(-1)*(t-0)/1,if(lt(t,2),0,0)))");
  });

  it("runs a channel strip in signal order", () => {
    expect(
      stripSteps({
        strip: {
          ...DEFAULT_STRIP,
          trim: 3,
          highpass: { on: true, freq: 80 },
          eq: {
            on: true,
            bands: [
              { type: "lowshelf", freq: 120, gain: -2, q: 0.7 },
              { type: "peak", freq: 500, gain: 0, q: 1 },
              { type: "peak", freq: 2500, gain: 3, q: 1.4 },
              { type: "highshelf", freq: 8000, gain: 1.5, q: 0.7 },
            ],
          },
          comp: { on: true, threshold: -20, ratio: 4, attack: 5, release: 100, makeup: 6 },
          pan: -0.5,
          fader: -3,
        },
      }),
    ).toEqual([
      "volume=3dB",
      "highpass=f=80",
      "lowshelf=f=120:t=q:w=0.7:g=-2",
      "equalizer=f=2500:t=q:w=1.4:g=3",
      "highshelf=f=8000:t=q:w=0.7:g=1.5",
      "acompressor=threshold=0.1:ratio=4:attack=5:release=100:makeup=1.995262",
      "pan=stereo|c0=1*c0|c1=0.5*c1",
      "volume=-3dB",
    ]);
  });

  it("mixes a channel's sounds and pads it to the film length", () => {
    const { spec } = project();
    const plan = buildMixPlan(spec, undefined);
    const music = plan.channels.find((c) => c.id === "music")!;
    const job = channelJob(plan, music, resolve);
    expect(job.inputs).toEqual(["/files/assets/song.mp3"]);
    expect(job.filter).toMatch(/\[s0\]anull,apad,atrim=end=10,asetpts=PTS-STARTPTS\[out\]$/);
    expect(jobArgs(job, "/out.wav", "pcm_f32le", undefined, 11)).toEqual([
      "-i",
      "/files/assets/song.mp3",
      "-filter_complex",
      job.filter,
      "-map",
      "[out]",
      "-t",
      "11",
      "-ac",
      "2",
      "-ar",
      "48000",
      "-c:a",
      "pcm_f32le",
      "/out.wav",
    ]);
  });

  it("ducks a bus with a sidechain keyed from other channels", () => {
    const job = busJob(
      { fps: 30, durationInFrames: 300 },
      {
        id: "music",
        name: "Music",
        fader: -2,
        channels: ["music"],
        duck: { keys: ["fx", "clip"], amount: 14, attack: 50, release: 400 },
      },
      (id) => `/stems/${id}.wav`,
    );
    expect(job.inputs).toEqual(["/stems/music.wav", "/stems/fx.wav", "/stems/clip.wav"]);
    expect(job.filter).toBe(
      `[0:a]anull,apad,atrim=end=10,asetpts=PTS-STARTPTS,volume=-2dB[main];[1:a][2:a]amix=inputs=2:normalize=0:duration=longest:dropout_transition=0,apad,atrim=end=10,asetpts=PTS-STARTPTS[key];[main][key]sidechaincompress=threshold=0.01:ratio=${duckRatio(
        14,
      )
        .toFixed(3)
        .replace(
          /\.?0+$/,
          "",
        )}:attack=50:release=400:makeup=1,apad,atrim=end=10,asetpts=PTS-STARTPTS[out]`,
    );
  });

  it("measures and then normalises loudness on the master, limiting last", () => {
    const master = {
      ...defaultMix().master,
      loudness: "web" as const,
      limiter: { on: true, ceiling: -1 },
    };
    const plan = { fps: 30, durationInFrames: 300, master };
    expect(loudnessMeasureJob(plan, ["/b1.wav"])?.filter).toContain(
      "loudnorm=I=-14:TP=-1:LRA=11:print_format=json",
    );
    const measured = parseLoudnorm(
      'junk\n{\n\t"input_i" : "-20.1",\n\t"input_tp" : "-3.2",\n\t"input_lra" : "4.0",\n\t"input_thresh" : "-30.5",\n\t"output_i" : "-14.0",\n\t"target_offset" : "0.1"\n}\n',
    );
    expect(measured).toEqual({
      input_i: "-20.1",
      input_tp: "-3.2",
      input_lra: "4.0",
      input_thresh: "-30.5",
      target_offset: "0.1",
    });
    const job = masterJob(plan, ["/b1.wav", "/b2.wav"], measured);
    expect(job.filter).toBe(
      "[0:a][1:a]amix=inputs=2:normalize=0:duration=longest:dropout_transition=0,apad,atrim=end=10,asetpts=PTS-STARTPTS,loudnorm=I=-14:TP=-1:LRA=11:measured_I=-20.1:measured_TP=-3.2:measured_LRA=4.0:measured_thresh=-30.5:offset=0.1:linear=true:print_format=summary,aresample=48000,aresample=192000,alimiter=limit=0.891251:level=false,aresample=48000[out]",
    );
  });

  it("uses silence for an empty bus", () => {
    const job = busJob(
      { fps: 30, durationInFrames: 60 },
      { id: "x", name: "X", fader: 0, channels: [], duck: null },
      () => "",
    );
    expect(job).toEqual({
      inputs: [],
      filter: "anullsrc=r=48000:cl=stereo,atrim=end=2[out]",
      output: "out",
    });
  });
});

describe("normalizeWorkflow", () => {
  it("prunes mixer channels of deleted nodes", () => {
    const g = new GraphBuilder();
    g.node("m", "audio.music", 0, 0);
    const wf = g.build({ id: "x", name: "x" });
    const mix = defaultMix();
    mix.channels.gone = strip({ fader: -3 });
    mix.channels.m = strip({ fader: -6 });
    expect(Object.keys(normalizeWorkflow({ ...wf, mix }).mix?.channels ?? {})).toEqual(["m"]);
  });
});

describe("bus processing", () => {
  it("runs the bus through low cut, EQ and compressor before its fader", () => {
    const processing = {
      highpass: { on: true, freq: 60 },
      eq: {
        on: true,
        bands: [
          { type: "lowshelf" as const, freq: 100, gain: 2, q: 0.7 },
          { type: "peak" as const, freq: 600, gain: 0, q: 1 },
          { type: "peak" as const, freq: 4000, gain: 0, q: 1 },
          { type: "highshelf" as const, freq: 9000, gain: 0, q: 0.7 },
        ],
      },
      comp: { on: true, threshold: -16, ratio: 4, attack: 3, release: 80, makeup: 0 },
    };
    const job = busJob(
      { fps: 30, durationInFrames: 300 },
      { id: "fx", name: "Effects", fader: -3, channels: ["a"], duck: null, processing },
      (id) => `${id}.wav`,
    );
    expect(job.filter).toContain("highpass=f=60,lowshelf=f=100");
    expect(job.filter.indexOf("acompressor")).toBeLessThan(job.filter.indexOf("volume=-3dB"));
  });
});
