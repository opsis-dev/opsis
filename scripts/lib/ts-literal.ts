import { isRecord } from "@opsis/video/common/guards";
import { Node, Project, SyntaxKind, type SourceFile } from "ts-morph";

const project = new Project({ useInMemoryFileSystem: true });
let scratchFiles = 0;

export const scratchFile = (name: string, ext: string, source: string): SourceFile =>
  project.createSourceFile(`${name}-${++scratchFiles}.${ext}`, source);

const MAX_DEPTH = 8;

const NOT_THIS_KIND = Symbol("not this kind");

type Resolve = (node: Node | undefined) => unknown;

type Resolver = (node: Node, next: Resolve, scope?: SourceFile) => unknown;

const scalarValue: Resolver = (node) => {
  if (Node.isStringLiteral(node) || Node.isNoSubstitutionTemplateLiteral(node))
    return node.getLiteralValue();
  if (Node.isNumericLiteral(node)) return node.getLiteralValue();
  if (Node.isTrueLiteral(node)) return true;
  if (Node.isFalseLiteral(node)) return false;
  return NOT_THIS_KIND;
};

const negatedValue: Resolver = (node, next) => {
  if (!Node.isPrefixUnaryExpression(node) || node.getOperatorToken() !== SyntaxKind.MinusToken)
    return NOT_THIS_KIND;
  const v = next(node.getOperand());
  return typeof v === "number" ? -v : undefined;
};

const unwrappedValue: Resolver = (node, next) =>
  Node.isAsExpression(node) ||
  Node.isSatisfiesExpression(node) ||
  Node.isParenthesizedExpression(node)
    ? next(node.getExpression())
    : NOT_THIS_KIND;

const arrayValue: Resolver = (node, next) =>
  Node.isArrayLiteralExpression(node) ? node.getElements().map((e) => next(e)) : NOT_THIS_KIND;

const propertyName = (prop: Node) => prop.getText().replace(/^["']|["']$/g, "");

const objectValue: Resolver = (node, next) => {
  if (!Node.isObjectLiteralExpression(node)) return NOT_THIS_KIND;
  const out: Record<string, unknown> = {};
  for (const prop of node.getProperties())
    if (Node.isPropertyAssignment(prop))
      out[propertyName(prop.getNameNode())] = next(prop.getInitializer());
  return out;
};

const enumVariantsValue: Resolver = (node, next) => {
  if (!Node.isCallExpression(node) || node.getExpression().getText() !== "enumVariants")
    return NOT_THIS_KIND;
  const variants = next(node.getArguments()[0]);
  return Array.isArray(variants)
    ? Object.fromEntries(variants.map((v) => [String(v), {}]))
    : undefined;
};

const referencedValue: Resolver = (node, next, scope) => {
  if (!scope) return NOT_THIS_KIND;
  if (Node.isIdentifier(node))
    return next(scope.getVariableDeclaration(node.getText())?.getInitializer());
  if (!Node.isPropertyAccessExpression(node)) return NOT_THIS_KIND;
  const owner = next(node.getExpression());
  return isRecord(owner) ? owner[node.getName()] : undefined;
};

const RESOLVERS = [
  scalarValue,
  negatedValue,
  unwrappedValue,
  arrayValue,
  objectValue,
  enumVariantsValue,
  referencedValue,
];

function literal(node: Node | undefined, scope?: SourceFile, depth = 0): unknown {
  if (!node || depth > MAX_DEPTH) return undefined;
  const next: Resolve = (n) => literal(n, scope, depth + 1);
  for (const resolve of RESOLVERS) {
    const value = resolve(node, next, scope);
    if (value !== NOT_THIS_KIND) return value;
  }
  return undefined;
}

export function literalOf<T>(
  node: Node | undefined,
  guard: (value: unknown) => value is T,
  scope?: SourceFile,
): T | undefined {
  const value = literal(node, scope);
  return guard(value) ? value : undefined;
}

export function sourceDefaults(source: string, exportName: string): Map<string, unknown> {
  const sf = scratchFile("src", "tsx", source);
  const out = new Map<string, unknown>();
  const init = sf.getVariableDeclaration(exportName)?.getInitializer();
  const fn =
    init && (Node.isArrowFunction(init) || Node.isFunctionExpression(init))
      ? init
      : sf.getFunction(exportName);
  const binding = fn?.getParameters()[0]?.getNameNode();
  if (!binding || !Node.isObjectBindingPattern(binding)) return out;
  for (const el of binding.getElements()) {
    if (el.getDotDotDotToken()) continue;
    const key = el.getPropertyNameNode()?.getText() ?? el.getName();
    out.set(key, literal(el.getInitializer(), sf));
  }
  return out;
}

export function bindingDefaults(source: string): Map<string, unknown> {
  const sf = scratchFile("bindings", "tsx", source);
  const out = new Map<string, unknown>();
  for (const el of sf.getDescendantsOfKind(SyntaxKind.BindingElement)) {
    const init = el.getInitializer();
    if (!init || el.getDotDotDotToken()) continue;
    const key = el.getPropertyNameNode()?.getText() ?? el.getName();
    if (!out.has(key)) out.set(key, literal(init, sf));
  }
  return out;
}
