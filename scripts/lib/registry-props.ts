import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import {
  Node,
  SyntaxKind,
  type ArrowFunction,
  type FunctionDeclaration,
  type FunctionExpression,
  type SourceFile,
  type TypeNode,
} from "ts-morph";
import { HIDDEN_PROPS, type CuratedElement } from "../registry-ingest.list";
import { namesLibrary } from "./registry-controls";
import { scratchFile } from "./ts-literal";

export type ComponentProp = {
  name: string;
  required: boolean;
  defaultText?: string;
};

export type ComponentProps = { props: ComponentProp[] };

type ComponentFunction = ArrowFunction | FunctionExpression | FunctionDeclaration;

function componentFunction(sf: SourceFile, exportName: string): ComponentFunction | undefined {
  const declared = sf.getFunction(exportName);
  if (declared) return declared;
  const init = sf.getVariableDeclaration(exportName)?.getInitializer();
  if (!init) return undefined;
  if (Node.isIdentifier(init) && init.getText() !== exportName)
    return componentFunction(sf, init.getText());
  if (Node.isArrowFunction(init) || Node.isFunctionExpression(init)) return init;
  if (Node.isCallExpression(init)) {
    const inner = init.getArguments()[0];
    if (inner && (Node.isArrowFunction(inner) || Node.isFunctionExpression(inner))) return inner;
  }
  return undefined;
}

function callTypeArgument(sf: SourceFile, exportName: string): TypeNode | undefined {
  const init = sf.getVariableDeclaration(exportName)?.getInitializer();
  return init && Node.isCallExpression(init) ? init.getTypeArguments()[0] : undefined;
}

function declaredPropsType(sf: SourceFile, exportName: string, fn: ComponentFunction) {
  const param = fn.getParameters()[0]?.getTypeNode();
  if (param) return param;
  const declared = sf.getVariableDeclaration(exportName)?.getTypeNode();
  if (declared && Node.isTypeReference(declared)) return declared.getTypeArguments()[0];
  return undefined;
}

type Members = Map<string, boolean>;

function collectMembers(sf: SourceFile, node: TypeNode | undefined, seen: Set<string>): Members {
  const members: Members = new Map();
  if (!node) return members;
  if (Node.isTypeLiteral(node)) {
    for (const m of node.getProperties()) members.set(m.getName(), !m.hasQuestionToken());
    return members;
  }
  if (Node.isIntersectionTypeNode(node) || Node.isParenthesizedTypeNode(node)) {
    const parts = Node.isParenthesizedTypeNode(node) ? [node.getTypeNode()] : node.getTypeNodes();
    for (const part of parts)
      for (const [k, v] of collectMembers(sf, part, seen)) members.set(k, v);
    return members;
  }
  if (Node.isTypeReference(node)) return collectNamed(sf, node.getTypeName().getText(), seen);
  return members;
}

const origins = new WeakMap<SourceFile, string>();

function importedFrom(sf: SourceFile, name: string): SourceFile | undefined {
  const origin = origins.get(sf);
  if (!origin) return undefined;
  const decl = sf
    .getImportDeclarations()
    .find(
      (d) =>
        d.getModuleSpecifierValue().startsWith(".") &&
        d.getNamedImports().some((n) => n.getName() === name),
    );
  if (!decl) return undefined;
  const base = path.resolve(path.dirname(origin), decl.getModuleSpecifierValue());
  const file = [`${base}.ts`, `${base}.tsx`, path.join(base, "index.ts")].find((f) =>
    existsSync(f),
  );
  if (!file) return undefined;
  const imported = scratchFile("imported", "tsx", readFileSync(file, "utf8"));
  origins.set(imported, file);
  return imported;
}

function collectNamed(sf: SourceFile, name: string, seen: Set<string>): Members {
  const members: Members = new Map();
  if (seen.has(name)) return members;
  seen.add(name);
  const alias = sf.getTypeAlias(name);
  if (alias) return collectMembers(sf, alias.getTypeNode(), seen);
  const iface = sf.getInterface(name);
  if (!iface) {
    seen.delete(name);
    const imported = importedFrom(sf, name);
    return imported ? collectNamed(imported, name, seen) : members;
  }
  for (const clause of iface.getHeritageClauses())
    for (const base of clause.getTypeNodes())
      for (const [k, v] of collectNamed(sf, base.getExpression().getText(), seen))
        members.set(k, v);
  for (const m of iface.getProperties()) members.set(m.getName(), !m.hasQuestionToken());
  return members;
}

