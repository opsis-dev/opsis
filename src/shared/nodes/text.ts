import { z } from "zod";
import { EMPTY_CLIP_FRAMES, emptyClip, itemClip, stackClips } from "../compile/clip";
import type { StyleSteps } from "../compile/style";
import { TEXT_COLOR } from "../elements/colors";
import { CAPTIONS_ELEMENT, captionsSchema } from "../elements/core";
import { fontProp } from "../fonts";
import { formatDuration, humanize } from "../format";
import { asValue, type CaptionsValue } from "../schema/values";
import { framesToMs, msToFrames } from "../timeline/time";
import { quotedRow, row, truncate } from "./card";
import { annotationNode } from "./cursor";
import { clipInput, defineNode, nodeBase, opt, port } from "./define";
import { int, more, ms, requiredText } from "./params";

const base = nodeBase("Text & overlays", "compile");

const MS_PER_MINUTE = 60_000;
const ENDS_SENTENCE = /[.!?…。！？]$/u;
const CLOSING_MARKS = /["'”’)\]}»」』]+$/u;
const ABBREVIATION = /^(?:\p{L}\.){2,}$|^(?:mr|mrs|ms|dr|st|vs|etc|inc|ltd|no|fig|approx)\.$/iu;
const UNSPACED_SCRIPT = /[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}\p{Script=Thai}]/u;
const PARAGRAPH_BREAK = /\n\s*\n/;

type ScriptPace = { wordsPerMinute: number; startMs: number; linePauseMs: number };

function wordsOf(chunk: string): string[] {
  if (!UNSPACED_SCRIPT.test(chunk)) return [chunk];
  const words: string[] = [];
  for (const { segment, isWordLike } of new Intl.Segmenter(undefined, {
    granularity: "word",
  }).segment(chunk)) {
    if (isWordLike || !words.length) words.push(segment);
    else words[words.length - 1] += segment;
  }
  return words;
}

export function endsSentence(word: string): boolean {
  const bare = word.replace(CLOSING_MARKS, "");
  return ENDS_SENTENCE.test(bare) && !ABBREVIATION.test(bare);
}

export function scriptToCaptions(script: string, pace: ScriptPace): CaptionsValue["captions"] {
  const perWord = MS_PER_MINUTE / pace.wordsPerMinute;
  const captions: CaptionsValue["captions"] = [];
  let t = pace.startMs;
  for (const paragraph of script.split(PARAGRAPH_BREAK)) {
    const words = paragraph.split(/\s+/).filter(Boolean).flatMap(wordsOf);
    for (const [i, word] of words.entries()) {
      captions.push({ text: word, startMs: t, endMs: t + perWord });
      const last = i === words.length - 1;
      const pause = last ? pace.linePauseMs * 2 : endsSentence(word) ? pace.linePauseMs : 0;
      t += perWord + pause;
    }
  }
  return captions;
}

const EMPTY_CAPTIONS_MS = 1000;
const CAPTION_TAIL_MS = 500;

const tooltip = annotationNode({
  type: "text.tooltip",
  title: "Tooltip",
  kind: "tooltip",
  category: base.category,
  icon: "message-square",
  description: "A tooltip that stays attached to an element as the camera moves.",
  defaults: { text: "Click to continue", color: "#18181b" },
  label: { fontSize: 18, textColor: TEXT_COLOR },
});

const calloutArrow = annotationNode({
  type: "text.calloutArrow",
  title: "Callout arrow",
  kind: "callout",
  category: base.category,
  icon: "move-up-right",
  description: "A hand-drawn arrow pointing at an element, with a label.",
  defaults: { text: "New!", color: "#f43f5e" },
  label: { fontSize: 30 },
});

const captionsFromTextNode = defineNode({
  ...base,
  type: "text.captionsFromText",
  title: "Captions from script",
  description: "Turns your script into timed captions at a natural speaking pace.",
  inputs: {},
  outputs: { captions: port("captions", "Captions") },
  card: ({ params: p }) => [
    quotedRow("captions", p.script, "Write a script"),
    row("gauge", `${p.wordsPerMinute} words/min`),
  ],
  params: z.object({
    script: more(
      requiredText(
        "Script",
        "Write a script",
        "Record a real flow. Turn it into a cinematic demo.",
      ),
      { widget: "textarea" },
    ),
    wordsPerMinute: int(165, "Speaking pace", { min: 60, max: 400, unit: "words/min" }),
    startMs: ms(0, "Start at"),
    linePauseMs: ms(350, "Pause between sentences", 5000),
  }),
  compile: ({ params }) => ({
    captions: { type: "captions", captions: scriptToCaptions(params.script, params) },
  }),
});

const captionsNode = defineNode({
  ...base,
  type: "text.captions",
  title: "Show captions",
  description: "Displays captions over the video, word by word, karaoke or line by line.",
  inputs: { captions: port("captions", "Captions"), clip: opt("clip", "Over clip") },
  outputs: { clip: port("clip", "Clip") },
  card: ({ params: p, inputCaptionsMs, inputFrames, settings }) => {
    const spoken = inputCaptionsMs?.captions;
    const clipMs = inputFrames?.clip && framesToMs(inputFrames.clip, settings.fps);
    return [
      row("captions", truncate([p.style, p.position].filter(Boolean).map(humanize).join(" · "))),
      spoken !== undefined &&
        clipMs &&
        spoken > clipMs &&
        row(
          "triangle-alert",
          `Runs ${formatDuration(spoken - clipMs)} past the clip, which cuts it off`,
          {
            tone: "warn",
          },
        ),
    ];
  },
  params: captionsSchema,
  compile: ({ nodeId, inputs, params, settings }) => {
    const caps = asValue(inputs.captions, "captions");
    const under = clipInput(inputs);
    if (!caps) return { clip: under ?? emptyClip(EMPTY_CLIP_FRAMES) };
    const end = caps.captions.at(-1)?.endMs ?? EMPTY_CAPTIONS_MS;
    const layer = itemClip(
      nodeId,
      CAPTIONS_ELEMENT,
      {
        ...params,
        fontFamily: fontProp(params.fontFamily),
        captions: caps.captions,
      },
      msToFrames(end + CAPTION_TAIL_MS, settings.fps),
    );
    return {
      clip: under
        ? { ...stackClips(nodeId, [under, layer]), durationInFrames: under.durationInFrames }
        : layer,
    };
  },
});

export const textNodes = [tooltip.node, calloutArrow.node, captionsFromTextNode, captionsNode];

export const textStyles: StyleSteps = [tooltip.step, calloutArrow.step];
