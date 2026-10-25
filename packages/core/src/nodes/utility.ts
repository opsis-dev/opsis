import { z } from "zod";
import { missing, row, truncate } from "./card";
import { defineNode } from "./define";
import { more, text } from "./params";

export const GROUP_TYPE = "core.group";
export const NOTE_TYPE = "core.note";

export const groupParams = z.object({
  title: text("Group", "Title"),
  color: more(text("#a78bfa", "Colour"), { widget: "color" }),
});

export const noteParams = z.object({ text: text("Notes…", "Text", "textarea") });

const groupNode = defineNode({
  type: GROUP_TYPE,
  title: "Group",
  category: "Utility",
  phase: "compile",
  hidden: true,
  description: "Boxes nodes together. Select nodes, then Group.",
  inputs: {},
  outputs: {},
  params: groupParams,
});

const noteNode = defineNode({
  type: NOTE_TYPE,
  title: "Note",
  category: "Utility",
  description: "A sticky note on the canvas. Not part of the video.",
  phase: "compile",
  cardHeight: 140,
  inputs: {},
  outputs: {},
  params: noteParams,
  card: ({ params: p }) => {
    const first = p.text.split("\n").find((line) => line.trim());
    return [
      first ? row("sticky-note", truncate(first.trim())) : missing("sticky-note", "Write a note"),
    ];
  },
});

export const utilityNodes = [groupNode, noteNode];
