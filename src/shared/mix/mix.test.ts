import { describe, expect, it } from "vitest";
import { itemHasSound } from "../compile/clip";
import { findById } from "../lists";
import type { TimelineSpec } from "../schema/timeline";
import type { Workflow } from "../schema/workflow";
import { GraphBuilder } from "../templates/build";
import { filmSpec, itemsOf, must, TITLE } from "../test-fixtures";
import { busJob, channelJob, jobArgs, loudnessMeasureJob, masterJob, parseLoudnorm } from "./graph";
import { applyMixLevels, withMasterStem } from "./levels";
import { buildMixPlan, channelIdentity, type MixPlan, type SoundPlacement } from "./plan";
import { DEFAULT_STRIP, dbToGain, defaultMix, type Bus, type Mix, type Strip } from "./types";

function project(mix?: Mix): { wf: Workflow; spec: TimelineSpec } {
  const g = new GraphBuilder();
  g.node("t", TITLE, 0, 0, { clipDuration: 300 });
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
    .link("mix", "out");
  const wf = { ...g.build({ id: "wf", name: "Mix" }), ...(mix ? { mix } : {}) };
  return { wf, spec: filmSpec(wf) };
}

const strip = (patch: Partial<Strip>): Strip => ({ ...DEFAULT_STRIP, ...patch });

const channel = (plan: MixPlan, id: string) => must(findById(plan.channels, id), `channel ${id}`);

const sound = (spec: TimelineSpec, nodeId: string) =>
  must(
    spec.audio.find((a) => a.nodeId === nodeId),
    `sound ${nodeId}`,
  );

const withDuck = (mix: Mix, patch: Partial<Bus["duck"]>): Mix => ({
  ...mix,
  buses: mix.buses.map((b) =>
    b.id === "music"
      ? {
          ...b,
          duck: { ...b.duck, on: true, keys: ["effects"], amount: 12, attack: 100, ...patch },
        }
      : b,
  ),
});

const NO_PROCESSING = {
  highpass: DEFAULT_STRIP.highpass,
  eq: DEFAULT_STRIP.eq,
  comp: DEFAULT_STRIP.comp,
};

describe("mixer levels", () => {
  const gain = (mix: Mix, nodeId: string) =>
    sound(applyMixLevels(project().spec, mix), nodeId).gain;

  it("multiplies channel, bus and master faders", () => {
    const mix = defaultMix();
    mix.channels.fx = strip({ fader: -6 });
    mix.buses = mix.buses.map((b) => (b.id === "effects" ? { ...b, fader: -6 } : b));
    mix.master = { ...mix.master, fader: 6 };
    expect(gain(mix, "fx")).toBeCloseTo(dbToGain(-6), 5);
    expect(gain(mix, "music")).toBeCloseTo(dbToGain(6), 5);
  });

  it("mutes and solos channels and buses", () => {
    const mix = defaultMix();
    mix.channels.fx = strip({ solo: true });
    expect(gain(mix, "music")).toBe(0);
    expect(gain(mix, "fx")).toBe(1);
    mix.buses = mix.buses.map((b) => (b.id === "music" ? { ...b, solo: true } : b));
    expect(gain(mix, "music")).toBe(1);
    mix.channels.music = strip({ mute: true });
    expect(gain(mix, "music")).toBe(0);
  });

  it("applies gains to sounds and clip sound in the live preview", () => {
    const mix = defaultMix();
    mix.channels.music = strip({ fader: -20 });
    mix.channels.clipfile = strip({ mute: true });
    const { spec } = project(mix);
    const live = applyMixLevels(spec, mix);
    expect(sound(live, "music").gain).toBeCloseTo(0.1, 5);
    expect(sound(live, "fx").gain).toBe(1);
    const video = itemsOf(live).find((i) => i.nodeId === "clipfile");
    expect(video?.props).toMatchObject({ muted: true, volume: 0 });
  });

  it("lets ducking ignore sounds shorter than a set length", () => {
    const mix = defaultMix();
    const { spec } = project(withDuck(mix, {}));
    const music = (m: Mix) => sound(applyMixLevels(spec, m), "music");
    expect(music(withDuck(mix, {})).duck?.keys).toEqual(["fx"]);
    expect(music(withDuck(mix, { ignoreShorterMs: 5000 })).duck).toBeUndefined();
  });

  it("turns bus ducking into a live approximation keyed by channel", () => {
    const mix = withDuck(defaultMix(), {});
    const { spec } = project(mix);
    const live = applyMixLevels(spec, mix);
    const music = sound(live, "music");
    expect(music.duck?.keys).toEqual(["fx"]);
    expect(music.duck?.level).toBeCloseTo(dbToGain(-12), 5);
    expect(sound(live, "fx").duck).toBeUndefined();
  });

  it("ducks only for clips whose sound plays", () => {
    const musicOver = (rate: number) => {
      const g = new GraphBuilder();
      g.node("music", "audio.music", 0, 0, { src: "assets/song.mp3", fadeInMs: 0, fadeOutMs: 0 });
      g.node("clipfile", "media.import", 0, 1, { path: "assets/clip.mp4" });
      g.node("fast", "timeline.speed", 1, 1, { rate });
      g.node("mix", "audio.mix", 2, 0);
      g.node("out", "output.render", 3, 0);
      g.link("clipfile", "fast")
        .link("fast", "mix")
        .link("music", "mix", "audio", "sounds")
        .link("mix", "out");
      const spec = filmSpec(g.build({ id: "wf", name: "Sped up" }));
      expect(itemsOf(spec).some(itemHasSound)).toBe(true);
      return sound(applyMixLevels(spec, withDuck(defaultMix(), { keys: ["clips"] })), "music");
    };
    expect(musicOver(1).duck?.keys).toEqual(["clipfile"]);
    expect(musicOver(2).duck).toBeUndefined();
  });

  it("plays only the master stem when one is ready", () => {
    const { spec } = project();
    const stem = withMasterStem(spec, { src: "project:.opsis/cache/mix/m.wav", hash: "m" });
    expect(stem.audio).toEqual([]);
    expect(stem.mix?.hash).toBe("m");
    expect(itemsOf(stem).filter(itemHasSound)).toEqual([]);
  });
});

