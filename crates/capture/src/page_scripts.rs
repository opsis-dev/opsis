use std::sync::LazyLock;

use opsis_compile::nodes::capture::Record;
use opsis_types::secrets::SavedItem;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const DEFAULT_CURSOR: &str = "default";

const INTERACTIVE_TAGS: [&str; 5] = ["button", "a", "input", "textarea", "select"];
const INTERACTIVE_ROLES: [&str; 8] = [
    "button", "link", "menuitem", "tab", "option", "checkbox", "switch", "combobox",
];

pub const COOKIE_BANNER_SELECTORS: [&str; 8] = [
    "#onetrust-banner-sdk",
    "#onetrust-consent-sdk",
    "#CybotCookiebotDialog",
    ".cc-window",
    ".cookie-banner",
    "[id*='cookie-banner' i]",
    "[class*='cookie-consent' i]",
    "[aria-label*='cookie' i][role='dialog']",
];

pub const CHAT_WIDGET_SELECTORS: [&str; 7] = [
    "#intercom-container",
    ".intercom-lightweight-app",
    "#hubspot-messages-iframe-container",
    ".drift-frame-controller",
    "#crisp-chatbox",
    "iframe[title*='chat' i]",
    "[id*='chat-widget' i]",
];

static URL_VALUE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)url\(\s*(?:"[^"]*"|'[^']*'|[^)]*)\s*\)(?:\s+[-+\d.]+){0,2}"#)
        .expect("pattern")
});
static KEYWORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z][a-z-]*$").expect("pattern"));

pub fn interactive_selector() -> String {
    INTERACTIVE_TAGS
        .iter()
        .map(|t| (*t).to_owned())
        .chain(INTERACTIVE_ROLES.iter().map(|r| format!("[role={r}]")))
        .collect::<Vec<_>>()
        .join(",")
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorUnder {
    pub cursor: String,
    pub editable: bool,
}

pub fn cursor_keyword(under: &CursorUnder) -> String {
    let stripped = URL_VALUE.replace_all(&under.cursor, "");
    let keyword = stripped
        .split(',')
        .map(|part| part.trim().to_lowercase())
        .rfind(|part| !part.is_empty())
        .unwrap_or_else(|| "auto".to_owned());
    if !KEYWORD.is_match(&keyword) {
        return DEFAULT_CURSOR.to_owned();
    }
    if keyword == "auto" {
        return if under.editable {
            "text".to_owned()
        } else {
            DEFAULT_CURSOR.to_owned()
        };
    }
    keyword
}

pub fn hiding_css(selectors: &[String]) -> String {
    selectors
        .iter()
        .map(|s| format!("{s} {{ display: none !important; visibility: hidden !important; }}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn user_script(body: &str, run_async: bool) -> String {
    format!(
        "({}() => {{\n{body}\n}})()",
        if run_async { "async " } else { "" }
    )
}

pub fn run_user_script(script: &str) -> String {
    user_script(script, true)
}

pub fn before_first_load(id: &str, script: &str) -> String {
    let key = serde_json::to_string(&format!("opsis-script:{id}")).expect("strings serialize");
    let first_time = [
        "try {".to_owned(),
        format!("  if (sessionStorage.getItem({key})) return;"),
        format!("  sessionStorage.setItem({key}, \"1\");"),
        "} catch {".to_owned(),
        "  return;".to_owned(),
        "}".to_owned(),
    ]
    .join("\n");
    user_script(&format!("{first_time}\n{script}"), false)
}

pub fn call(function: &str, arg: &Value) -> String {
    format!("({function})({arg})")
}

pub const ADD_STYLES: &str = r#"function addStyles(css) {
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
}"#;

pub const INVALID_SELECTORS: &str = r#"function invalidSelectors(selectors) {
  return selectors.filter((selector) => {
    try {
      document.createDocumentFragment().querySelector(selector);
      return false;
    } catch {
      return true;
    }
  });
}"#;

pub const OPTION_VALUE: &str = r#"function optionValue(wanted) {
  const el = this;
  if (!(el instanceof HTMLSelectElement)) return null;
  const options = [...el.options];
  const option =
    options.find((o) => o.value === wanted) ??
    options.find((o) => o.label.trim() === wanted.trim());
  return option?.value;
}"#;

