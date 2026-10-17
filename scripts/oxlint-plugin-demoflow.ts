import type { RuleTester } from "oxlint/plugins-dev";

type Rule = Parameters<RuleTester["run"]>[1];

const ANIMATE_CLASS =
  /(^|\s)(motion-safe:|motion-reduce:|hover:|group-hover:)?animate-[a-z0-9[\]-]+/;
const MESSAGE =
  "Tailwind `animate-*` classes animate on the wall clock and do not render in Remotion. Derive the style from useCurrentFrame() instead.";

const noAnimateClass: Rule = {
  meta: {
    type: "problem",
    docs: { description: "Disallow Tailwind animate-* classes inside Remotion compositions" },
  },
  create(context) {
    return {
      Literal(node) {
        if (typeof node.value === "string" && ANIMATE_CLASS.test(node.value))
          context.report({ node, message: MESSAGE });
      },
      TemplateLiteral(node) {
        if (node.quasis.some((q) => ANIMATE_CLASS.test(q.value.raw)))
          context.report({ node, message: MESSAGE });
      },
    };
  },
};

export default {
  meta: { name: "demoflow" },
  rules: { "no-animate-class": noAnimateClass },
};
