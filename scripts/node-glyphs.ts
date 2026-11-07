import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { icons as lucide } from "lucide";
import { allocOutputs, buildPlan, interpPolar, resampleIcon } from "morphicons";
import { z } from "zod";
import { NOTICES, loadGlyph } from "./lib/glyph-sets";
import { ROOT } from "./lib/tool-config";

type IconNode = ReadonlyArray<readonly [string, Record<string, string>]>;

const OUT = path.join(ROOT, "crates", "ui", "glyphs.generated.json");
const FALLBACK_GLYPH = "lucide:box";
const MORPH_FROM = -0.05;
const MORPH_STEP = 0.05;
const MORPH_STEPS = 23;
const MORPH_DECIMALS = 10;
const ICONS = [
  "app-window",
  "arrow-left",
  "arrow-left-right",
  "arrow-right",
  "arrow-right-from-line",
  "arrow-right-left",
  "arrow-up-right",
  "audio-lines",
  "blend",
  "bookmark",
  "box",
  "braces",
  "captions",
  "check",
  "check-check",
  "chevron-down",
  "chevron-left",
  "chevron-right",
  "chevrons-up-down",
  "circle",
  "circle-alert",
  "circle-check",
  "circle-dot",
  "circle-slash",
  "circle-x",
  "clapperboard",
  "clock",
  "code",
  "code-xml",
  "columns-2",
  "combine",
  "command",
  "copy",
  "cpu",
  "crop",
  "crosshair",
  "download",
  "ellipsis",
  "external-link",
  "eye",
  "eye-off",
  "fast-forward",
  "file",
  "file-archive",
  "file-play",
  "file-plus",
  "file-search",
  "file-video",
  "film",
  "flashlight",
  "folder",
  "folder-kanban",
  "folder-open",
  "gallery-horizontal",
  "gauge",
  "git-branch",
  "globe",
  "grab",
  "grid-3x3",
  "grip-vertical",
  "hard-drive",
  "hash",
  "history",
  "hourglass",
  "image",
  "image-play",
  "info",
  "key-round",
  "keyboard",
  "languages",
  "laptop",
  "layers",
  "layout-grid",
  "link",
  "list",
  "list-checks",
  "list-video",
  "locate-fixed",
  "magnet",
  "map",
  "maximize",
  "maximize-2",
  "menu",
  "message-square",
  "minimize",
  "minus",
  "monitor",
  "moon",
  "mouse",
  "mouse-pointer",
  "mouse-pointer-2",
  "mouse-pointer-click",
  "move",
  "move-up-right",
  "music",
  "palette",
  "panel-bottom",
  "panel-left",
  "panel-left-open",
  "panel-right",
  "pause",
  "pipette",
  "play",
  "plug",
  "plus",
  "presentation",
  "radio",
  "ratio",
  "redo-2",
  "refresh-cw",
  "repeat",
  "reply",
  "rocket",
  "rotate-ccw",
  "rotate-cw",
  "ruler",
  "save",
  "scale",
  "scan",
  "scan-search",
  "scissors",
  "scroll",
  "scroll-text",
  "search",
  "server",
  "settings",
  "settings-2",
  "shapes",
  "shield-alert",
  "shield-check",
  "sliders-horizontal",
  "sliders-vertical",
  "smartphone",
  "sparkles",
  "spline",
  "square",
  "square-arrow-out-up-right",
  "square-chart-gantt",
  "square-dashed",
  "square-terminal",
  "sticky-note",
  "sun",
  "timer",
  "timer-off",
  "touchpad",
  "trash-2",
  "triangle-alert",
  "type",
  "undo-2",
  "ungroup",
  "upload",
  "video",
  "volume-2",
  "volume-x",
  "wallpaper",
  "wifi",
  "wind",
  "x",
  "zoom-in",
  "zoom-out",
];
const lookSchema = z.object({ glyph: z.string(), morph: z.string().optional() });
const catalogSchema = z.object({ nodes: z.array(z.object({ look: lookSchema.optional() })) });

function catalogLooks(): Array<z.infer<typeof lookSchema>> {
  const text = execFileSync(
    "cargo",
    ["run", "-q", "--release", "-p", "opsis-compile", "--bin", "editor-data", "catalog"],
    { cwd: ROOT, encoding: "utf8", maxBuffer: 1 << 26 },
  );
  return catalogSchema
    .parse(JSON.parse(text))
    .nodes.flatMap((node) => (node.look ? [node.look] : []));
}

function usedGlyphs(looks: ReadonlyArray<z.infer<typeof lookSchema>>): string[] {
  const used = new Set<string>([FALLBACK_GLYPH]);
  for (const look of looks) {
    used.add(look.glyph);
    if (look.morph) used.add(look.morph);
  }
  return [...used].sort();
}

function pascal(name: string): string {
  return name
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join("");
}

function lucideIcons(names: readonly string[]): Record<string, IconNode> {
  const table = lucide as unknown as Record<string, IconNode>;
  return Object.fromEntries(
    names.map((name) => {
      const node = table[pascal(name)];
      if (!node) throw new Error(`no lucide icon ${name}`);
      return [name, node];
    }),
  );
}

type Morph = { from: number; step: number; closed: boolean[]; frames: number[][][] };

function bakeMorph(from: IconNode, to: IconNode): Morph {
  const plan = buildPlan(resampleIcon(from), resampleIcon(to));
  const out = allocOutputs(plan);
  const frames = Array.from({ length: MORPH_STEPS }, (_, index) => {
    interpPolar(plan, MORPH_FROM + index * MORPH_STEP, out);
    return out.map((points) =>
      Array.from(points, (value) => Math.round(value * MORPH_DECIMALS) / MORPH_DECIMALS),
    );
  });
  return {
    from: MORPH_FROM,
    step: MORPH_STEP,
    closed: plan.items.map((item) => item.closed),
    frames,
  };
}

function bakeMorphs(
  looks: ReadonlyArray<z.infer<typeof lookSchema>>,
  glyphs: Record<string, IconNode>,
): Record<string, Morph> {
  const pairs = new Map<string, readonly [string, string]>();
  for (const look of looks)
    if (look.morph) pairs.set(`${look.glyph}>${look.morph}`, [look.glyph, look.morph]);
  return Object.fromEntries(
    [...pairs]
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([key, [from, to]]) => [key, bakeMorph(glyphs[from], glyphs[to])]),
  );
}

async function main(): Promise<void> {
  const notices = readFileSync(NOTICES, "utf8");
  const looks = catalogLooks();
  const names = usedGlyphs(looks);
  const glyphs: Record<string, IconNode> = Object.fromEntries(
    await Promise.all(names.map(async (name) => [name, await loadGlyph(name, notices)] as const)),
  );
  const morphs = bakeMorphs(looks, glyphs);
  writeFileSync(OUT, `${JSON.stringify({ glyphs, icons: lucideIcons(ICONS), morphs })}\n`);
  console.log(
    `${names.length} glyphs, ${ICONS.length} icons, ${Object.keys(morphs).length} morphs`,
  );
}

await main();