#[derive(Debug, Clone, Serialize)]
pub struct SavedStorage<'a> {
    pub key: String,
    pub origin: &'a str,
    pub items: &'a [SavedItem],
}

pub const RESTORE_LOCAL_STORAGE: &str = r#"function restoreLocalStorage({ key, origin, items }) {
  if (location.origin !== origin) return;
  try {
    if (sessionStorage.getItem(key)) return;
    sessionStorage.setItem(key, "1");
    for (const { name, value } of items) localStorage.setItem(name, value);
  } catch {
    return;
  }
}"#;

const HOTSPOT: (f64, f64) = (2.0, 2.0);
const OFF_SCREEN_PX: f64 = -300.0;
const PRESSED_SCALE: f64 = 0.9;
const RIPPLE_MS: f64 = 350.0;
const RIPPLE_CLEANUP_MS: f64 = 30.0;
const CURSOR_Z: i64 = 2_147_483_647;

fn arrow_svg(p: &Record) -> String {
    let (w, h) = (48.0 * p.cursor_scale, 64.0 * p.cursor_scale);
    let fill = html_escape::encode_double_quoted_attribute(&p.cursor_fill);
    let stroke = html_escape::encode_double_quoted_attribute(&p.cursor_outline);
    format!(
        r##"<svg width="{w}" height="{h}" viewBox="0 0 24 32" xmlns="http://www.w3.org/2000/svg">
<path d="M1 1 L1 23 L6.5 17.8 L10.2 26.6 L13.7 25 L9.95 16.5 L17 16.3 Z" fill="{fill}" stroke="{stroke}" stroke-width="1.85" stroke-linejoin="round" paint-order="stroke"/>
</svg>"##
    )
}

fn cursor_css(p: &Record) -> String {
    let ring = &p.ring_color;
    let shown = if p.click_ring { "block" } else { "none" };
    format!(
        r#"
* {{ cursor: none !important; }}
#pw-cursor {{
  position: fixed; top: 0; left: 0; pointer-events: none; z-index: {CURSOR_Z};
  transform: translate({off}px, {off}px);
  transform-origin: {hx}px {hy}px;
  filter: drop-shadow(0 0 0.5px {glow}) drop-shadow(0 1px 0 {glow}) drop-shadow(0 0 2px {glow}) drop-shadow(0 5px 12px rgba(0,0,0,.6));
}}
#pw-cursor svg {{ display: block; }}
.pw-ripple {{
  position: fixed; width: 22px; height: 22px; margin: -11px 0 0 -11px; border-radius: 50%;
  border: 2px solid {ring}; background: color-mix(in srgb, {ring} 18%, transparent); display: {shown};
  pointer-events: none; z-index: {below};
  animation: pw-ripple {RIPPLE_MS}ms ease-out forwards;
}}
@keyframes pw-ripple {{ from {{ transform: scale(.3); opacity: .85; }} to {{ transform: scale(1.35); opacity: 0; }} }}
"#,
        off = OFF_SCREEN_PX,
        hx = HOTSPOT.0 * p.cursor_scale,
        hy = HOTSPOT.1 * p.cursor_scale,
        glow = p.cursor_outline,
        below = CURSOR_Z - 1,
    )
}

pub fn cursor_look(p: &Record) -> Value {
    json!({
        "hotspot": { "x": HOTSPOT.0 * p.cursor_scale, "y": HOTSPOT.1 * p.cursor_scale },
        "arrow": arrow_svg(p),
        "css": cursor_css(p),
        "offScreen": OFF_SCREEN_PX,
        "pressedScale": PRESSED_SCALE,
        "rippleMs": RIPPLE_MS + RIPPLE_CLEANUP_MS,
    })
}

pub const INSTALL_CURSOR: &str = r#"function installCursor(look) {
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
    const place = (scale) => {
      cursor.style.transform = `translate(${x - hotspot.x}px,${y - hotspot.y}px) scale(${scale})`;
    };
    const follow = (e) => {
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
}"#;

