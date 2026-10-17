import { z } from "zod";
import { defineNode } from "./define";
import { more, text } from "./params";

const groupNode = defineNode({
  type: "core.group",
  title: "Group",
  category: "Utility",
  icon: "group",
  phase: "compile",
  hidden: true,
  description: "Boxes nodes together. Select nodes, then Group.",
  inputs: {},
  outputs: {},
  params: z.object({
    title: text("Group", "Title"),
    color: more(text("#a78bfa", "Colour"), { widget: "color" }),
  }),
});

const noteNode = defineNode({
  type: "core.note",
  title: "Note",
  category: "Utility",
  icon: "sticky-note",
  description: "A sticky note on the canvas. Not part of the video.",
  phase: "compile",
  inputs: {},
  outputs: {},
  params: z.object({ text: text("Notes…", "Text", "textarea") }),
});

export const utilityNodes = [groupNode, noteNode];
