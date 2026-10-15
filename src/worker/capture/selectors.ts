import type { Locator, Page } from "playwright-core";

export type Candidate = { by: string; locator: Locator };

export type SelectorHint = {
  by: string;
  kind: "role" | "label" | "placeholder" | "testid" | "text" | "css";
  role?: string;
  name?: string;
  text?: string;
  css?: string;
  exact?: boolean;
};

const ARIA_ROLES = new Set([
  "alert",
  "alertdialog",
  "application",
  "article",
  "banner",
  "blockquote",
  "button",
  "caption",
  "cell",
  "checkbox",
  "code",
  "columnheader",
  "combobox",
  "complementary",
  "contentinfo",
  "definition",
  "deletion",
  "dialog",
  "directory",
  "document",
  "emphasis",
  "feed",
  "figure",
  "form",
  "generic",
  "grid",
  "gridcell",
  "group",
  "heading",
  "img",
  "insertion",
  "link",
  "list",
  "listbox",
  "listitem",
  "log",
  "main",
  "marquee",
  "math",
  "menu",
  "menubar",
  "menuitem",
  "menuitemcheckbox",
  "menuitemradio",
  "meter",
  "navigation",
  "none",
  "note",
  "option",
  "paragraph",
  "presentation",
  "progressbar",
  "radio",
  "radiogroup",
  "region",
  "row",
  "rowgroup",
  "rowheader",
  "scrollbar",
  "search",
  "searchbox",
  "separator",
  "slider",
  "spinbutton",
  "status",
  "strong",
  "subscript",
  "superscript",
  "switch",
  "tab",
  "table",
  "tablist",
  "tabpanel",
  "term",
  "textbox",
  "time",
  "timer",
  "toolbar",
  "tooltip",
  "tree",
  "treegrid",
  "treeitem",
]);

function nameMatcher(name: string, exact = true): RegExp {
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return new RegExp(exact ? `^${escaped}$` : `^${escaped}`, "i");
}

