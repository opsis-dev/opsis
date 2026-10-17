import type { TimelineSpec } from "../schema/timeline";
import { flattenSpec } from "./flatten";
import { chapterTime } from "./time";

type FilmChapter = { title: string; startMs: number; endMs: number };

const MIN_CHAPTER_MS = 10_000;
export const MIN_LISTED_CHAPTERS = 3;
const OPENING_TITLE = "Intro";

export function filmChapters(spec: TimelineSpec): FilmChapter[] {
  const durationMs = (spec.durationInFrames / spec.fps) * 1000;
  const starts = flattenSpec(spec)
    .marks.filter((m) => m.kind === "chapter")
    .map((m) => ({ title: m.label, startMs: Math.round((m.frame / spec.fps) * 1000) }))
    .filter((c) => c.startMs < durationMs)
    .toSorted((a, b) => a.startMs - b.startMs);
  if (starts.length === 0) return [];
  if (starts[0].startMs >= MIN_CHAPTER_MS) starts.unshift({ title: OPENING_TITLE, startMs: 0 });
  else starts[0] = { ...starts[0], startMs: 0 };
  const kept: Array<{ title: string; startMs: number }> = [];
  for (const c of starts) {
    const previous = kept.at(-1);
    if (previous && c.startMs - previous.startMs < MIN_CHAPTER_MS) continue;
    kept.push(c);
  }
  while (kept.length > 1 && durationMs - kept.at(-1)!.startMs < MIN_CHAPTER_MS) kept.pop();
  return kept.map((c, i) => ({
    ...c,
    endMs: Math.round(kept[i + 1]?.startMs ?? durationMs),
  }));
}

export function chapterList(chapters: readonly FilmChapter[]): string {
  return chapters.map((c) => `${chapterTime(c.startMs)} ${c.title}`).join("\n") + "\n";
}

const escapeMeta = (value: string) => value.replace(/[=;#\\\n]/g, (c) => `\\${c}`);

export function chapterMetadata(chapters: readonly FilmChapter[]): string {
  return [
    ";FFMETADATA1",
    ...chapters.flatMap((c) => [
      "[CHAPTER]",
      "TIMEBASE=1/1000",
      `START=${c.startMs}`,
      `END=${c.endMs}`,
      `title=${escapeMeta(c.title)}`,
    ]),
    "",
  ].join("\n");
}
