import { interpolate, random, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { CLAMPED } from "#video/core/anim.ts";
import {
  defaultExitFrames,
  resolveSpringConfig,
  type MotionSpring,
} from "#video/lib/motion/motion-primitive.ts";
import { EASING_ENTER, EASING_EXIT } from "#video/lib/motion/timing.ts";

export type SplitMode = "chars" | "words" | "lines";

export type SplitOrder = "start" | "end" | "center" | "edges" | "random";

export type SplitUnit = {
  text: string;
  index: number;
  total: number;
  lineIndex: number;
  wordIndex: number;
  indexInWord: number;
  indexInLine: number;
};

export type SplitWord = {
  text: string;
  index: number;
  lineIndex: number;
  indexInLine: number;
  units: SplitUnit[];
};

export type SplitLine = {
  text: string;
  index: number;
  words: SplitWord[];
  units: SplitUnit[];
};

export type SplitResult = {
  mode: SplitMode;
  lines: SplitLine[];
  words: SplitWord[];
  units: SplitUnit[];
  total: number;
};

function toLines(text: string): string[] {
  return text.split(/\r?\n/);
}

function toWords(line: string): string[] {
  return line.split(/\s+/).filter((word) => word.length > 0);
}

function toChars(word: string): string[] {
  return Array.from(word);
}

export function splitText(text: string, mode: SplitMode = "chars"): SplitResult {
  const units: SplitUnit[] = [];
  const words: SplitWord[] = [];
  const lines: SplitLine[] = [];

  const rawLines = toLines(text);

  for (let lineIndex = 0; lineIndex < rawLines.length; lineIndex += 1) {
    const rawLine = rawLines[lineIndex];
    const lineWords: SplitWord[] = [];
    const lineUnits: SplitUnit[] = [];

    const rawWords = mode === "lines" ? [rawLine.trim()].filter(Boolean) : toWords(rawLine);

    for (let indexInLine = 0; indexInLine < rawWords.length; indexInLine += 1) {
      const rawWord = rawWords[indexInLine];
      const wordUnits: SplitUnit[] = [];
      const pieces = mode === "chars" ? toChars(rawWord) : [rawWord];

      const word: SplitWord = {
        text: rawWord,
        index: words.length,
        lineIndex,
        indexInLine,
        units: wordUnits,
      };

      for (let indexInWord = 0; indexInWord < pieces.length; indexInWord += 1) {
        const unit: SplitUnit = {
          text: pieces[indexInWord],
          index: units.length,
          total: 0,
          lineIndex,
          wordIndex: word.index,
          indexInWord,
          indexInLine: lineUnits.length,
        };
        units.push(unit);
        lineUnits.push(unit);
        wordUnits.push(unit);
      }

      words.push(word);
      lineWords.push(word);
    }

    lines.push({
      text: rawLine,
      index: lineIndex,
      words: lineWords,
      units: lineUnits,
    });
  }

  const total = units.length;
  for (const unit of units) {
    unit.total = total;
  }

  return { mode, lines, words, units, total };
}

export function staggerRanks(total: number, order: SplitOrder = "start", seed = 1): number[] {
  if (total <= 0) {
    return [];
  }

  const indices = Array.from({ length: total }, (_, index) => index);
  const middle = (total - 1) / 2;

  switch (order) {
    case "end":
      return indices.map((index) => total - 1 - index);
    case "center":
      return indices.map((index) => Math.round(Math.abs(index - middle)));
    case "edges":
      return indices.map((index) => Math.round(middle - Math.abs(index - middle)));
    case "random": {
      const shuffled = [...indices].sort((a, b) => random(`${seed}:${a}`) - random(`${seed}:${b}`));
      const ranks = Array.from({ length: total }, () => 0);
      shuffled.forEach((originalIndex, rank) => {
        ranks[originalIndex] = rank;
      });
      return ranks;
    }
    default:
      return indices;
  }
}

export type SplitTimingOptions = {
  mode?: SplitMode;
  staggerInFrames?: number;
  durationInFrames?: number;
  delayInFrames?: number;
  order?: SplitOrder;
  seed?: number;
  spring?: MotionSpring;
  exit?: boolean;
  exitInFrames?: number;
  exitAtInFrames?: number;
  exitStaggerInFrames?: number;
  frame?: number;
};

export const SPLIT_STAGGER_DEFAULTS: Record<SplitMode, number> = {
  chars: 2,
  words: 4,
  lines: 7,
};

export const SPLIT_DURATION_DEFAULTS: Record<SplitMode, number> = {
  chars: 20,
  words: 24,
  lines: 28,
};

export type SplitUnitState = SplitUnit & {
  delayInFrames: number;
  rank: number;
  enter: number;
  exit: number;
  displace: number;
  opacity: number;
  value: number;
};

export type SplitTextState = {
  mode: SplitMode;
  lines: Array<{
    text: string;
    index: number;
    words: Array<{
      text: string;
      index: number;
      lineIndex: number;
      indexInLine: number;
      units: SplitUnitState[];
    }>;
  }>;
  units: SplitUnitState[];
  total: number;
  lastEnterFrame: number;
};

const ENTER_OPACITY_LEAD = 0.55;
const EXIT_OPACITY_LEAD = 0.7;

export function useSplitText({
  text,
  mode = "chars",
  staggerInFrames,
  durationInFrames,
  delayInFrames = 0,
  order = "start",
  seed = 1,
  spring: springProp,
  exit = false,
  exitInFrames,
  exitAtInFrames,
  exitStaggerInFrames,
  frame: frameOverride,
}: SplitTimingOptions & { text: string }): SplitTextState {
  const currentFrame = useCurrentFrame();
  const frame = frameOverride ?? currentFrame;
  const { fps, durationInFrames: windowFrames } = useVideoConfig();

  const stagger = staggerInFrames ?? SPLIT_STAGGER_DEFAULTS[mode];
  const duration = Math.max(1, Math.round(durationInFrames ?? SPLIT_DURATION_DEFAULTS[mode]));
  const delay = Math.max(0, Math.round(delayInFrames));
  const exitFrames =
    exitInFrames === undefined
      ? defaultExitFrames(duration)
      : Math.max(1, Math.round(exitInFrames));
  const exitStagger = exitStaggerInFrames ?? stagger;

  const split = splitText(text, mode);
  const ranks = staggerRanks(split.total, order, seed);
  const maxRank = ranks.length > 0 ? Math.max(...ranks) : 0;
  const lastEnterFrame = delay + maxRank * stagger + duration;

  const wantsExit = exit || exitInFrames !== undefined || exitAtInFrames !== undefined;
  const hasWindow = Number.isFinite(windowFrames);
  const requestedExit =
    exitAtInFrames ?? (hasWindow ? windowFrames - exitFrames - maxRank * exitStagger : null);
  const exitStart =
    wantsExit && requestedExit !== null ? Math.max(requestedExit, lastEnterFrame) : null;

  const springConfig = springProp ? resolveSpringConfig(springProp) : null;

  function stateFor(unit: SplitUnit): SplitUnitState {
    const rank = ranks[unit.index] ?? unit.index;
    const start = delay + rank * stagger;

    const enterTime = interpolate(frame, [start, start + duration], [0, 1], CLAMPED);
    const enter = springConfig
      ? spring({
          frame,
          fps,
          config: springConfig,
          delay: start,
          durationInFrames: duration,
        })
      : interpolate(enterTime, [0, 1], [0, 1], {
          easing: EASING_ENTER,
          ...CLAMPED,
        });
    const enterOpacity = interpolate(enterTime, [0, ENTER_OPACITY_LEAD], [0, 1], CLAMPED);

    if (exitStart === null) {
      return {
        ...unit,
        rank,
        delayInFrames: start,
        enter,
        exit: 0,
        displace: 1 - enter,
        opacity: enterOpacity,
        value: enter,
      };
    }

    const unitExitStart = exitStart + rank * exitStagger;
    const exitTime = interpolate(
      frame,
      [unitExitStart, unitExitStart + exitFrames],
      [0, 1],
      CLAMPED,
    );
    const exitProgress = interpolate(exitTime, [0, 1], [0, 1], {
      easing: EASING_EXIT,
      ...CLAMPED,
    });
    const exitOpacity = interpolate(exitTime, [0, EXIT_OPACITY_LEAD], [1, 0], CLAMPED);

    return {
      ...unit,
      rank,
      delayInFrames: start,
      enter,
      exit: exitProgress,
      displace: exitProgress > 0 ? exitProgress : 1 - enter,
      opacity: enterOpacity * exitOpacity,
      value: enter * (1 - exitProgress),
    };
  }

  const states = split.units.map(stateFor);

  return {
    mode,
    lines: split.lines.map((line) => ({
      text: line.text,
      index: line.index,
      words: line.words.map((word) => ({
        text: word.text,
        index: word.index,
        lineIndex: word.lineIndex,
        indexInLine: word.indexInLine,
        units: word.units.map((unit) => states[unit.index]),
      })),
    })),
    units: states,
    total: split.total,
    lastEnterFrame,
  };
}