pub const CURSOR_UNDER: &str = r#"function cursorUnder({ x, y }) {
  const textInputs = ["text", "search", "email", "url", "tel", "password", "number"];
  const el = document.elementFromPoint(x, y);
  if (!el) return { cursor: "default", editable: false };
  const editable =
    el instanceof HTMLTextAreaElement ||
    (el instanceof HTMLInputElement && textInputs.includes(el.type)) ||
    (el instanceof HTMLElement && el.isContentEditable);
  return { cursor: getComputedStyle(el).cursor, editable };
}"#;

pub const LIST_ON_SCREEN: &str = r#"function listOnScreen({ max, selector }) {
  const shown = (e) => {
    const r = e.getBoundingClientRect();
    const s = getComputedStyle(e);
    return (
      r.width > 2 &&
      r.height > 2 &&
      s.visibility !== "hidden" &&
      s.display !== "none" &&
      s.opacity !== "0" &&
      r.bottom > 0 &&
      r.top < innerHeight &&
      r.right > 0 &&
      r.left < innerWidth
    );
  };
  return Array.from(document.querySelectorAll(selector))
    .filter(shown)
    .slice(0, max)
    .map((e) => {
      const r = e.getBoundingClientRect();
      const text = e.getAttribute("aria-label") || e.getAttribute("placeholder") || e.textContent;
      return {
        tag: e.tagName.toLowerCase(),
        role: e.getAttribute("role") ?? "",
        type: e.getAttribute("type") ?? "",
        text: text.trim().replace(/\s+/g, " ").slice(0, 44),
        x: Math.round(r.x),
        y: Math.round(r.y),
        w: Math.round(r.width),
        h: Math.round(r.height),
      };
    });
}"#;

pub const FIT_PAGE_TO_SCALE: &str = r#"function fitPageToScale({ scale, width, height }) {
  const viewportUnit = /([-+]?[0-9]*\.?[0-9]+)(dvh|svh|lvh|vh|dvw|svw|lvw|vw)\b/g;
  const startupFrames = 30;
  const reapplyMs = 250;
  const zoom = String(scale);

  function pin(target, key, value) {
    try {
      Object.defineProperty(target, key, { get: () => value, configurable: true });
    } catch {}
  }

  function pinViewport() {
    pin(window, "innerWidth", width);
    pin(window, "innerHeight", height);
    if (!window.visualViewport) return;
    pin(window.visualViewport, "width", width);
    pin(window.visualViewport, "height", height);
  }

  const mayHaveViewportUnit = (value) => value.includes("v");

  function toPx(text) {
    return text.replace(
      viewportUnit,
      (_, amount, unit) => `${(parseFloat(amount) / 100) * (unit.endsWith("h") ? height : width)}px`,
    );
  }

  function rewriteStyle(style) {
    for (let i = 0; i < style.length; i++) {
      const prop = style[i];
      const value = style.getPropertyValue(prop);
      if (!mayHaveViewportUnit(value)) continue;
      const next = toPx(value);
      if (next !== value) style.setProperty(prop, next, style.getPropertyPriority(prop));
    }
  }

  function readableRules(sheet) {
    try {
      return Array.from(sheet.cssRules);
    } catch {
      return [];
    }
  }

  let seenSheets = 0;
  function rewriteNewSheets() {
    const sheets = document.styleSheets;
    if (sheets.length === seenSheets) return;
    seenSheets = sheets.length;
    for (const sheet of Array.from(sheets))
      for (const rule of readableRules(sheet))
        if ("style" in rule && rule.style instanceof CSSStyleDeclaration) rewriteStyle(rule.style);
  }

  function apply() {
    const root = document.documentElement;
    if (root && root.style.getPropertyValue("zoom") !== zoom) root.style.setProperty("zoom", zoom);
    rewriteNewSheets();
  }

  function applyForStartupFrames(frames = 0) {
    apply();
    if (frames + 1 < startupFrames) requestAnimationFrame(() => applyForStartupFrames(frames + 1));
  }

  pinViewport();
  document.addEventListener("DOMContentLoaded", apply);
  window.addEventListener("load", apply);
  setInterval(apply, reapplyMs);
  apply();
  requestAnimationFrame(() => applyForStartupFrames());
}"#;

