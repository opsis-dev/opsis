import path from "node:path";
import type { Context, ESTree, Fix, Fixer, Rule } from "@oxlint/plugins";

type Node = ESTree.Node;
type MaybeNode = Node | null | undefined;
type StaticMember = ESTree.StaticMemberExpression;

const COMMON_DIR = path.join(import.meta.dirname, "lib", "common");

function commonModule(filename: string, module: string): string | null {
  const file = filename.replaceAll("\\", "/");
  if (!file.includes("/scripts/")) return null;
  const relative = path.relative(path.dirname(filename), path.join(COMMON_DIR, module));
  const posix = relative.split(path.sep).join("/");
  return posix.startsWith(".") ? posix : `./${posix}`;
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

function isIdentifier(node: MaybeNode, name?: string): boolean {
  return node?.type === "Identifier" && (name === undefined || node.name === name);
}

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
  "Use findById from scripts/lib/common/lists.ts.",
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
  "Use parseJsonWith from scripts/lib/common/guards.ts.",
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
            message: `Use ${guard} from scripts/lib/common/guards.ts.`,
            fix: fixWith(context, guard, "guards", (fixer) => [fixer.replaceText(node, guard)]),
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
    "use-find-by-id": useFindById,
    "use-function-declarations": useFunctionDeclarations,
    "use-parse-json-with": useParseJsonWith,
    "use-root": useRoot,
    "use-type-guards": useTypeGuards,
  },
};
