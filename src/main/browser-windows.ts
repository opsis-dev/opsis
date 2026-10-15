import { BrowserWindow, session as electronSession, type Cookie } from "electron";
import type { PickRequest, PickResult } from "../shared/ipc";
import type { LoginRequest } from "../shared/worker-protocol";

const PICK_PREFIX = "__demoflow_pick__:";

const PICKER_SCRIPT = `(() => {
  if (window.__demoflowPicker) return;
  window.__demoflowPicker = true;
  const box = document.createElement('div');
  box.style.cssText = 'position:fixed;pointer-events:none;z-index:2147483647;border:2px solid #a78bfa;background:rgba(167,139,250,.15);border-radius:4px';
  const tag = document.createElement('div');
  tag.style.cssText = 'position:fixed;pointer-events:none;z-index:2147483647;background:#18181b;color:#fafafa;font:600 12px system-ui;padding:4px 8px;border-radius:6px';
  document.documentElement.appendChild(box);
  document.documentElement.appendChild(tag);
  const INTERACTIVE = 'button,a,input,textarea,select,[role=button],[role=link],[role=menuitem],[role=tab],[role=option],[role=checkbox],[role=switch],[role=combobox],label';
  const clean = (s) => (s || '').replace(/\\s+/g, ' ').trim();
  const nameOf = (el) => {
    const aria = el.getAttribute('aria-label'); if (aria) return clean(aria);
    if (el.id) { const lab = document.querySelector('label[for="' + CSS.escape(el.id) + '"]'); if (lab) return clean(lab.textContent); }
    const ph = el.getAttribute('placeholder'); if (ph) return clean(ph);
    const text = clean(el.innerText || el.textContent);
    if (text && text.length <= 60) return text;
    const tid = el.getAttribute('data-testid'); if (tid) return tid;
    const title = el.getAttribute('title'); if (title) return clean(title);
    return '';
  };
  const cssOf = (el) => {
    if (el.id && !/\\d{3,}/.test(el.id)) return '#' + CSS.escape(el.id);
    const tid = el.getAttribute('data-testid'); if (tid) return '[data-testid="' + tid + '"]';
    const parts = [];
    let cur = el;
    while (cur && cur.nodeType === 1 && parts.length < 6) {
      let part = cur.tagName.toLowerCase();
      const parent = cur.parentElement;
      if (parent) {
        const same = [...parent.children].filter((c) => c.tagName === cur.tagName);
        if (same.length > 1) part += ':nth-of-type(' + (same.indexOf(cur) + 1) + ')';
      }
      parts.unshift(part);
      if (cur.id) { parts[0] = '#' + CSS.escape(cur.id); break; }
      cur = parent;
    }
    return parts.join(' > ');
  };
  const roleOf = (el) => {
    const explicit = el.getAttribute('role'); if (explicit) return explicit;
    const tag = el.tagName.toLowerCase();
    if (tag === 'a' && el.hasAttribute('href')) return 'link';
    if (tag === 'button') return 'button';
    if (tag === 'select') return 'combobox';
    if (tag === 'textarea') return 'textbox';
    if (tag === 'img') return 'img';
    if (/^h[1-6]$/.test(tag)) return 'heading';
    if (tag === 'input') {
      const type = (el.getAttribute('type') || 'text').toLowerCase();
      if (type === 'checkbox' || type === 'radio') return type;
      if (type === 'search') return 'searchbox';
      if (['button', 'submit', 'reset'].includes(type)) return 'button';
      if (type === 'range') return 'slider';
      return 'textbox';
    }
    return '';
  };
  const pick = (target) => (target.closest && target.closest(INTERACTIVE)) || target;
  document.addEventListener('mousemove', (e) => {
    const el = pick(e.target);
    const r = el.getBoundingClientRect();
    Object.assign(box.style, { left: r.left + 'px', top: r.top + 'px', width: r.width + 'px', height: r.height + 'px' });
    tag.textContent = nameOf(el) || cssOf(el);
    Object.assign(tag.style, { left: r.left + 'px', top: Math.max(0, r.top - 26) + 'px' });
  }, true);
  document.addEventListener('click', (e) => {
    e.preventDefault(); e.stopPropagation();
    const el = pick(e.target);
    const name = nameOf(el);
    const css = cssOf(el);
    console.log('${PICK_PREFIX}' + JSON.stringify({ target: name || ('css=' + css), css, role: name ? roleOf(el) : '', selectorUsed: name ? 'accessible name' : 'css' }));
  }, true);
})()`;

