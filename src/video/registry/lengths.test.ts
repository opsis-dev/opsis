import { describe, expect, it } from "vitest";
import { ELEMENTS } from "../../shared/elements";
import { animationLength } from "../../shared/elements/lengths";
import { settingsInFrames } from "../../shared/elements/seconds";
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

const inFrames = (key: string, settings: Settings): Settings =>
  settingsInFrames(meta(key), settings);

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
      "remocn/signup-flow",
      "remocn/release-teaser",
      "remocn/launch-anything",
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
    expect(growth("remocn/search-reveal", { text, framesPerCharacter: 0.2 })).toBe(
      getSearchRevealDuration({ text, framesPerCharacter: 6 }) - getSearchRevealDuration(usual),
    );
  });

  it("grows a checklist by as much as its own length does", () => {
    const usual = defaultsOf("remocn/check-list");
    const key = "remocn/check-list";
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
    const changed = { ...usual, items, itemGap: 0.8 };
    expect(growth(key, changed)).toBe(
      checkListDuration(items, timing(inFrames(key, changed))) -
        checkListDuration(usual.items as typeof items, timing(inFrames(key, usual))),
    );
  });

  it("grows a code morph by as much as its own length does", () => {
    const key = "remocn/code-morph";
    const usual = inFrames(key, defaultsOf(key));
    const changed = {
      steps: [
        { at: 0, code: "a" },
        { at: 2, code: "b" },
        { at: 4, code: "c" },
      ],
      morphDuration: 1.5,
    };
    const framed = inFrames(key, changed);
    type Steps = { at: number; code: string }[];
    expect(growth(key, changed)).toBe(
      getCodeMorphDuration({
        steps: framed.steps as Steps,
        morphDuration: Number(framed.morphDuration),
      }) -
        getCodeMorphDuration({
          steps: usual.steps as Steps,
          morphDuration: Number(usual.morphDuration),
        }),
    );
  });

  it("grows a shortcut sequence about as much as its own length does", () => {
    const key = "remocn/keystroke";
    type Steps = { at: number; keys: string }[];
    const usual = defaultsOf(key);
    const changed = { steps: [...(usual.steps as Steps), { at: 9, keys: "mod+s" }], linger: 1.5 };
    const framed = inFrames(key, changed);
    const framedUsual = inFrames(key, usual);
    const expected =
      getKeystrokeDuration({ steps: framed.steps as Steps, linger: Number(framed.linger) }) -
      getKeystrokeDuration({
        steps: framedUsual.steps as Steps,
        linger: Number(framedUsual.linger),
      });
    expect(Math.abs(growth(key, changed) - expected)).toBeLessThanOrEqual(4);
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
    const key = "snapcn/punch-lines";
    const usual = inFrames(key, defaultsOf(key));
    const script = "One. | Two. | Three. | Four. | Five.";
    expect(growth(key, { script, holds: "1.5, 2" })).toBe(
      punchLinesDuration(script, "45, 60") -
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
    const lengths = ["snapcn/status-cycle", "snapcn/laptop-frame", "remocn/soft-blur-in"].map(
      (key) => {
        const element = meta(key);
        return animationLength(element, element.defaultProps, 60) / element.defaultDurationInFrames;
      },
    );
    expect(lengths).toEqual([2, 2, 2]);
  });

  it("runs an element with no timing of its own for its usual length at its speed", () => {
    const element = meta("remocn/strikethrough-replace");
    const usual = element.defaultDurationInFrames;
    expect(animationLength(element, { ...element.defaultProps, speed: 0.5 }, 30)).toBe(2 * usual);
  });

  it("plays faster at a higher speed", () => {
    const element = meta("remocn/soft-blur-in");
    const usual = animationLength(element, element.defaultProps, 30);
    expect(animationLength(element, { ...element.defaultProps, speed: 2 }, 30)).toBeLessThan(usual);
  });
});
