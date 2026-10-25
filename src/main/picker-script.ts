import { INTERACTIVE_SELECTOR, type PickedRole } from "@opsis/core/interactive";
import type { PickResult } from "@opsis/core/ipc";

type Box = { left: number; top: number; width: number; height: number };
type Point = { left: number; top: number };

function clean(s: string | null): string {
  return (s || "").replace(/\s+/g, " ").trim();
}

function labelFor(el: Element): string | null {
  if (!el.id) return null;
  const label = document.querySelector(`label[for="${CSS.escape(el.id)}"]`);
  return label ? clean(label.textContent) : null;
}

function nameOf(el: Element, maxLength: number): string {
  const aria = el.getAttribute("aria-label");
  if (aria) return clean(aria);
  const label = labelFor(el);
  if (label !== null) return label;
  const placeholder = el.getAttribute("placeholder");
  if (placeholder) return clean(placeholder);
  const text = clean((el instanceof HTMLElement && el.innerText) || el.textContent);
  if (text && text.length <= maxLength) return text;
  const testId = el.getAttribute("data-testid");
  if (testId) return testId;
  return clean(el.getAttribute("title"));
}

function pathStep(el: Element): string {
  const step = el.tagName.toLowerCase();
  const parent = el.parentElement;
  if (!parent) return step;
  const same = [...parent.children].filter((c) => c.tagName === el.tagName);
  return same.length > 1 ? `${step}:nth-of-type(${same.indexOf(el) + 1})` : step;
}

function cssPath(el: Element, maxDepth: number): string {
  const parts: string[] = [];
  let cur: Element | null = el;
  while (cur && parts.length < maxDepth) {
    if (cur.id) {
      parts.unshift(`#${CSS.escape(cur.id)}`);
      break;
    }
    parts.unshift(pathStep(cur));
    cur = cur.parentElement;
  }
  return parts.join(" > ");
}

function cssOf(el: Element, maxDepth: number): string {
  if (el.id && !/\d{3,}/.test(el.id)) return `#${CSS.escape(el.id)}`;
  const testId = el.getAttribute("data-testid");
  if (testId) return `[data-testid="${CSS.escape(testId)}"]`;
  return cssPath(el, maxDepth);
}

function inputRole(el: Element): PickedRole {
  const type = (el.getAttribute("type") || "text").toLowerCase();
  if (type === "checkbox" || type === "radio") return type;
  if (type === "search") return "searchbox";
  if (["button", "submit", "reset"].includes(type)) return "button";
  if (type === "range") return "slider";
  return "textbox";
}

function implicitRole(el: Element): PickedRole | "" {
  const tagName = el.tagName.toLowerCase();
  if (tagName === "a" && el.hasAttribute("href")) return "link";
  if (tagName === "button") return "button";
  if (tagName === "select") return "combobox";
  if (tagName === "textarea") return "textbox";
  if (tagName === "img") return "img";
  if (/^h[1-6]$/.test(tagName)) return "heading";
  if (tagName === "input") return inputRole(el);
  return "";
}

function roleOf(el: Element): string {
  return el.getAttribute("role") || implicitRole(el);
}

function createHighlight(): (rect: Box, label: string, labelAt: Point) => void {
  const OVERLAY = "position:fixed;pointer-events:none;z-index:2147483647";
  const ACCENT = "#a78bfa";
  const ACCENT_FILL = "rgba(167,139,250,.15)";
  const TAG_BACKGROUND = "#18181b";
  const TAG_TEXT = "#fafafa";
  const overlay = (css: string) => {
    const el = document.createElement("div");
    el.style.cssText = `${OVERLAY};${css}`;
    document.documentElement.appendChild(el);
    return el;
  };
  const box = overlay(`border:2px solid ${ACCENT};background:${ACCENT_FILL};border-radius:4px`);
  const tag = overlay(
    `background:${TAG_BACKGROUND};color:${TAG_TEXT};font:600 12px system-ui;padding:4px 8px;border-radius:6px`,
  );
  return (rect, label, labelAt) => {
    Object.assign(box.style, {
      left: `${rect.left}px`,
      top: `${rect.top}px`,
      width: `${rect.width}px`,
      height: `${rect.height}px`,
    });
    tag.textContent = label;
    Object.assign(tag.style, { left: `${labelAt.left}px`, top: `${labelAt.top}px` });
  };
}

