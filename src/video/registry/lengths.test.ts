import { describe, expect, it } from "vitest";
import { ELEMENTS } from "../../shared/elements";
import { animationLength } from "../../shared/elements/lengths";
import type { ElementMeta } from "../../shared/elements/types";
import { getAgentRunDuration } from "../elements/remocn/agent-run";
import { checkListDuration } from "../elements/remocn/check-list";
import { getCodeMorphDuration } from "../elements/remocn/code-morph";
import { getKeystrokeDuration } from "../elements/remocn/keystroke";
import { getSearchRevealDuration } from "../elements/remocn/search-reveal";
import { wordPushLength } from "../elements/remocn/word-push";
import { punchLinesDuration } from "../elements/snap-cn/punch-lines";

type Settings = Record<string, unknown>;

const meta = (key: string): ElementMeta => {
  const found = ELEMENTS.find((e) => e.key === key);
  if (!found) throw new Error(key);
  return found;
};

const defaultsOf = (key: string): Settings => ({ ...meta(key).defaultProps });

const growth = (key: string, changed: Settings, fps = 30) =>
  animationLength(meta(key), { ...defaultsOf(key), ...changed }, fps) -
  animationLength(meta(key), defaultsOf(key), fps);

describe("element lengths", () => {
  it("keeps the usual length for the usual settings unless that cuts the animation off", () => {
    const lengths = ELEMENTS.map((element) => ({
      key: element.key,
      change: animationLength(element, element.defaultProps, 30) - element.defaultDurationInFrames,
    }));
    expect(lengths.filter((length) => length.change < 0)).toEqual([]);
    expect(lengths.filter((length) => length.change > 0).map((length) => length.key)).toEqual([
      "remocn/terminal-simulator",
      "remotion-ui/notification-stack",
    ]);
  });

  it("grows a word push by as much as its own length does", () => {
    const text = "one two three four five six seven eight nine ten";
    const usual = String(defaultsOf("remocn/word-push").text);
    expect(growth("remocn/word-push", { text })).toBe(wordPushLength(text) - wordPushLength(usual));
  });

  it("grows a search reveal by as much as its own length does", () => {
    const text = "a much longer search phrase to type";
    const usual = { text: String(defaultsOf("remocn/search-reveal").text) };
    expect(growth("remocn/search-reveal", { text, framesPerCharacter: 6 })).toBe(
      getSearchRevealDuration({ text, framesPerCharacter: 6 }) - getSearchRevealDuration(usual),
    );
  });

  it("grows a checklist by as much as its own length does", () => {
    const usual = defaultsOf("remocn/check-list");
    const timing = (s: Settings) => ({
      step: Number(s.step),
      itemGap: Number(s.itemGap),
      perStep: Number(s.perStep),
      closeGap: Number(s.closeGap),
    });
    const items = [
      { text: "Write the script", checked: true },
      { text: "Record every scene in one take", checked: true },
      { text: "Add music", checked: false },
      { text: "Check the captions twice before sending", checked: true },
      { text: "Publish", checked: true },
    ];
    const changed = { ...usual, items, itemGap: 24 };
    expect(growth("remocn/check-list", changed)).toBe(
      checkListDuration(items, timing(changed)) -
        checkListDuration(usual.items as typeof items, timing(usual)),
    );
  });

  it("grows a code morph by as much as its own length does", () => {
    const usual = defaultsOf("remocn/code-morph");
    const steps = [
      { at: 0, code: "a" },
      { at: 40, code: "b" },
      { at: 120, code: "c" },
    ];
    expect(growth("remocn/code-morph", { steps, morphDuration: 50 })).toBe(
      getCodeMorphDuration({ steps, morphDuration: 50 }) -
        getCodeMorphDuration({
          steps: usual.steps as typeof steps,
          morphDuration: Number(usual.morphDuration),
        }),
    );
  });

  it("grows a shortcut sequence about as much as its own length does", () => {
    const usual = defaultsOf("remocn/keystroke");
    const steps = [...(usual.steps as { at: number; keys: string }[]), { at: 260, keys: "mod+s" }];
    const expected =
      getKeystrokeDuration({ steps, linger: 40 }) -
      getKeystrokeDuration({ steps: usual.steps as typeof steps, linger: Number(usual.linger) });
    expect(
      Math.abs(growth("remocn/keystroke", { steps, linger: 40 }) - expected),
    ).toBeLessThanOrEqual(4);
  });

  it("grows an agent run about as much as its own length does", () => {
    const usual = defaultsOf("remocn/agent-run");
    const run = (s: Settings) => ({
      title: String(s.runTitle),
      prompt: String(s.runPrompt),
      plan: s.runPlan as string[],
      steps: s.runSteps as { name: string; duration: number }[],
      answer: String(s.runAnswer),
    });
    const changed = {
      ...usual,
      runAnswer: `${String(usual.runAnswer)}\n\n${String(usual.runAnswer)}`,
      runSteps: [...(usual.runSteps as unknown[]), { name: "Write the summary", duration: 2 }],
    };
    const expected = getAgentRunDuration(run(changed)) - getAgentRunDuration(run(usual));
    const actual = growth("remocn/agent-run", changed);
    expect(Math.abs(actual - expected)).toBeLessThanOrEqual(expected * 0.2);
  });

  it("grows punch lines by as much as their own length does", () => {
    const usual = defaultsOf("snapcn/punch-lines");
    const script = "One. | Two. | Three. | Four. | Five.";
    expect(growth("snapcn/punch-lines", { script, holds: "40, 60" })).toBe(
      punchLinesDuration(script, "40, 60") -
        punchLinesDuration(String(usual.script), String(usual.holds)),
    );
  });

  it("measures timing in seconds at the project's frame rate", () => {
    const element = meta("remotion-ui/changelog-entry");
    expect(animationLength(element, element.defaultProps, 60)).toBe(
      2 * element.defaultDurationInFrames,
    );
  });

  it("keeps elements made for 30 frames a second as long in seconds at 60", () => {
    const lengths = ["snapcn/status-cycle", "snapcn/laptop-frame"].map((key) => {
      const element = meta(key);
      return animationLength(element, element.defaultProps, 60) / element.defaultDurationInFrames;
    });
    expect(lengths).toEqual([2, 2]);
  });

  it("plays faster at a higher speed", () => {
    const element = meta("remocn/soft-blur-in");
    const usual = animationLength(element, element.defaultProps, 30);
    expect(animationLength(element, { ...element.defaultProps, speed: 2 }, 30)).toBeLessThan(usual);
  });
});
