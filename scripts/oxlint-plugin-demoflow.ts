type Node = { type: string; value?: unknown; quasis?: Array<{ value: { raw: string } }> };
type Context = { report: (d: { node: Node; message: string }) => void };

const ANIMATE_CLASS =
  /(^|\s)(motion-safe:|motion-reduce:|hover:|group-hover:)?animate-[a-z0-9[\]-]+/;
const MESSAGE =
  "Tailwind `animate-*` classes animate on the wall clock and do not render in Remotion. Derive the style from useCurrentFrame() instead.";

const noAnimateClass = {
  meta: {
    type: "problem",
    docs: { description: "Disallow Tailwind animate-* classes inside Remotion compositions" },
  },
  create(context: Context) {
    const check = (node: Node, text: string) => {
      if (ANIMATE_CLASS.test(text)) context.report({ node, message: MESSAGE });
    };
    return {
      Literal(node: Node) {
        if (typeof node.value === "string") check(node, node.value);
      },
      TemplateLiteral(node: Node) {
        for (const q of node.quasis ?? []) check(node, q.value.raw);
      },
    };
  },
};

export default {
  meta: { name: "demoflow" },
  rules: { "no-animate-class": noAnimateClass },
};
