import { INTERACTIVE_SELECTOR } from "../shared/interactive";

function installPicker(prefix: string, interactive: string): void {
  if (Reflect.get(window, "__demoflowPicker")) return;
  Reflect.set(window, "__demoflowPicker", true);

  const OVERLAY = "position:fixed;pointer-events:none;z-index:2147483647";
  const ACCENT = "#a78bfa";
  const ACCENT_FILL = "rgba(167,139,250,.15)";
  const TAG_BACKGROUND = "#18181b";
  const TAG_TEXT = "#fafafa";
  const SPOT_SIZE = 12;
  const TAG_GAP = 26;
  const SPOT_TAG_OFFSET = 10;
  const SPOT_TAG_RISE = 30;
  const MAX_NAME_LENGTH = 60;
  const MAX_PATH_DEPTH = 6;

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

  const clean = (s: string | null) => (s || "").replace(/\s+/g, " ").trim();

  const labelFor = (el: Element) => {
    if (!el.id) return null;
    const label = document.querySelector(`label[for="${CSS.escape(el.id)}"]`);
    return label ? clean(label.textContent) : null;
  };

  const nameOf = (el: Element) => {
    const aria = el.getAttribute("aria-label");
    if (aria) return clean(aria);
    const label = labelFor(el);
    if (label !== null) return label;
    const placeholder = el.getAttribute("placeholder");
    if (placeholder) return clean(placeholder);
    const text = clean((el instanceof HTMLElement && el.innerText) || el.textContent);
    if (text && text.length <= MAX_NAME_LENGTH) return text;
    const testId = el.getAttribute("data-testid");
    if (testId) return testId;
    return clean(el.getAttribute("title"));
  };

  const pathStep = (el: Element) => {
    const step = el.tagName.toLowerCase();
    const parent = el.parentElement;
    if (!parent) return step;
    const same = [...parent.children].filter((c) => c.tagName === el.tagName);
    return same.length > 1 ? `${step}:nth-of-type(${same.indexOf(el) + 1})` : step;
  };

  const cssPath = (el: Element) => {
    const parts: string[] = [];
    let cur: Element | null = el;
    while (cur && parts.length < MAX_PATH_DEPTH) {
      if (cur.id) {
        parts.unshift(`#${CSS.escape(cur.id)}`);
        break;
      }
      parts.unshift(pathStep(cur));
      cur = cur.parentElement;
    }
    return parts.join(" > ");
  };

  const cssOf = (el: Element) => {
    if (el.id && !/\d{3,}/.test(el.id)) return `#${CSS.escape(el.id)}`;
    const testId = el.getAttribute("data-testid");
    if (testId) return `[data-testid="${testId}"]`;
    return cssPath(el);
  };

  const inputRole = (el: Element) => {
    const type = (el.getAttribute("type") || "text").toLowerCase();
    if (type === "checkbox" || type === "radio") return type;
    if (type === "search") return "searchbox";
    if (["button", "submit", "reset"].includes(type)) return "button";
    if (type === "range") return "slider";
    return "textbox";
  };

  const roleOf = (el: Element) => {
    const explicit = el.getAttribute("role");
    if (explicit) return explicit;
    const tagName = el.tagName.toLowerCase();
    if (tagName === "a" && el.hasAttribute("href")) return "link";
    if (tagName === "button") return "button";
    if (tagName === "select") return "combobox";
    if (tagName === "textarea") return "textbox";
    if (tagName === "img") return "img";
    if (/^h[1-6]$/.test(tagName)) return "heading";
    if (tagName === "input") return inputRole(el);
    return "";
  };

  const pick = (target: EventTarget | null) =>
    target instanceof Element ? target.closest(interactive) || target : null;

  const report = (result: { target: string; css: string; role: string; selectorUsed: string }) =>
    console.log(prefix + JSON.stringify(result));

  const place = (
    rect: { left: number; top: number; width: number; height: number },
    label: string,
    labelAt: { left: number; top: number },
  ) => {
    Object.assign(box.style, {
      left: `${rect.left}px`,
      top: `${rect.top}px`,
      width: `${rect.width}px`,
      height: `${rect.height}px`,
    });
    tag.textContent = label;
    Object.assign(tag.style, { left: `${labelAt.left}px`, top: `${labelAt.top}px` });
  };

  const showSpot = (e: MouseEvent) => {
    const x = Math.round(e.clientX);
    const y = Math.round(e.clientY);
    const half = SPOT_SIZE / 2;
    place({ left: x - half, top: y - half, width: SPOT_SIZE, height: SPOT_SIZE }, `${x}, ${y}`, {
      left: x + SPOT_TAG_OFFSET,
      top: Math.max(0, y - SPOT_TAG_RISE),
    });
  };

  const showElement = (e: MouseEvent) => {
    const el = pick(e.target);
    if (!el) return;
    const r = el.getBoundingClientRect();
    place(r, nameOf(el) || cssOf(el), { left: r.left, top: Math.max(0, r.top - TAG_GAP) });
  };

  let last: MouseEvent | null = null;
  const show = (e: MouseEvent | null, shift: boolean) => {
    if (!e) return;
    if (shift) showSpot(e);
    else showElement(e);
  };

  const reportClick = (e: MouseEvent) => {
    if (e.shiftKey) {
      const spot = `${Math.round(e.clientX)}, ${Math.round(e.clientY)}`;
      report({ target: spot, css: "", role: "", selectorUsed: "spot" });
      return;
    }
    const el = pick(e.target);
    if (!el) return;
    const name = nameOf(el);
    const css = cssOf(el);
    report({
      target: name || `css=${css}`,
      css,
      role: name ? roleOf(el) : "",
      selectorUsed: name ? "accessible name" : "css",
    });
  };

  document.addEventListener(
    "mousemove",
    (e) => {
      last = e;
      show(e, e.shiftKey);
    },
    true,
  );
  document.addEventListener(
    "keydown",
    (e) => {
      if (e.key === "Shift") show(last, true);
    },
    true,
  );
  document.addEventListener(
    "keyup",
    (e) => {
      if (e.key === "Shift") show(last, false);
    },
    true,
  );
  document.addEventListener(
    "click",
    (e) => {
      e.preventDefault();
      e.stopPropagation();
      reportClick(e);
    },
    true,
  );
}

export const pickerScript = (prefix: string): string =>
  `(${installPicker.toString()})(${JSON.stringify(prefix)}, ${JSON.stringify(`${INTERACTIVE_SELECTOR},label`)})`;
