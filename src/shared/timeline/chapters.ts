import type { TimelineSpec } from "../schema/timeline";
import { flattenSpec } from "./flatten";
import { chapterTime, framesToMs } from "./time";

export type FilmChapter = { title: string; startMs: number; endMs: number };

type ChapterStart = Omit<FilmChapter, "endMs">;

const MIN_CHAPTER_MS = 10_000;
export const MIN_LISTED_CHAPTERS = 3;
const OPENING_TITLE = "Intro";

export function filmChapters(spec: TimelineSpec): FilmChapter[] {
  const durationMs = framesToMs(spec.durationInFrames, spec.fps);
  const starts = flattenSpec(spec)
    .marks.filter((m) => m.kind === "chapter")
    .map((m): ChapterStart => ({ title: m.label, startMs: framesToMs(m.frame, spec.fps) }))
    .filter((c) => c.startMs < durationMs)
    .toSorted((a, b) => a.startMs - b.startMs);
  if (starts.length === 0) return [];
  if (starts[0].startMs >= MIN_CHAPTER_MS) starts.unshift({ title: OPENING_TITLE, startMs: 0 });
  else starts[0] = { ...starts[0], startMs: 0 };
  const kept: ChapterStart[] = [];
  for (const c of starts) {
    const previous = kept.at(-1);
    if (previous && c.startMs - previous.startMs < MIN_CHAPTER_MS) continue;
    kept.push(c);
  }
  while (kept.length > 1 && durationMs - kept[kept.length - 1].startMs < MIN_CHAPTER_MS) kept.pop();
  return kept.map((c, i) => ({ ...c, endMs: kept[i + 1]?.startMs ?? durationMs }));
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
