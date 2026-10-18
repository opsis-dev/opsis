import { Node, Project, SyntaxKind, type SourceFile } from "ts-morph";
import { isRecord } from "../../src/shared/guards";

const project = new Project({ useInMemoryFileSystem: true });
let scratchFiles = 0;

export const scratchFile = (name: string, ext: string, source: string): SourceFile =>
  project.createSourceFile(`${name}-${++scratchFiles}.${ext}`, source);

const MAX_DEPTH = 8;

export function literal(node: Node | undefined, scope?: SourceFile, depth = 0): unknown {
  if (!node || depth > MAX_DEPTH) return undefined;
  const next = (n: Node | undefined) => literal(n, scope, depth + 1);
  if (Node.isStringLiteral(node) || Node.isNoSubstitutionTemplateLiteral(node))
    return node.getLiteralValue();
  if (Node.isNumericLiteral(node)) return node.getLiteralValue();
  if (Node.isPrefixUnaryExpression(node) && node.getOperatorToken() === SyntaxKind.MinusToken) {
    const v = next(node.getOperand());
    return typeof v === "number" ? -v : undefined;
  }
  if (node.getKind() === SyntaxKind.TrueKeyword) return true;
  if (node.getKind() === SyntaxKind.FalseKeyword) return false;
  if (
    Node.isAsExpression(node) ||
    Node.isSatisfiesExpression(node) ||
    Node.isParenthesizedExpression(node)
  )
    return next(node.getExpression());
  if (Node.isArrayLiteralExpression(node)) return node.getElements().map((e) => next(e));
  if (Node.isObjectLiteralExpression(node)) {
    const out: Record<string, unknown> = {};
    for (const prop of node.getProperties()) {
      if (Node.isPropertyAssignment(prop)) {
        const name = prop
          .getNameNode()
          .getText()
          .replace(/^["']|["']$/g, "");
        out[name] = next(prop.getInitializer());
      }
    }
    return out;
  }
  if (Node.isCallExpression(node) && node.getExpression().getText() === "enumVariants") {
    const arr = next(node.getArguments()[0]);
    if (Array.isArray(arr)) return Object.fromEntries(arr.map((v) => [String(v), {}]));
  }
  if (scope && Node.isIdentifier(node)) {
    const decl = scope.getVariableDeclaration(node.getText());
    return decl ? next(decl.getInitializer()) : undefined;
  }
  if (scope && Node.isPropertyAccessExpression(node)) {
    const owner = next(node.getExpression());
    if (isRecord(owner)) return owner[node.getName()];
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
