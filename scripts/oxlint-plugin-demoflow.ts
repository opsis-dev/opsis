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

type Visitor = ReturnType<Exclude<Rule["create"], undefined>>;
type NodeOf<K extends string> = Parameters<Exclude<Visitor[K], undefined>>[0];
type CallExpression = NodeOf<"CallExpression">;
type Node = CallExpression["parent"];
type MaybeNode = Node | null | undefined;
type StaticMember = Extract<Node, { type: "MemberExpression"; computed: false }>;

const isIdentifier = (node: MaybeNode, name?: string): boolean =>
  node?.type === "Identifier" && (name === undefined || node.name === name);

const isMember = (node: MaybeNode, property: string): node is StaticMember =>
  node?.type === "MemberExpression" && !node.computed && isIdentifier(node.property, property);

const isLiteral = (node: MaybeNode, value: unknown): boolean =>
  node?.type === "Literal" && node.value === value;

const callRule = (
  description: string,
  message: string,
  matches: (call: CallExpression) => boolean,
): Rule => ({
  meta: { type: "suggestion", docs: { description } },
  create(context) {
    return {
      CallExpression(node) {
        if (matches(node)) context.report({ node, message });
      },
    };
  },
});

function strictEqualityBody(fn: MaybeNode): { name: string; sides: [Node, Node] } | null {
  if (fn?.type !== "ArrowFunctionExpression" || fn.params.length !== 1) return null;
  const [param] = fn.params;
  const body = fn.body;
  if (param.type !== "Identifier" || body.type !== "BinaryExpression" || body.operator !== "===")
    return null;
  return { name: param.name, sides: [body.left, body.right] };
}

const comparesIdOf = (fn: MaybeNode): boolean => {
  const equality = strictEqualityBody(fn);
  return (
    !!equality &&
    equality.sides.some((side) => isMember(side, "id") && isIdentifier(side.object, equality.name))
  );
};

const useFindById = callRule(
  "Find an item by id through findById",
  "Use findById from src/shared/lists.ts.",
  (call) => isMember(call.callee, "find") && comparesIdOf(call.arguments[0]),
);

const useOnAbort = callRule(
  "Listen for an abort through onAbort",
  "Use onAbort from src/shared/errors.ts.",
  (call) =>
    (isMember(call.callee, "addEventListener") || isMember(call.callee, "removeEventListener")) &&
    isLiteral(call.arguments[0], "abort"),
);

const useParseJsonWith = callRule(
  "Parse and check JSON through parseJsonWith",
  "Use parseJsonWith from src/shared/guards.ts.",
  (call) => {
    const arg: MaybeNode = call.arguments[0];
    return (
      isMember(call.callee, "safeParse") &&
      ((arg?.type === "ConditionalExpression" && isMember(arg.test, "ok")) ||
        (arg?.type === "CallExpression" &&
          isMember(arg.callee, "parse") &&
          isIdentifier(arg.callee.object, "JSON")))
    );
  },
);

const useRoot = callRule(
  "Use the shared repository root in scripts",
  "Use ROOT from scripts/lib/tool-config.ts.",
  (call) => {
    const first: MaybeNode = call.arguments[0];
    return (
      isMember(call.callee, "resolve") &&
      isMember(first, "dirname") &&
      first.object.type === "MetaProperty"
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
        const equality = strictEqualityBody(node);
        if (!equality) return;
        const typed = equality.sides.some(
          (side) =>
            side.type === "UnaryExpression" &&
            side.operator === "typeof" &&
            isIdentifier(side.argument, equality.name),
        );
        const kind = equality.sides.find((side) => side.type === "Literal");
        const guard = kind?.type === "Literal" ? TYPE_GUARDS[String(kind.value)] : undefined;
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
            expression.type === "CallExpression" &&
            isMember(expression.callee, "round") &&
            PERCENT_AFTER.test(quasis[i + 1].value.raw),
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