function spotOf(e: MouseEvent): { x: number; y: number; text: string } {
  const x = Math.round(e.clientX);
  const y = Math.round(e.clientY);
  return { x, y, text: `${x}, ${y}` };
}

function installPicker(prefix: string, interactive: string): void {
  if (Reflect.get(window, "__opsisPicker")) return;
  Reflect.set(window, "__opsisPicker", true);

  const SPOT_SIZE = 12;
  const TAG_GAP = 26;
  const SPOT_TAG_OFFSET = 10;
  const SPOT_TAG_RISE = 30;
  const MAX_NAME_LENGTH = 60;
  const MAX_PATH_DEPTH = 6;

  const place = createHighlight();
  const pick = (target: EventTarget | null) =>
    target instanceof Element ? target.closest(interactive) || target : null;
  const report = (result: NonNullable<PickResult>) => console.log(prefix + JSON.stringify(result));

  const showSpot = (e: MouseEvent) => {
    const { x, y, text } = spotOf(e);
    const half = SPOT_SIZE / 2;
    place({ left: x - half, top: y - half, width: SPOT_SIZE, height: SPOT_SIZE }, text, {
      left: x + SPOT_TAG_OFFSET,
      top: Math.max(0, y - SPOT_TAG_RISE),
    });
  };

  const showElement = (e: MouseEvent) => {
    const el = pick(e.target);
    if (!el) return;
    const r = el.getBoundingClientRect();
    const label = nameOf(el, MAX_NAME_LENGTH) || cssOf(el, MAX_PATH_DEPTH);
    place(r, label, { left: r.left, top: Math.max(0, r.top - TAG_GAP) });
  };

  let last: MouseEvent | null = null;
  const show = (e: MouseEvent | null, shift: boolean) => {
    if (!e) return;
    if (shift) showSpot(e);
    else showElement(e);
  };

  const reportClick = (e: MouseEvent) => {
    if (e.shiftKey) {
      report({ target: spotOf(e).text, css: "", role: "", selectorUsed: "spot" });
      return;
    }
    const el = pick(e.target);
    if (!el) return;
    const name = nameOf(el, MAX_NAME_LENGTH);
    const css = cssOf(el, MAX_PATH_DEPTH);
    report({
      target: name || `css=${css}`,
      css,
      role: name ? roleOf(el) : "",
      selectorUsed: name ? "accessible name" : "css",
    });
  };

  const capture = <K extends keyof DocumentEventMap>(
    type: K,
    listener: (e: DocumentEventMap[K]) => void,
  ) => document.addEventListener(type, listener, true);

  capture("mousemove", (e) => {
    last = e;
    show(e, e.shiftKey);
  });
  capture("keydown", (e) => {
    if (e.key === "Shift") show(last, true);
  });
  capture("keyup", (e) => {
    if (e.key === "Shift") show(last, false);
  });
  capture("click", (e) => {
    e.preventDefault();
    e.stopPropagation();
    reportClick(e);
  });
}

const PAGE_FUNCTIONS = [
  clean,
  labelFor,
  nameOf,
  pathStep,
  cssPath,
  cssOf,
  inputRole,
  implicitRole,
  roleOf,
  createHighlight,
  spotOf,
  installPicker,
];

export const pickerScript = (prefix: string): string =>
  [
    "(() => {",
    ...PAGE_FUNCTIONS.map(String),
    `${installPicker.name}(${JSON.stringify(prefix)}, ${JSON.stringify(`${INTERACTIVE_SELECTOR},label`)});`,
    "})()",
  ].join("\n");