export function componentProps(
  source: string,
  exportName: string,
  file?: string,
): ComponentProps | null {
  const sf = scratchFile("props", "tsx", source);
  if (file) origins.set(sf, file);
  const fn = componentFunction(sf, exportName);
  const typeArg = fn ? undefined : callTypeArgument(sf, exportName);
  if (!fn && !typeArg) return null;
  const members = collectMembers(
    sf,
    fn ? declaredPropsType(sf, exportName, fn) : typeArg,
    new Set(),
  );
  const defaults = new Map<string, string | undefined>();
  const binding = fn?.getParameters()[0]?.getNameNode();
  if (binding && Node.isObjectBindingPattern(binding))
    for (const el of binding.getElements()) {
      if (el.getDotDotDotToken()) continue;
      defaults.set(
        el.getPropertyNameNode()?.getText() ?? el.getName(),
        el.getInitializer()?.getText(),
      );
    }
  if (members.size === 0 && defaults.size === 0) return null;
  const names = new Set([...members.keys(), ...defaults.keys()]);
  return {
    props: [...names].map((name) => ({
      name,
      required: (members.get(name) ?? false) && defaults.get(name) === undefined,
      defaultText: defaults.get(name),
    })),
  };
}

export type PropCoverage = {
  key: string;
  item: CuratedElement;
  file: string;
  exportName: string;
  shown: ReadonlySet<string>;
};

function coveredByWrappers(item: CuratedElement): string[] {
  return [
    ...Object.values(item.nest ?? {}).map((p) => p.split(".")[0]),
    ...Object.values(item.lists ?? {}).flatMap((columns) => Object.values(columns)),
    ...(item.marks ? [item.marks.to] : []),
    ...(item.theme ? ["theme", "mode"] : []),
    ...(item.place === "points" ? ["from", "to"] : []),
    ...(item.atEnd ? ["delay"] : []),
    ...(item.kind === "sequencer" ? ["scenes", "config"] : []),
  ];
}

export function hiddenPropProblems({ key, item, file, exportName, shown }: PropCoverage): string[] {
  const source = readFileSync(file, "utf8");
  const read = componentProps(source, exportName, file);
  if (!read)
    return item.props === "unchecked"
      ? []
      : [`${key}: its props can't be read; mark it props: "unchecked" in the list`];
  const wrapped = new Set(coveredByWrappers(item));
  const hidden = read.props
    .map((p) => p.name)
    .filter((name) => !shown.has(name) && !wrapped.has(name));
  const named = [...hiddenDefaultTexts(source, hidden)].flatMap(([prop, texts]) =>
    texts
      .filter(namesLibrary)
      .map(
        (t) =>
          `${key}: the hidden default for "${prop}" names a library, company or product: ${JSON.stringify(t).slice(0, 80)}`,
      ),
  );
  const covered = new Set([
    ...shown,
    ...Object.keys(item.omit ?? {}),
    ...Object.keys(HIDDEN_PROPS),
    ...coveredByWrappers(item),
  ]);
  return [
    ...named,
    ...read.props
      .filter((p) => !covered.has(p.name))
      .map((p) =>
        p.required
          ? `${key}: required prop "${p.name}" has no setting, fixed value or default`
          : `${key}: prop "${p.name}" (default ${p.defaultText?.slice(0, 40) ?? "none"}) is not a setting, a fixed prop or omitted with a reason`,
      ),
  ];
}

const LABEL_ATTRIBUTES = new Set(["aria-label", "title", "alt"]);

function textsIn(node: Node): string[] {
  const out: string[] = [];
  for (const n of [node, ...node.getDescendants()]) {
    if (Node.isStringLiteral(n) || Node.isNoSubstitutionTemplateLiteral(n)) {
      const parent = n.getParent();
      const attribute =
        parent && Node.isJsxAttribute(parent) ? parent.getNameNode().getText() : null;
      if (attribute === null || LABEL_ATTRIBUTES.has(attribute)) out.push(n.getLiteralValue());
    }
    if (Node.isJsxText(n) && n.getText().trim()) out.push(n.getText().trim());
  }
  return out;
}

function referencedInitializers(sf: SourceFile, node: Node, depth = 0): Node[] {
  if (depth > 2) return [];
  const names = new Set(
    [node, ...node.getDescendants()]
      .filter(
        (n) =>
          Node.isIdentifier(n) || Node.isJsxOpeningElement(n) || Node.isJsxSelfClosingElement(n),
      )
      .map((n) => (Node.isIdentifier(n) ? n.getText() : n.getTagNameNode().getText())),
  );
  return [...names].flatMap((name) => {
    const init = sf.getVariableDeclaration(name)?.getInitializer();
    const fn = sf.getFunction(name);
    const found = init ?? fn;
    return found ? [found, ...referencedInitializers(sf, found, depth + 1)] : [];
  });
}

export function hiddenDefaultTexts(
  source: string,
  props: readonly string[],
): Map<string, string[]> {
  const sf = scratchFile("hidden-defaults", "tsx", source);
  const wanted = new Set(props);
  const out = new Map<string, string[]>();
  for (const el of sf.getDescendantsOfKind(SyntaxKind.BindingElement)) {
    const key = el.getPropertyNameNode()?.getText() ?? el.getName();
    const init = el.getInitializer();
    if (!init || !wanted.has(key) || out.has(key)) continue;
    out.set(key, [init, ...referencedInitializers(sf, init)].flatMap(textsIn));
  }
  return out;
}
