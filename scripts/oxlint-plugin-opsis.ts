import type { ESTree, Rule } from "@oxlint/plugins";

type Node = ESTree.Node;
type MaybeNode = Node | null | undefined;
type StaticMember = ESTree.StaticMemberExpression;

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

const isIdentifier = (node: MaybeNode, name?: string): boolean =>
  node?.type === "Identifier" && (name === undefined || node.name === name);

const THREE_CLOCKS = new Set(["Clock", "Timer"]);
const THREE_CLOCK_MESSAGE =
  "Three.js clocks and animation loops run on the wall clock and do not render in Remotion. Derive the scene from useCurrentFrame() instead.";

const clockName = (node: MaybeNode): string | null => {
  if (node?.type === "Identifier") return node.name;
  if (node?.type === "MemberExpression" && !node.computed && node.property.type === "Identifier")
    return node.property.name;
  return null;
};

const noThreeClock: Rule = {
  meta: {
    type: "problem",
    docs: {
      description: "Disallow Three.js clocks and animation loops inside Remotion compositions",
    },
  },
  create(context) {
    return {
      NewExpression(node) {
        const name = clockName(node.callee);
        if (name && THREE_CLOCKS.has(name)) context.report({ node, message: THREE_CLOCK_MESSAGE });
      },
      CallExpression(node) {
        if (clockName(node.callee) === "setAnimationLoop")
          context.report({ node, message: THREE_CLOCK_MESSAGE });
      },
    };
  },
};

const isMember = (node: MaybeNode, property: string): node is StaticMember =>
  node?.type === "MemberExpression" && !node.computed && isIdentifier(node.property, property);

const callRule = (
  description: string,
  message: string,
  matches: (call: ESTree.CallExpression) => boolean,
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
  "Use findById from packages/video/src/common/lists.ts.",
  (call) => isMember(call.callee, "find") && comparesIdOf(call.arguments[0]),
);

const useParseJsonWith = callRule(
  "Parse and check JSON through parseJsonWith",
  "Use parseJsonWith from packages/video/src/common/guards.ts.",
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
          context.report({
            node,
            message: `Use ${guard} from packages/video/src/common/guards.ts.`,
          });
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
          context.report({
            node,
            message: "Use percentText from packages/video/src/common/format.ts.",
          });
      },
    };
  },
};

const FC_NAMES = new Set(["FC", "FunctionComponent"]);

const isComponentType = (node: MaybeNode): boolean => {
  if (node?.type !== "TSTypeReference") return false;
  const name = node.typeName;
  if (name.type === "Identifier") return FC_NAMES.has(name.name);
  return name.type === "TSQualifiedName" && FC_NAMES.has(name.right.name);
};

const useFunctionComponents: Rule = {
  meta: {
    type: "suggestion",
    docs: { description: "Write components as named functions, not React.FC constants" },
  },
  create(context) {
    return {
      VariableDeclarator(node) {
        const init = node.init;
        if (
          node.id.type === "Identifier" &&
          isComponentType(node.id.typeAnnotation?.typeAnnotation) &&
          (init?.type === "ArrowFunctionExpression" || init?.type === "FunctionExpression")
        )
          context.report({
            node,
            message:
              "Write the component as a named function with its props type on the parameter.",
          });
      },
    };
  },
};

export default {
  meta: { name: "opsis" },
  rules: {
    "no-animate-class": noAnimateClass,
    "no-three-clock": noThreeClock,
    "use-find-by-id": useFindById,
    "use-function-components": useFunctionComponents,
    "use-parse-json-with": useParseJsonWith,
    "use-percent-text": usePercentText,
    "use-root": useRoot,
    "use-type-guards": useTypeGuards,
  },
};