describe("mix plan", () => {
  it("lists channels in the order their sounds are connected, not by when they play", () => {
    const g = new GraphBuilder();
    g.node("t", TITLE, 0, 0, { clipDuration: 300 });
    g.node("late", "audio.sfx", 0, 1, { src: "app:sfx/whoosh.wav", atMs: 5000 });
    g.node("early", "audio.sfx", 0, 2, { src: "app:sfx/whip.wav", atMs: 1000 });
    g.node("mix", "audio.mix", 1, 0);
    g.node("out", "output.render", 2, 0);
    g.link("t", "mix")
      .link("late", "mix", "audio", "sounds")
      .link("early", "mix", "audio", "sounds")
      .link("mix", "out");
    const plan = buildMixPlan(filmSpec(g.build({ id: "wf", name: "Order" })), undefined);
    expect(plan.channels.map((c) => c.id)).toEqual(["late", "early"]);
  });

  it("has a channel per sound source, routed to the default buses", () => {
    const { spec } = project();
    const plan = buildMixPlan(spec, undefined);
    expect(plan.channels.map((c) => [c.id, c.kind, c.bus])).toEqual([
      ["music", "music", "music"],
      ["fx", "effect", "effects"],
      ["clipfile", "clip", "clips"],
    ]);
    expect(findById(plan.buses, "effects")?.channels).toEqual(["fx"]);
    expect(channel(plan, "fx").sounds).toEqual([
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
    expect(channelIdentity(a, channel(a, "fx"))).toEqual(channelIdentity(b, channel(b, "fx")));
    expect(findById(b.buses, "effects")?.channels).toEqual([]);
  });
});

const FILM = { fps: 30, durationInFrames: 300 };

describe("mix filter graphs", () => {
  const resolve = (ref: string) => `/files/${ref.split(":")[1]}`;

  const PADDED = "[s0]anull,apad,atrim=end=10,asetpts=PTS-STARTPTS[out]";
  const soundJob = (placement: Partial<SoundPlacement>, s: Strip = DEFAULT_STRIP) =>
    channelJob(
      FILM,
      {
        sounds: [
          {
            src: "project:a.wav",
            at: 0,
            offset: 0,
            length: 90,
            rate: 1,
            loop: false,
            volume: 1,
            fadeIn: 0,
            fadeOut: 0,
            ...placement,
          },
        ],
        strip: s,
      },
      resolve,
    );

  it("places a sound in time with fades and volume", () => {
    const job = soundJob({ at: 30, offset: 15, length: 60, volume: 0.5, fadeIn: 6, fadeOut: 12 });
    expect(job.inputs).toEqual(["/files/a.wav"]);
    expect(job.filter).toBe(
      `[0:a]aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo,atrim=start=0.5,asetpts=PTS-STARTPTS,atrim=end=2,asetpts=PTS-STARTPTS,volume=0.5,afade=t=in:st=0:d=0.2,afade=t=out:st=1.6:d=0.4,adelay=delays=1000:all=1[s0];${PADDED}`,
    );
  });

  it("loops and pans with keyframes", () => {
    const { filter } = soundJob({
      loop: true,
      envelope: {
        pan: [
          { frame: 0, value: -1, ease: "linear" },
          { frame: 30, value: 1, ease: "linear" },
        ],
      },
    });
    expect(filter).toContain("aloop=loop=-1:size=2147483647");
    expect(filter).toContain("channelsplit=channel_layout=stereo[s0l][s0r]");
    expect(filter).toContain("[s0L][s0R]join=inputs=2:channel_layout=stereo[s0]");
    expect(filter).toContain("min(1,1-(if(lt(t,0),-1,if(lt(t,1),-1+(2)*(t-0)/1,1))))");
  });

  it("builds volume keyframes as a piecewise expression", () => {
    const { filter } = soundJob({
      envelope: {
        volume: [
          { frame: 0, value: 1, ease: "linear" },
          { frame: 30, value: 0, ease: "linear" },
          { frame: 60, value: 0, ease: "linear" },
        ],
      },
    });
    expect(filter).toContain(
      "volume=eval=frame:volume='if(lt(t,0),1,if(lt(t,1),1+(-1)*(t-0)/1,if(lt(t,2),0,0)))'",
    );
  });

  it("runs a channel strip in signal order", () => {
    const { filter } = soundJob(
      {},
      {
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
    );
    const pinned = "aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo";
    expect(filter.split(";").slice(1)).toEqual([
      "[s0]anull,apad,atrim=end=10,asetpts=PTS-STARTPTS,volume=3dB,highpass=f=80,lowshelf=f=120:t=q:w=0.7:g=-2,equalizer=f=2500:t=q:w=1.4:g=3,highshelf=f=8000:t=q:w=0.7:g=1.5[sum]",
      `[sum]${pinned},asplit=2[pre][c]`,
      `[c]acompressor=threshold=0.1:ratio=4:attack=5:release=100:makeup=1.995262,${pinned},asplit=2[post][m]`,
      "[m]pan=stereo|c0=1*c0|c1=0.5*c1,volume=-3dB[out]",
    ]);
  });

  it("mixes a channel's sounds and pads it to the film length", () => {
    const { spec } = project();
    const plan = buildMixPlan(spec, undefined);
    const job = channelJob(plan, channel(plan, "music"), resolve);
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
      FILM,
      {
        id: "music",
        name: "Music",
        fader: -2,
        channels: ["music"],
        duck: { keys: ["fx", "clip"], amount: 14, attack: 50, release: 400 },
        processing: NO_PROCESSING,
      },
      (id) => `/stems/${id}.wav`,
    );
    expect(job.inputs).toEqual(["/stems/music.wav", "/stems/fx.wav", "/stems/clip.wav"]);
    expect(job.filter).toBe(
      "[0:a]anull,apad,atrim=end=10,asetpts=PTS-STARTPTS,volume=-2dB[main];[1:a][2:a]amix=inputs=2:normalize=0:duration=longest:dropout_transition=0,apad,atrim=end=10,asetpts=PTS-STARTPTS[key];[main][key]sidechaincompress=threshold=0.01:ratio=2:attack=50:release=400:makeup=1,apad,atrim=end=10,asetpts=PTS-STARTPTS[out]",
    );
  });

  it("measures and then normalises loudness on the master, limiting last", () => {
    const master = {
      ...defaultMix().master,
      loudness: "web" as const,
      limiter: { on: true, ceiling: -1 },
    };
    const plan = { ...FILM, master };
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
    expect(parseLoudnorm('{ "input_i" : "-20.1", "input_tp" : "-3.2" }')).toBeNull();
    expect(parseLoudnorm('{ "input_i" : "-inf" }')).toBeNull();
    const job = masterJob(plan, ["/b1.wav", "/b2.wav"], measured);
    expect(job.filter).toBe(
      "[0:a][1:a]amix=inputs=2:normalize=0:duration=longest:dropout_transition=0,apad,atrim=end=10,asetpts=PTS-STARTPTS,loudnorm=I=-14:TP=-1:LRA=11:measured_I=-20.1:measured_TP=-3.2:measured_LRA=4.0:measured_thresh=-30.5:offset=0.1:linear=true:print_format=summary,aresample=48000,aresample=192000,alimiter=limit=0.891251:level=false,aresample=48000[out]",
    );
  });

  it("uses silence for an empty bus", () => {
    const job = busJob(
      { fps: 30, durationInFrames: 60 },
      { id: "x", name: "X", fader: 0, channels: [], duck: null, processing: NO_PROCESSING },
      () => "",
    );
    expect(job).toEqual({
      inputs: [],
      filter: "anullsrc=r=48000:cl=stereo,atrim=end=2[out]",
      output: "out",
    });
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
      FILM,
      { id: "fx", name: "Effects", fader: -3, channels: ["a"], duck: null, processing },
      (id) => `${id}.wav`,
    );
    expect(job.filter).toContain("highpass=f=60,lowshelf=f=100");
    expect(job.filter.indexOf("acompressor")).toBeLessThan(job.filter.indexOf("volume=-3dB"));
  });
});
