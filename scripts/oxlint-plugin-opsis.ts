import type { Context, ESTree, Fix, Fixer, Rule } from "@oxlint/plugins";

type Node = ESTree.Node;
type MaybeNode = Node | null | undefined;
type StaticMember = ESTree.StaticMemberExpression;

function commonModule(filename: string, module: string): string | null {
  const file = filename.replaceAll("\\", "/");
  if (file.includes("/packages/video/src/")) return `#video/common/${module}.ts`;
  if (file.includes("/apps/frontend/src/") || file.includes("/scripts/"))
    return `@opsis/video/common/${module}`;
  return null;
}

function importing(context: Context, fixer: Fixer, name: string, module: string): Fix[] | null {
  const source = context.sourceCode;
  if (new RegExp(`\\b(function|const|let|class) ${name}\\b`).test(source.text)) return null;
  const imports = source.ast.body.filter(
    (n): n is ESTree.ImportDeclaration => n.type === "ImportDeclaration",
  );
  const existing = imports.find((n) => n.source.value === module && n.importKind !== "type");
  if (existing) {
    const named = existing.specifiers.filter(
      (sp): sp is ESTree.ImportSpecifier => sp.type === "ImportSpecifier",
    );
    if (named.some((sp) => sp.local.name === name)) return [];
    const last = named.at(-1);
    return last ? [fixer.insertTextAfter(last, `, ${name}`)] : null;
  }
  const line = `import { ${name} } from "${module}";`;
  const after = imports.at(-1);
  return [
    after
      ? fixer.insertTextAfter(after, `\n${line}`)
      : fixer.insertTextBeforeRange([0, 0], `${line}\n`),
  ];
}

function fixWith(
  context: Context,
  name: string,
  module: string,
  edit: (fixer: Fixer) => Fix[],
): ((fixer: Fixer) => Fix[]) | undefined {
  const from = commonModule(context.filename, module);
  if (!from) return undefined;
  return (fixer) => {
    const imported = importing(context, fixer, name, from);
    return imported ? [...edit(fixer), ...imported] : [];
  };
}

const ANIMATE_CLASS =
  /(^|\s)(motion-safe:|motion-reduce:|hover:|group-hover:)?animate-[a-z0-9[\]-]+/;
const MESSAGE =
  "Tailwind `animate-*` classes animate on the wall clock and do not render in the film. Derive the style from the runtime's useFrame() instead.";

