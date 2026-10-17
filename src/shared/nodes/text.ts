import { z } from "zod";
import { EMPTY_CLIP_FRAMES, emptyClip, itemClip, stackClips } from "../compile/clip";
import { CAPTIONS_ELEMENT, captionsSchema } from "../elements/core";
import { asValue, type CaptionsValue } from "../schema/values";
import { quote, row, truncate } from "./card";
import { clipInput, defineNode, opt, port } from "./define";
import { int, ms, text } from "./params";
import { fontStack } from "../fonts";
import { humanize } from "../format";
import { msToFrames } from "../timeline/time";

const base = { category: "Text & overlays", phase: "compile" } as const;

const captionsFromTextNode = defineNode({
  ...base,
  type: "text.captionsFromText",
  title: "Captions from script",
  icon: "captions",
  description: "Turns your script into timed captions at a natural speaking pace.",
  inputs: {},
  outputs: { captions: port("captions", "Captions") },
  card: ({ params: p }) => [
    p.script.trim()
      ? row("captions", quote(p.script))
      : row("captions", "Write a script", { tone: "warn" }),
    row("gauge", `${p.wordsPerMinute} words/min`),
  ],
  params: z.object({
    script: text("Record a real flow. Turn it into a cinematic demo.", "Script", "textarea"),
    wordsPerMinute: int(165, "Speaking pace", { min: 60, max: 400, unit: "words/min" }),
    startMs: ms(0, "Start at"),
    linePauseMs: ms(350, "Pause between sentences", 5000),
  }),
  compile: ({ params }) => {
    const perWord = 60_000 / params.wordsPerMinute;
    const captions: CaptionsValue["captions"] = [];
    let t = params.startMs;
    for (const word of params.script.split(/\s+/).filter(Boolean)) {
      captions.push({ text: word, startMs: t, endMs: t + perWord });
      t += perWord + (/[.!?]$/.test(word) ? params.linePauseMs : 0);
    }
    return { captions: { type: "captions", captions } };
  },
});

const captionsNode = defineNode({
  ...base,
  type: "text.captions",
  title: "Show captions",
  icon: "text-quote",
  description: "Displays captions over the video, word by word, karaoke or line by line.",
  inputs: { captions: port("captions", "Captions"), clip: opt("clip", "Over clip") },
  outputs: { clip: port("clip", "Clip") },
  card: ({ params: p }) => [
    row("captions", truncate([p.style, p.position].filter(Boolean).map(humanize).join(" · "))),
  ],
  params: captionsSchema,
  compile: ({ nodeId, inputs, params, settings }) => {
    const caps = asValue(inputs.captions, "captions");
    const under = clipInput(inputs);
    if (!caps) return { clip: under ?? emptyClip(EMPTY_CLIP_FRAMES) };
    const end = caps.captions.at(-1)?.endMs ?? 1000;
    const layer = itemClip(
      nodeId,
      CAPTIONS_ELEMENT,
      {
        ...params,
        fontFamily: params.fontFamily ? fontStack(params.fontFamily) : undefined,
        captions: caps.captions,
      },
      msToFrames(end + 500, settings.fps),
    );
    return {
      clip: under
        ? { ...stackClips(nodeId, [under, layer]), durationInFrames: under.durationInFrames }
        : layer,
    };
  },
});

export const textNodes = [captionsFromTextNode, captionsNode];