function sandboxedWindow(
  parent: BrowserWindow | null,
  title: string,
  partition: string,
  size?: { width: number; height: number },
) {
  return new BrowserWindow({
    parent: parent ?? undefined,
    modal: false,
    width: size?.width ?? 1280,
    height: size?.height ?? 860,
    title,
    autoHideMenuBar: true,
    webPreferences: { partition, sandbox: true, contextIsolation: true, nodeIntegration: false },
  });
}

export function pickInWindow(parent: BrowserWindow | null, req: PickRequest): Promise<PickResult> {
  return new Promise((resolve) => {
    const win = sandboxedWindow(
      parent,
      "Pick an element — click it",
      `demoflow-picker-${Date.now()}`,
      req.viewport,
    );
    let result: PickResult = null;
    const inject = () =>
      void win.webContents.executeJavaScript(PICKER_SCRIPT).catch(() => undefined);
    win.webContents.on("did-finish-load", inject);
    win.webContents.on("console-message", (event) => {
      const message = event.message;
      if (!message.startsWith(PICK_PREFIX)) return;
      result = JSON.parse(message.slice(PICK_PREFIX.length)) as PickResult;
      win.close();
    });
    win.on("closed", () => resolve(result));
    void win.loadURL(req.url);
  });
}

const playwrightSameSite = (s: Cookie["sameSite"]) =>
  s === "strict" ? "Strict" : s === "no_restriction" ? "None" : "Lax";

export function loginInWindow(
  parent: BrowserWindow | null,
  req: LoginRequest,
): Promise<string | null> {
  return new Promise((resolve) => {
    const partition = `demoflow-login-${Date.now()}`;
    const win = sandboxedWindow(parent, "Log in — close this window when you're done", partition);
    const origins = new Map<string, Array<{ name: string; value: string }>>();
    let finished = false;

    const snapshot = async () => {
      try {
        const url = new URL(win.webContents.getURL());
        if (!/^https?:$/.test(url.protocol)) return;
        const entries = (await win.webContents.executeJavaScript(
          "Object.entries(localStorage)",
        )) as Array<[string, string]>;
        origins.set(
          url.origin,
          entries.map(([name, value]) => ({ name, value })),
        );
      } catch {
        return;
      }
    };

    const complete = async () => {
      if (finished) return;
      finished = true;
      clearTimeout(timer);
      clearInterval(poll);
      await snapshot();
      const cookies = await electronSession.fromPartition(partition).cookies.get({});
      const state = {
        cookies: cookies.map((c) => ({
          name: c.name,
          value: c.value,
          domain:
            c.hostOnly || !c.domain
              ? (c.domain ?? "").replace(/^\./, "")
              : c.domain.startsWith(".")
                ? c.domain
                : `.${c.domain}`,
          path: c.path ?? "/",
          expires: c.session || !c.expirationDate ? -1 : c.expirationDate,
          httpOnly: !!c.httpOnly,
          secure: !!c.secure,
          sameSite: playwrightSameSite(c.sameSite),
        })),
        origins: [...origins].map(([origin, localStorage]) => ({ origin, localStorage })),
      };
      if (!win.isDestroyed()) win.destroy();
      resolve(JSON.stringify(state));
    };

    const timer = setTimeout(() => void complete(), req.timeoutSec * 1000);
    let landedUrl: string | null = null;
    const check = () => {
      if (finished || win.isDestroyed()) return;
      void snapshot();
      const url = win.webContents.getURL();
      if (!url || url === "about:blank") return;
      if (landedUrl === null) landedUrl = url;
      else if (req.doneWhen === "url-change" && url !== landedUrl) void complete();
      if (req.doneWhen === "selector" && req.selector) {
        void win.webContents
          .executeJavaScript(`!!document.querySelector(${JSON.stringify(req.selector)})`)
          .then((found: boolean) => {
            if (found) void complete();
          })
          .catch(() => undefined);
      }
    };
    const poll = req.doneWhen === "selector" ? setInterval(check, 500) : undefined;
    win.webContents.on("did-finish-load", check);
    win.webContents.on("did-navigate-in-page", check);
    win.on("close", (event) => {
      if (finished) return;
      event.preventDefault();
      clearTimeout(timer);
      void complete();
    });
    void win.loadURL(req.url);
  });
}