const noAnimateClass: Rule = {
  meta: {
    type: "problem",
    docs: { description: "Disallow Tailwind animate-* classes inside the film" },
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

function isIdentifier(node: MaybeNode, name?: string): boolean {
  return node?.type === "Identifier" && (name === undefined || node.name === name);
}

const THREE_CLOCKS = new Set(["Clock", "Timer"]);
const THREE_CLOCK_MESSAGE =
  "Three.js clocks and animation loops run on the wall clock and do not render in the film. Derive the scene from the runtime's useFrame() instead.";

function clockName(node: MaybeNode): string | null {
  if (node?.type === "Identifier") return node.name;
  if (node?.type === "MemberExpression" && !node.computed && node.property.type === "Identifier")
    return node.property.name;
  return null;
}

const noThreeClock: Rule = {
  meta: {
    type: "problem",
    docs: {
      description: "Disallow Three.js clocks and animation loops inside the film",
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

function isMember(node: MaybeNode, property: string): node is StaticMember {
  return (
    node?.type === "MemberExpression" && !node.computed && isIdentifier(node.property, property)
  );
}

function callRule(
  description: string,
  message: string,
  matches: (call: ESTree.CallExpression) => boolean,
  fix?: (call: ESTree.CallExpression, context: Context) => ((fixer: Fixer) => Fix[]) | undefined,
): Rule {
  return {
    meta: { type: "suggestion", fixable: fix ? "code" : undefined, docs: { description } },
    create(context) {
      return {
        CallExpression(node) {
          if (matches(node)) context.report({ node, message, fix: fix?.(node, context) });
        },
      };
    },
  };
}

function strictEqualityBody(fn: MaybeNode): { name: string; sides: [Node, Node] } | null {
  if (fn?.type !== "ArrowFunctionExpression" || fn.params.length !== 1) return null;
  const [param] = fn.params;
  const body = fn.body;
  if (param.type !== "Identifier" || body.type !== "BinaryExpression" || body.operator !== "===")
    return null;
  return { name: param.name, sides: [body.left, body.right] };
}

function comparesIdOf(fn: MaybeNode): boolean {
  const equality = strictEqualityBody(fn);
  return (
    !!equality &&
    equality.sides.some((side) => isMember(side, "id") && isIdentifier(side.object, equality.name))
  );
}

const useFindById = callRule(
  "Find an item by id through findById",
  "Use findById from packages/video/src/common/lists.ts.",
  (call) => isMember(call.callee, "find") && comparesIdOf(call.arguments[0]),
  (call, context) => {
    const equality = strictEqualityBody(call.arguments[0]);
    if (!equality || !isMember(call.callee, "find")) return undefined;
    function text(node: Node) {
      return context.sourceCode.getText(node);
    }
    const id = equality.sides.find(
      (side) => !(isMember(side, "id") && isIdentifier(side.object, equality.name)),
    );
    if (!id || new RegExp(`\\b${equality.name}\\b`).test(text(id))) return undefined;
    const list = text(call.callee.object);
    return fixWith(context, "findById", "lists", (fixer) => [
      fixer.replaceText(call, `findById(${list}, ${text(id)})`),
    ]);
  },
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
  meta: {
    type: "suggestion",
    fixable: "code",
    docs: { description: "Use the shared primitive type guards" },
  },
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
            fix: fixWith(context, guard, "guards", (fixer) => [fixer.replaceText(node, guard)]),
          });
      },
    };
  },
};

const PERCENT_AFTER = /^\s?%/;

function hundredths(node: Node): Node | null {
  if (node.type !== "BinaryExpression" || node.operator !== "*") return null;
  function isHundred(side: Node) {
    return side.type === "Literal" && side.value === 100;
  }
  if (isHundred(node.right)) return node.left;
  if (isHundred(node.left)) return node.right;
  return null;
}

const usePercentText: Rule = {
  meta: {
    type: "suggestion",
    fixable: "code",
    docs: { description: "Write percentages through percentText" },
  },
  create(context) {
    return {
      TemplateLiteral(node) {
        const quasis = node.quasis;
        const rounded = node.expressions.findIndex(
          (expression, i) =>
            expression.type === "CallExpression" &&
            isMember(expression.callee, "round") &&
            PERCENT_AFTER.test(quasis[i + 1].value.raw),
        );
        if (rounded === -1) return;
        const call = node.expressions[rounded];
        const fraction =
          call.type === "CallExpression" && call.arguments.length === 1
            ? hundredths(call.arguments[0])
            : null;
        const text = context.sourceCode.text;
        const sign = text.indexOf("%", call.range[1]);
        const exact = fraction && text.slice(call.range[1], sign) === "}";
        context.report({
          node,
          message: "Use percentText from packages/video/src/common/format.ts.",
          fix:
            exact && fraction
              ? fixWith(context, "percentText", "format", (fixer) => [
                  fixer.replaceText(call, `percentText(${context.sourceCode.getText(fraction)})`),
                  fixer.removeRange([sign, sign + 1]),
                ])
              : undefined,
        });
      },
    };
  },
};

const FC_NAMES = new Set(["FC", "FunctionComponent"]);

function isComponentType(node: MaybeNode): boolean {
  if (node?.type !== "TSTypeReference") return false;
  const name = node.typeName;
  if (name.type === "Identifier") return FC_NAMES.has(name.name);
  return name.type === "TSQualifiedName" && FC_NAMES.has(name.right.name);
}

