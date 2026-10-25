import { nonBlankString } from "#core/guards";
import type { Action } from "#core/schema/actions";

const naming =
  (verb: string, alone = verb) =>
  (what: string | undefined) =>
    what ? `${verb} ${what}` : alone;

const ACTION_LABELS: Partial<Record<Action["kind"], (what: string | undefined) => string>> = {
  click: naming("Click"),
  type: naming("Type"),
  hover: naming("Hover"),
  scroll: () => "Scroll",
  navigate: naming("Go to", "Go to page"),
  chapter: (what) => what ?? "Chapter",
  key: naming("Press", "Press keys"),
  drag: naming("Drag"),
};

export function actionLabel(action: Pick<Action, "kind" | "label" | "text">): string | null {
  const label = ACTION_LABELS[action.kind];
  return label ? label(nonBlankString(action.label) ?? nonBlankString(action.text)) : null;
}
