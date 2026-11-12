export type Rect = { x: number; y: number; width: number; height: number };

export type Box = {
  path: number[];
  kind: "element" | "text";
  tag: string;
  rect: Rect;
  lines: Rect[];
};

declare global {
  interface Window {
    oracleBoxes?: () => Box[];
  }
}

function rectOf(rect: DOMRect): Rect {
  return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
}

function textLines(node: Text): Rect[] {
  const range = document.createRange();
  range.selectNodeContents(node);
  return [...range.getClientRects()].map(rectOf);
}

function visit(node: Node, path: number[], out: Box[]) {
  if (node instanceof Text) {
    const range = document.createRange();
    range.selectNodeContents(node);
    out.push({
      path,
      kind: "text",
      tag: "#text",
      rect: rectOf(range.getBoundingClientRect()),
      lines: textLines(node),
    });
    return;
  }
  if (!(node instanceof Element)) return;
  out.push({
    path,
    kind: "element",
    tag: node.tagName.toLowerCase(),
    rect: rectOf(node.getBoundingClientRect()),
    lines: [],
  });
  [...node.childNodes].forEach((child, i) => visit(child, [...path, i], out));
}

export function boxesOf(root: Element): Box[] {
  const out: Box[] = [];
  [...root.childNodes].forEach((child, i) => visit(child, [i], out));
  return out;
}