function componentFix(
  context: Context,
  declarator: ESTree.VariableDeclarator,
): ((fixer: Fixer) => Fix) | undefined {
  const declaration = declarator.parent;
  const init = declarator.init;
  if (
    declaration?.type !== "VariableDeclaration" ||
    declaration.declarations.length !== 1 ||
    declarator.id.type !== "Identifier" ||
    (init?.type !== "ArrowFunctionExpression" && init?.type !== "FunctionExpression") ||
    !init.body ||
    init.params.length > 1 ||
    (init.type === "ArrowFunctionExpression" && usesOwnBinding(init.body))
  )
    return undefined;
  function text(node: Node) {
    return context.sourceCode.getText(node);
  }
  const annotation = declarator.id.typeAnnotation?.typeAnnotation;
  const props =
    annotation?.type === "TSTypeReference" ? annotation.typeArguments?.params[0] : undefined;
  const [param] = init.params;
  const typed =
    param && "typeAnnotation" in param && param.typeAnnotation
      ? text(param)
      : param
        ? `${text(param)}${props ? `: ${text(props)}` : ""}`
        : "";
  const body =
    init.body.type === "BlockStatement" ? text(init.body) : `{\n  return ${text(init.body)};\n}`;
  const name = declarator.id.name;
  return (fixer) => fixer.replaceText(declaration, `function ${name}(${typed}) ${body}`);
}

const useFunctionComponents: Rule = {
  meta: {
    type: "suggestion",
    fixable: "code",
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
            fix: componentFix(context, node),
          });
      },
    };
  },
};

const FUNCTION_BOUNDARIES = new Set([
  "FunctionDeclaration",
  "FunctionExpression",
  "ClassDeclaration",
  "ClassExpression",
]);

function usesOwnBinding(node: unknown): boolean {
  if (!node || typeof node !== "object") return false;
  if (Array.isArray(node)) return node.some(usesOwnBinding);
  const n = node as { type?: string; name?: string };
  if (n.type === "ThisExpression" || (n.type === "Identifier" && n.name === "arguments"))
    return true;
  if (n.type && FUNCTION_BOUNDARIES.has(n.type)) return false;
  return Object.entries(node).some(
    ([key, child]) => key !== "parent" && typeof child === "object" && usesOwnBinding(child),
  );
}

const useFunctionDeclarations: Rule = {
  meta: {
    type: "suggestion",
    fixable: "code",
    docs: { description: "Write functions as declarations, not functions assigned to a const" },
  },
  create(context) {
    function text(node: Node) {
      return context.sourceCode.getText(node);
    }
    return {
      VariableDeclaration(node) {
        const [declarator] = node.declarations;
        const init = declarator?.init;
        if (
          node.kind !== "const" ||
          node.declare ||
          node.declarations.length !== 1 ||
          declarator.id.type !== "Identifier" ||
          (init?.type !== "ArrowFunctionExpression" && init?.type !== "FunctionExpression") ||
          !init.body
        )
          return;
        const body = init.body;
        const message = `Write \`${declarator.id.name}\` as a function declaration (AGENTS.md).`;
        const fixable =
          !declarator.id.typeAnnotation &&
          !(init.type === "FunctionExpression" && init.id) &&
          !(init.type === "ArrowFunctionExpression" && usesOwnBinding(body));
        if (!fixable) {
          context.report({ node, message });
          return;
        }
        const typeParameters = init.typeParameters
          ? text(init.typeParameters).replace(/,\s*>$/, ">")
          : "";
        const params = init.params.map(text).join(", ");
        const returned = init.returnType ? text(init.returnType) : "";
        const returnType = returned && !returned.startsWith(":") ? `: ${returned}` : returned;
        const block = body.type === "BlockStatement" ? text(body) : `{\n  return ${text(body)};\n}`;
        const declaration = `${init.async ? "async " : ""}function ${declarator.id.name}${typeParameters}(${params})${returnType} ${block}`;
        context.report({ node, message, fix: (fixer) => fixer.replaceText(node, declaration) });
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
    "use-function-declarations": useFunctionDeclarations,
    "use-parse-json-with": useParseJsonWith,
    "use-percent-text": usePercentText,
    "use-root": useRoot,
    "use-type-guards": useTypeGuards,
  },
};