pub const STEP_FRAME: &str = r#"async function stepFrame(ms) {
  const clock = globalThis.__pwClock?.controller;
  if (clock) {
    try {
      await clock.runFor(ms);
    } catch (error) {
      queueMicrotask(() => {
        throw error;
      });
    }
  }
  const step = (doc) => {
    for (const animation of doc.getAnimations()) {
      if (animation.playState === "running")
        animation.currentTime = (animation.currentTime ?? 0) + ms * animation.playbackRate;
    }
    for (const frame of doc.querySelectorAll("iframe")) {
      let inner = null;
      try {
        inner = frame.contentDocument;
      } catch {}
      if (inner) step(inner);
    }
  };
  step(document);
  return {
    page: clock ? { time: clock.now(), fixed: clock._now.isFixedTime } : null,
    left: visualViewport.pageLeft,
    top: visualViewport.pageTop,
  };
}"#;

pub const WHOLLY_ON_SCREEN: &str = "function() { const r = this.getBoundingClientRect(); return r.top >= 0 && r.left >= 0 && r.bottom <= innerHeight && r.right <= innerWidth; }";

pub const STEP_OVERLAYS: &str = r#"(injected, ms) => {
  for (const animation of injected._highlight?._glassPaneShadow?.getAnimations() ?? []) {
    if (animation.playState === "running")
      animation.currentTime = (animation.currentTime ?? 0) + ms * animation.playbackRate;
  }
}"#;

pub const HOLD_PAGE_CLOCK: &str = r#"async function holdPageClock({ time, keepFixed }) {
  const clock = globalThis.__pwClock.controller;
  const fixed = keepFixed && clock._now.isFixedTime;
  const at = time ?? clock.now();
  if (time !== null) clock.install(time);
  await clock.pauseAt(at);
  if (fixed) clock.setFixedTime(at);
}"#;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PageNow {
    pub time: f64,
    pub fixed: bool,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct SteppedPage {
    pub page: Option<PageNow>,
    pub left: f64,
    pub top: f64,
}

pub fn page_time_for_new_pages(now: PageNow, wall: f64) -> String {
    let fixed = if now.fixed {
        format!(" c.log('setFixedTime', {wall}, {});", now.time)
    } else {
        String::new()
    };
    format!(
        "if (window === window.top && globalThis.__pwClock) {{ const c = globalThis.__pwClock.controller; c.log('pauseAt', {wall}, {});{fixed} }}",
        now.time
    )
}

pub const PLAN_ELEMENT_SCROLL: &str = r#"function planElementScroll(block) {
  const scrollers = [];
  for (let node = this.parentElement; node; node = node.parentElement) {
    if (node.scrollHeight > node.clientHeight || node.scrollWidth > node.clientWidth) scrollers.push(node);
  }
  const root = document.scrollingElement;
  if (root && !scrollers.includes(root)) scrollers.push(root);
  const from = scrollers.map((s) => [s.scrollLeft, s.scrollTop]);
  this.scrollIntoView({ block, inline: "nearest", behavior: "instant" });
  const to = scrollers.map((s) => [s.scrollLeft, s.scrollTop]);
  scrollers.forEach((s, i) => s.scrollTo({ left: from[i][0], top: from[i][1], behavior: "instant" }));
  this.__opsisScroll = { scrollers, from, to };
}"#;

pub const STEP_ELEMENT_SCROLL: &str = r#"function stepElementScroll(progress) {
  const plan = this.__opsisScroll;
  if (!plan) return;
  plan.scrollers.forEach((s, i) => {
    const [fromX, fromY] = plan.from[i];
    const [toX, toY] = plan.to[i];
    s.scrollTo({ left: fromX + (toX - fromX) * progress, top: fromY + (toY - fromY) * progress, behavior: "instant" });
  });
}"#;