function containsMatcher(text: string): RegExp {
  return new RegExp(text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i");
}

const NAMED_ROLES = ["button", "link", "textbox", "combobox", "option", "tab"] as const;

const INTERACTIVE_CSS =
  'button, a, input, textarea, select, [role="button"], [role="link"], ' +
  '[role="menuitem"], [role="tab"], [role="option"], [role="combobox"], ' +
  '[role="checkbox"], [role="switch"]';

const INTERACTIVE_ANCESTOR =
  'xpath=ancestor-or-self::*[@role="button" or @role="link" or @role="menuitem"' +
  ' or @role="tab" or @role="option" or @role="combobox" or self::button or' +
  " self::a][1]";

export function autoCandidates(page: Page, name: string): Candidate[] {
  const exact = nameMatcher(name, true);
  const loose = containsMatcher(name);
  const out: Candidate[] = NAMED_ROLES.map((role) => ({
    by: `role=${role} name="${name}"`,
    locator: page.getByRole(role, { name: exact }),
  }));
  out.push({ by: `label="${name}"`, locator: page.getByLabel(exact) });
  out.push({
    by: `placeholder~"${name}"`,
    locator: page.getByPlaceholder(loose),
  });
  out.push({ by: `testid="${name}"`, locator: page.getByTestId(name) });
  out.push({
    by: `control containing "${name}"`,
    locator: page.locator(INTERACTIVE_CSS).filter({ hasText: loose }),
  });
  out.push({
    by: `text "${name}" -> control`,
    locator: page.getByText(loose).locator(INTERACTIVE_ANCESTOR),
  });
  out.push({ by: `text "${name}"`, locator: page.getByText(loose) });
  return out;
}

export type TargetOverrides = Record<string, (page: Page) => Candidate[]>;

export function candidatesFor(page: Page, name: string, overrides?: TargetOverrides): Candidate[] {
  const override = overrides?.[name];
  return override ? override(page) : autoCandidates(page, name);
}

export function findByName(
  page: Page,
  name: string,
  overrides?: TargetOverrides,
  opts: { timeoutMs?: number } = {},
): Promise<Locator> {
  return resolveFirst(page, name, candidatesFor(page, name, overrides), opts);
}

export function softByName(
  page: Page,
  name: string,
  overrides?: TargetOverrides,
): Promise<Locator | null> {
  return softFirst(candidatesFor(page, name, overrides));
}

function locatorFromHint(page: Page, hint: SelectorHint): Locator {
  switch (hint.kind) {
    case "role": {
      const role = (hint.role || "").toLowerCase();
      if (!ARIA_ROLES.has(role)) return page.locator("html:not(*)");
      const name = hint.name ? nameMatcher(hint.name, hint.exact !== false) : undefined;
      return page.getByRole(role as Parameters<Page["getByRole"]>[0], {
        name,
      });
    }
    case "label":
      return page.getByLabel(nameMatcher(hint.name || hint.text || "", hint.exact !== false));
    case "placeholder":
      return page.getByPlaceholder(nameMatcher(hint.text || hint.name || "", hint.exact !== false));
    case "testid":
      return page.getByTestId(hint.text || "");
    case "text":
      return page.getByText(nameMatcher(hint.text || "", hint.exact !== false));
    case "css":
      return page.locator(hint.css || hint.by);
  }
}

export function candidatesFromHints(page: Page, hints: SelectorHint[]): Candidate[] {
  return hints.map((h) => ({
    by: h.by,
    locator: locatorFromHint(page, h),
  }));
}

export class SelectorError extends Error {
  constructor(
    public readonly label: string,
    public readonly tried: string[],
    public readonly dump: string,
  ) {
    super(`Couldn't find "${label}" on the page.\n  Looked for: ${tried.join(" | ")}\n${dump}`);
    this.name = "SelectorError";
  }
}

const OK = "✓";
const NO = "✗";

export async function dumpVisible(page: Page, max = 24): Promise<string> {
  try {
    const items = (await page.evaluate(`(function(){
      function vis(e){var r=e.getBoundingClientRect();var s=getComputedStyle(e);
        return r.width>2&&r.height>2&&s.visibility!=='hidden'&&s.display!=='none'&&s.opacity!=='0'&&r.bottom>0&&r.top<innerHeight&&r.right>0&&r.left<innerWidth;}
      var sel='button,[role=button],[role=combobox],[role=option],a,input,textarea,select,.ant-select';
      var els=[].slice.call(document.querySelectorAll(sel));
      var out=[];
      for(var i=0;i<els.length;i++){var e=els[i];if(!vis(e))continue;var r=e.getBoundingClientRect();
        var t=(e.getAttribute('aria-label')||e.getAttribute('placeholder')||e.textContent||'').trim().replace(/\\s+/g,' ').slice(0,44);
        out.push({tag:e.tagName.toLowerCase(),role:e.getAttribute('role')||'',type:e.getAttribute('type')||'',t:t,x:Math.round(r.x),y:Math.round(r.y),w:Math.round(r.width),h:Math.round(r.height)});
      }
      return out.slice(0, ${max});
    })()`)) as Array<{
      tag: string;
      role: string;
      type: string;
      t: string;
      x: number;
      y: number;
      w: number;
      h: number;
    }>;
    const lines = items.map(
      (e) =>
        `    · <${e.tag}${e.role ? ` role=${e.role}` : ""}${e.type ? ` type=${e.type}` : ""}> "${e.t}" @${e.x},${e.y} ${e.w}×${e.h}`,
    );
    return `  on screen now:\n${lines.join("\n") || "    (no visible interactive elements)"}`;
  } catch {
    return "  (could not snapshot page)";
  }
}

export async function softFirst(candidates: Candidate[]): Promise<Locator | null> {
  for (const c of candidates) {
    try {
      const loc = c.locator.first();
      if (await loc.isVisible()) return loc;
    } catch {}
  }
  return null;
}

export async function resolveFirst(
  page: Page,
  label: string,
  candidates: Candidate[],
  opts: { timeoutMs?: number } = {},
): Promise<Locator> {
  const timeoutMs = opts.timeoutMs ?? 15000;
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    for (const c of candidates) {
      try {
        const loc = c.locator.first();
        if (await loc.isVisible()) {
          console.log(`  ${OK} ${label} via ${c.by}`);
          return loc;
        }
      } catch {}
    }
    await page.waitForTimeout(250);
  }
  const tried = candidates.map((c) => c.by);
  console.error(`  ${NO} ${label} — none of [${tried.join(", ")}] visible in ${timeoutMs}ms`);
  throw new SelectorError(label, tried, await dumpVisible(page));
}
