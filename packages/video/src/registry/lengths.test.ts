import { describe, expect, it } from "vitest";
import { getAgentRunDuration } from "#video/elements/remocn/agent-run.tsx";
import { checkListDuration } from "#video/elements/remocn/check-list.tsx";
import { getCodeMorphDuration } from "#video/elements/remocn/code-morph.tsx";
import { commandMenuPlayEnd } from "#video/elements/remocn/command-menu.tsx";
import { getKeystrokeDuration } from "#video/elements/remocn/keystroke.tsx";
import { getSearchRevealDuration } from "#video/elements/remocn/search-reveal.tsx";
import { wordPushLength } from "#video/elements/remocn/word-push.tsx";
import { punchLinesDuration } from "#video/elements/snap-cn/punch-lines.tsx";
import CASES from "#video/registry/length-cases.generated.json";

type Settings = Record<string, unknown>;

type Steps<T> = T[];

const OWN_LENGTH: Record<string, (s: Settings) => number> = {
  "remocn/word-push": (s) => wordPushLength(String(s.text)),
  "remocn/search-reveal": (s) =>
    getSearchRevealDuration({
      text: String(s.text),
      framesPerCharacter: Number(s.framesPerCharacter),
    }),
  "remocn/check-list": (s) =>
    checkListDuration(s.items as Steps<{ text: string; checked: boolean }>, {
      step: Number(s.step),
      itemGap: Number(s.itemGap),
      perStep: Number(s.perStep),
      closeGap: Number(s.closeGap),
    }),
  "remocn/code-morph": (s) =>
    getCodeMorphDuration({
      steps: s.steps as Steps<{ at: number; code: string }>,
      morphDuration: Number(s.morphDuration),
    }),
  "remocn/keystroke": (s) =>
    getKeystrokeDuration({
      steps: s.steps as Steps<{ at: number; keys: string }>,
      linger: Number(s.linger),
    }),
  "remocn/agent-run": (s) =>
    getAgentRunDuration({
      title: String(s.runTitle),
      prompt: String(s.runPrompt),
      plan: s.runPlan as string[],
      steps: s.runSteps as Steps<{ name: string; duration: number }>,
      answer: String(s.runAnswer),
    }),
  "remocn/command-menu": (s) => commandMenuPlayEnd(String(s.query)),
  "snapcn/punch-lines": (s) => punchLinesDuration(String(s.script), String(s.holds)),
};

const ROUNDED: Record<string, (growth: number) => number> = {
  "remocn/command-menu": Math.ceil,
};

const TOLERANCE: Record<string, (expected: number) => number> = {
  "remocn/keystroke": () => 4,
  "remocn/agent-run": (expected) => expected * 0.2,
};

describe("element lengths", () => {
  it.each(CASES)(
    "grows $key by as much as its own length does",
    ({ key, usual, settings, growth }) => {
      const own = OWN_LENGTH[key];
      const round = ROUNDED[key] ?? ((difference: number) => difference);
      const expected = round(own(settings) - own(usual));
      expect(Math.abs(growth - expected)).toBeLessThanOrEqual(TOLERANCE[key]?.(expected) ?? 0);
    },
  );
});
