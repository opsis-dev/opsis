import { z } from "zod";
import { emptyClip, isClip, itemClip, msToFrames, stackClips } from "../compile/clip";
import { getElementMeta } from "../elements";
import type { CaptionsValue } from "../schema/values";
import { clip as clipText, quote, row } from "./card";
import { defineNode, opt, port } from "./define";
import { ms, text } from "./params";
import { fontStack } from "../fonts";

export const captionsFromTextNode = defineNode({
  type: "text.captionsFromText",
  title: "Captions from script",
  category: "Text & overlays",
  icon: "captions",
  phase: "compile",
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
    wordsPerMinute: z
      .number()
      .int()
      .min(60)
      .max(400)
      .default(165)
      .meta({ description: "Speaking pace", unit: "words/min" }),
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

const captionsMeta = getElementMeta("core/captions")!;

export const captionsNode = defineNode({
  type: "text.captions",
  title: "Show captions",
  category: "Text & overlays",
  icon: "text-quote",
  phase: "compile",
  description: "Displays captions over the video, word by word, karaoke or line by line.",
  inputs: { captions: port("captions", "Captions"), clip: opt("clip", "Over clip") },
  outputs: { clip: port("clip", "Clip") },
  card: ({ params: p }) => {
    const style = String((p as Record<string, unknown>).style ?? "");
    const position = String((p as Record<string, unknown>).position ?? "");
    return [
      row(
        "captions",
        clipText(
          [style, position]
            .filter(Boolean)
            .map((s) => s.charAt(0).toUpperCase() + s.slice(1))
            .join(" · ") || "Captions",
        ),
      ),
    ];
  },
  params: captionsMeta.schema,
  compile: ({ nodeId, inputs, params, settings }) => {
    const caps = inputs.captions as CaptionsValue | undefined;
    const base = isClip(inputs.clip) ? inputs.clip : undefined;
    if (!caps) return { clip: base ?? emptyClip(30) };
    const end = caps.captions.at(-1)?.endMs ?? 1000;
    const layer = itemClip(
      nodeId,
      "core/captions",
      {
        ...params,
        fontFamily: params.fontFamily
          ? fontStack(String(params.fontFamily), { families: [] })
          : undefined,
        captions: caps.captions,
      },
      msToFrames(end + 500, settings.fps),
    );
    return {
      clip: base
        ? { ...stackClips(nodeId, [base, layer]), durationInFrames: base.durationInFrames }
        : layer,
    };
  },
});

export const textNodes = [captionsFromTextNode, captionsNode];
