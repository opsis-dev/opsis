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

type Node = { type: string; [key: string]: unknown };

const isNode = (value: unknown): value is Node =>
  typeof value === "object" && value !== null && "type" in value;

const isIdentifier = (node: unknown, name?: string): boolean =>
  isNode(node) && node.type === "Identifier" && (name === undefined || node.name === name);

const isMember = (node: unknown, property: string): node is Node =>
  isNode(node) &&
  node.type === "MemberExpression" &&
  !node.computed &&
  isIdentifier(node.property, property);

const isLiteral = (node: unknown, value: unknown): boolean =>
  isNode(node) && node.type === "Literal" && node.value === value;

const callRule = (
  description: string,
  message: string,
  matches: (call: Node) => boolean,
): Rule => ({
  meta: { type: "suggestion", docs: { description } },
  create(context) {
    return {
      CallExpression(node) {
        if (matches(node as unknown as Node)) context.report({ node, message });
      },
    };
  },
});

const paramName = (fn: Node): string | null => {
  const params = fn.params as unknown[];
  const first = params[0];
  return params.length === 1 && isNode(first) && first.type === "Identifier"
    ? String(first.name)
    : null;
};

const comparesIdOf = (fn: unknown): boolean => {
  if (!isNode(fn) || fn.type !== "ArrowFunctionExpression") return false;
  const name = paramName(fn);
  const body = fn.body;
  if (!name || !isNode(body) || body.type !== "BinaryExpression" || body.operator !== "===")
    return false;
  return [body.left, body.right].some(
    (side) => isMember(side, "id") && isIdentifier(side.object, name),
  );
};

const useFindById = callRule(
  "Find an item by id through findById",
  "Use findById from src/shared/lists.ts.",
  (call) => isMember(call.callee, "find") && comparesIdOf((call.arguments as unknown[])[0]),
);

const useOnAbort = callRule(
  "Listen for an abort through onAbort",
  "Use onAbort from src/shared/errors.ts.",
  (call) =>
    (isMember(call.callee, "addEventListener") || isMember(call.callee, "removeEventListener")) &&
    isLiteral((call.arguments as unknown[])[0], "abort"),
);

const useParseJsonWith = callRule(
  "Parse and check JSON through parseJsonWith",
  "Use parseJsonWith from src/shared/guards.ts.",
  (call) => {
    const arg = (call.arguments as unknown[])[0];
    return (
      isMember(call.callee, "safeParse") &&
      isNode(arg) &&
      ((arg.type === "ConditionalExpression" && isMember(arg.test, "ok")) ||
        (arg.type === "CallExpression" &&
          isMember(arg.callee, "parse") &&
          isIdentifier((arg.callee as Node).object, "JSON")))
    );
  },
);

const useRoot = callRule(
  "Use the shared repository root in scripts",
  "Use ROOT from scripts/lib/tool-config.ts.",
  (call) => {
    const args = call.arguments as unknown[];
    return (
      isMember(call.callee, "resolve") &&
      isMember(args[0], "dirname") &&
      isNode(args[0]) &&
      isNode(args[0].object) &&
      args[0].object.type === "MetaProperty"
    );
  },
);

const TYPE_GUARDS: Record<string, string> = {
  string: "isString",
  number: "isNumber",
  boolean: "isBoolean",
};

const useTypeGuards: Rule = {
  meta: { type: "suggestion", docs: { description: "Use the shared primitive type guards" } },
  create(context) {
    return {
      ArrowFunctionExpression(node) {
        const fn = node as unknown as Node;
        const name = paramName(fn);
        const body = fn.body;
        if (!name || !isNode(body) || body.type !== "BinaryExpression" || body.operator !== "===")
          return;
        const sides = [body.left, body.right];
        const typed = sides.find(
          (side) =>
            isNode(side) &&
            side.type === "UnaryExpression" &&
            side.operator === "typeof" &&
            isIdentifier(side.argument, name),
        );
        const kind = sides.find((side) => isNode(side) && side.type === "Literal");
        const guard = isNode(kind) ? TYPE_GUARDS[String(kind.value)] : undefined;
        if (typed && guard)
          context.report({ node, message: `Use ${guard} from src/shared/guards.ts.` });
      },
    };
  },
};

const PERCENT_AFTER = /^\s?%/;

const usePercentText: Rule = {
  meta: { type: "suggestion", docs: { description: "Write percentages through percentText" } },
  create(context) {
    return {
      TemplateLiteral(node) {
        const quasis = node.quasis;
        const rounded = node.expressions.some(
          (expression, i) =>
            isNode(expression) &&
            expression.type === "CallExpression" &&
            isMember(expression.callee, "round") &&
            PERCENT_AFTER.test(quasis[i + 1]?.value.raw ?? ""),
        );
        if (rounded)
          context.report({ node, message: "Use percentText from src/shared/format.ts." });
      },
    };
  },
};

export default {
  meta: { name: "demoflow" },
  rules: {
    "no-animate-class": noAnimateClass,
    "use-find-by-id": useFindById,
    "use-on-abort": useOnAbort,
    "use-parse-json-with": useParseJsonWith,
    "use-percent-text": usePercentText,
    "use-root": useRoot,
    "use-type-guards": useTypeGuards,
  },
};
