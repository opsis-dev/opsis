import { describe, expect, it } from "vitest";
import { getKeystrokeDuration } from "#video/elements/overlay/keystroke.tsx";
import { commandMenuPlayEnd } from "#video/elements/scene/command-menu/index.tsx";
import { punchLinesDuration } from "#video/elements/text/punch-lines.tsx";
import { wordPushLength } from "#video/elements/text/word-push.tsx";
import { getAgentRunDuration } from "#video/elements/ui/agent-run.tsx";
import { checkListDuration } from "#video/elements/ui/check-list.tsx";
import { getCodeMorphDuration } from "#video/elements/ui/code-morph.tsx";
import { getSearchRevealDuration } from "#video/elements/ui/search-reveal.tsx";
import CASES from "#video/registry/length-cases.generated.json";

type Settings = Record<string, unknown>;

type Steps<T> = T[];

const OWN_LENGTH: Record<string, (s: Settings) => number> = {
  "word-push": (s) => wordPushLength(String(s.text)),
  "search-reveal": (s) =>
    getSearchRevealDuration({
      text: String(s.text),
      framesPerCharacter: Number(s.framesPerCharacter),
    }),
  "check-list": (s) =>
    checkListDuration(s.items as Steps<{ text: string; checked: boolean }>, {
      step: Number(s.step),
      itemGap: Number(s.itemGap),
      perStep: Number(s.perStep),
      closeGap: Number(s.closeGap),
    }),
  "code-morph": (s) =>
    getCodeMorphDuration({
      steps: s.steps as Steps<{ at: number; code: string }>,
      morphDuration: Number(s.morphDuration),
    }),
  keystroke: (s) =>
    getKeystrokeDuration({
      steps: s.steps as Steps<{ at: number; keys: string }>,
      linger: Number(s.linger),
    }),
  "agent-run": (s) =>
    getAgentRunDuration({
      title: String(s.runTitle),
      prompt: String(s.runPrompt),
      plan: s.runPlan as string[],
      steps: s.runSteps as Steps<{ name: string; duration: number }>,
      answer: String(s.runAnswer),
    }),
  "command-menu": (s) => commandMenuPlayEnd(String(s.query)),
  "punch-lines": (s) => punchLinesDuration(String(s.script), String(s.holds)),
};

const ROUNDED: Record<string, (growth: number) => number> = {
  "command-menu": Math.ceil,
};

const TOLERANCE: Record<string, (expected: number) => number> = {
  keystroke: () => 4,
  "agent-run": (expected) => expected * 0.2,
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
