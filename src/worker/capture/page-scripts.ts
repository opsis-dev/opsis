export const COOKIE_BANNER_SELECTORS = [
  "#onetrust-banner-sdk",
  "#onetrust-consent-sdk",
  "#CybotCookiebotDialog",
  ".cc-window",
  ".cookie-banner",
  "[id*='cookie-banner' i]",
  "[class*='cookie-consent' i]",
  "[aria-label*='cookie' i][role='dialog']",
];

export const CHAT_WIDGET_SELECTORS = [
  "#intercom-container",
  ".intercom-lightweight-app",
  "#hubspot-messages-iframe-container",
  ".drift-frame-controller",
  "#crisp-chatbox",
  "iframe[title*='chat' i]",
  "[id*='chat-widget' i]",
];

export const hidingCss = (selectors: readonly string[]): string =>
  `${selectors.join(",\n")} { display: none !important; visibility: hidden !important; }`;

export function addStyles(css: string): void {
  const style = document.createElement("style");
  style.textContent = css;
  const add = () => (document.head ?? document.documentElement).appendChild(style);
  if (document.documentElement) {
    add();
    return;
  }
  const watch = new MutationObserver(() => {
    if (!document.documentElement) return;
    watch.disconnect();
    add();
  });
  watch.observe(document, { childList: true });
}

const userScript = (body: string, { async = false } = {}): string =>
  `(${async ? "async " : ""}() => {\n${body}\n})()`;

export const runUserScript = (script: string): string => userScript(script, { async: true });

export function beforeFirstLoad(id: string, script: string): string {
  const key = JSON.stringify(`demoflow-script:${id}`);
  const firstTime = [
    "try {",
    `  if (sessionStorage.getItem(${key})) return;`,
    `  sessionStorage.setItem(${key}, "1");`,
    "} catch {",
    "  return;",
    "}",
  ].join("\n");
  return userScript(`${firstTime}\n${script}`);
}

const HOTSPOT = { x: 2, y: 2 };
const OFF_SCREEN_PX = -300;
const PRESSED_SCALE = 0.9;
const RIPPLE_MS = 350;
const RIPPLE_CLEANUP_MS = 30;
const CURSOR_Z = 2147483647;

const ARROW_SVG = `<svg width="48" height="64" viewBox="0 0 24 32" xmlns="http://www.w3.org/2000/svg">
<path d="M1 1 L1 23 L6.5 17.8 L10.2 26.6 L13.7 25 L9.95 16.5 L17 16.3 Z" fill="#111418" stroke="#ffffff" stroke-width="1.85" stroke-linejoin="round" paint-order="stroke"/>
</svg>`;

const CURSOR_CSS = `
* { cursor: none !important; }
#pw-cursor {
  position: fixed; top: 0; left: 0; pointer-events: none; z-index: ${CURSOR_Z};
  transform: translate(${OFF_SCREEN_PX}px, ${OFF_SCREEN_PX}px);
  transform-origin: ${HOTSPOT.x}px ${HOTSPOT.y}px;
  filter: drop-shadow(0 0 0.5px #fff) drop-shadow(0 1px 0 #fff) drop-shadow(0 0 2px #fff) drop-shadow(0 5px 12px rgba(0,0,0,.6));
}
#pw-cursor svg { display: block; }
.pw-ripple {
  position: fixed; width: 22px; height: 22px; margin: -11px 0 0 -11px; border-radius: 50%;
  border: 2px solid rgba(15,23,42,.55); background: rgba(15,23,42,.1);
  pointer-events: none; z-index: ${CURSOR_Z - 1};
  animation: pw-ripple ${RIPPLE_MS}ms ease-out forwards;
}
@keyframes pw-ripple { from { transform: scale(.3); opacity: .85; } to { transform: scale(1.35); opacity: 0; } }
`;

type CursorLook = {
  hotspot: { x: number; y: number };
  arrow: string;
  css: string;
  offScreen: number;
  pressedScale: number;
  rippleMs: number;
};

export const CURSOR_LOOK: CursorLook = {
  hotspot: HOTSPOT,
  arrow: ARROW_SVG,
  css: CURSOR_CSS,
  offScreen: OFF_SCREEN_PX,
  pressedScale: PRESSED_SCALE,
  rippleMs: RIPPLE_MS + RIPPLE_CLEANUP_MS,
};

export function installCursor(look: CursorLook): void {
  const { hotspot } = look;
  const mount = () => {
    if (document.getElementById("pw-cursor") || !document.body) return;
    const style = document.createElement("style");
    style.textContent = look.css;
    document.documentElement.appendChild(style);
    const cursor = document.createElement("div");
    cursor.id = "pw-cursor";
    cursor.innerHTML = look.arrow;
    document.body.appendChild(cursor);
    let x = look.offScreen;
    let y = look.offScreen;
    let down = false;
    const place = (scale: number) => {
      cursor.style.transform = `translate(${x - hotspot.x}px,${y - hotspot.y}px) scale(${scale})`;
    };
    const follow = (e: MouseEvent) => {
      x = e.clientX;
      y = e.clientY;
      place(down ? look.pressedScale : 1);
    };
    const release = () => {
      down = false;
      place(1);
    };
    const press = () => {
      down = true;
      place(look.pressedScale);
      const ripple = document.createElement("div");
      ripple.className = "pw-ripple";
      ripple.style.left = `${x}px`;
      ripple.style.top = `${y}px`;
      document.body.appendChild(ripple);
      setTimeout(() => ripple.remove(), look.rippleMs);
    };
    document.addEventListener("mousemove", follow, true);
    document.addEventListener("dragover", follow, true);
    document.addEventListener("mousedown", press, true);
    document.addEventListener("mouseup", release, true);
    document.addEventListener("drop", release, true);
    document.addEventListener("dragend", release, true);
  };
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", mount);
  else mount();
}
