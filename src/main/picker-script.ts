export function installPicker(prefix: string): void {
  if (Reflect.get(window, "__demoflowPicker")) return;
  Reflect.set(window, "__demoflowPicker", true);
  const box = document.createElement("div");
  box.style.cssText =
    "position:fixed;pointer-events:none;z-index:2147483647;border:2px solid #a78bfa;background:rgba(167,139,250,.15);border-radius:4px";
  const tag = document.createElement("div");
  tag.style.cssText =
    "position:fixed;pointer-events:none;z-index:2147483647;background:#18181b;color:#fafafa;font:600 12px system-ui;padding:4px 8px;border-radius:6px";
  document.documentElement.appendChild(box);
  document.documentElement.appendChild(tag);
  const INTERACTIVE =
    "button,a,input,textarea,select,[role=button],[role=link],[role=menuitem],[role=tab],[role=option],[role=checkbox],[role=switch],[role=combobox],label";
  const clean = (s: string | null) => (s || "").replace(/\s+/g, " ").trim();
  const nameOf = (el: Element) => {
    const aria = el.getAttribute("aria-label");
    if (aria) return clean(aria);
    if (el.id) {
      const label = document.querySelector(`label[for="${CSS.escape(el.id)}"]`);
      if (label) return clean(label.textContent);
    }
    const placeholder = el.getAttribute("placeholder");
    if (placeholder) return clean(placeholder);
    const text = clean((el instanceof HTMLElement && el.innerText) || el.textContent);
    if (text && text.length <= 60) return text;
    const testId = el.getAttribute("data-testid");
    if (testId) return testId;
    const title = el.getAttribute("title");
    if (title) return clean(title);
    return "";
  };
  const cssOf = (el: Element) => {
    if (el.id && !/\d{3,}/.test(el.id)) return `#${CSS.escape(el.id)}`;
    const testId = el.getAttribute("data-testid");
    if (testId) return `[data-testid="${testId}"]`;
    const parts: string[] = [];
    let cur: Element | null = el;
    while (cur && parts.length < 6) {
      const current: Element = cur;
      let part = current.tagName.toLowerCase();
      const parent = current.parentElement;
      if (parent) {
        const same = [...parent.children].filter((c) => c.tagName === current.tagName);
        if (same.length > 1) part += `:nth-of-type(${same.indexOf(current) + 1})`;
      }
      parts.unshift(part);
      if (current.id) {
        parts[0] = `#${CSS.escape(current.id)}`;
        break;
      }
      cur = parent;
    }
    return parts.join(" > ");
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
    if (tagName === "input") {
      const type = (el.getAttribute("type") || "text").toLowerCase();
      if (type === "checkbox" || type === "radio") return type;
      if (type === "search") return "searchbox";
      if (["button", "submit", "reset"].includes(type)) return "button";
      if (type === "range") return "slider";
      return "textbox";
    }
    return "";
  };
  const pick = (target: EventTarget | null) =>
    target instanceof Element ? target.closest(INTERACTIVE) || target : null;
  const report = (result: { target: string; css: string; role: string; selectorUsed: string }) =>
    console.log(prefix + JSON.stringify(result));
  let last: MouseEvent | null = null;
  const show = (e: MouseEvent | null, shift: boolean) => {
    if (!e) return;
    if (shift) {
      const x = Math.round(e.clientX);
      const y = Math.round(e.clientY);
      Object.assign(box.style, {
        left: `${x - 6}px`,
        top: `${y - 6}px`,
        width: "12px",
        height: "12px",
      });
      tag.textContent = `${x}, ${y}`;
      Object.assign(tag.style, { left: `${x + 10}px`, top: `${Math.max(0, y - 30)}px` });
      return;
    }
    const el = pick(e.target);
    if (!el) return;
    const r = el.getBoundingClientRect();
    Object.assign(box.style, {
      left: `${r.left}px`,
      top: `${r.top}px`,
      width: `${r.width}px`,
      height: `${r.height}px`,
    });
    tag.textContent = nameOf(el) || cssOf(el);
    Object.assign(tag.style, { left: `${r.left}px`, top: `${Math.max(0, r.top - 26)}px` });
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
    },
    true,
  );
}
