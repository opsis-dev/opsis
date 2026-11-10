import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

const DIRS = ["core", "camera", "three", "lib/backgrounds"];

const ELEMENT_FILES = [
  "elements/background/backdrop.tsx",
  "elements/background/gradient.tsx",
  "elements/background/noise.tsx",
  "elements/background/paper.tsx",
  "elements/background/shapes-3d.tsx",
  "elements/background/solid.tsx",
  "elements/background/starburst.tsx",
  "elements/device/screen-3d.tsx",
  "elements/effect/redact/index.tsx",
  "elements/media/image.tsx",
  "elements/media/video.tsx",
  "elements/overlay/callout.tsx",
  "elements/overlay/captions.tsx",
  "elements/overlay/light-leak.tsx",
  "elements/overlay/lottie.tsx",
  "elements/overlay/lower-third.tsx",
  "elements/scene/card-3d.tsx",
  "elements/scene/logo-3d.tsx",
  "elements/scene/model-3d.tsx",
  "elements/text/kinetic-headline.tsx",
  "elements/text/text-3d.tsx",
  "elements/text/title.tsx",
  "elements/transition/clock-wipe.tsx",
  "elements/transition/fade.tsx",
  "elements/transition/flip.tsx",
  "elements/transition/iris.tsx",
  "elements/transition/slide.tsx",
  "elements/transition/wipe.tsx",
];

const COLOR_LITERAL = /#[0-9a-fA-F]{3,8}\b|rgba?\([^)]*\)|hsla?\([^)]*\)|oklch\([^)]*\)/g;

const ALLOWED: Record<string, Record<string, string>> = {
  "camera/annotations.tsx": { "rgba(0,0,0,0.25)": "tooltip shadow" },
  "camera/demo-clip.tsx": {
    "#fcfcfc": "page behind a full-page recording while it loads",
    "#0b0b10": "bleed behind a full-page recording while it loads",
    "rgba(0,0,0,0.45)": "full-page recording shadow",
  },
  "camera/lib/style.ts": {
    "#faf6f1": "Stage style ground",
    "rgba(${INK}, ${alpha})": "shadow and rim ink",
  },
  "camera/stage.tsx": {
    "rgba(30,20,10,0.30)": "light rim glow",
    "rgba(255,255,255,0.10)": "dark rim glow",
    "rgba(255,255,255,0.14)": "dark rim line",
  },
  "camera/window-frame.tsx": {
    "#f0f2f5": "light title bar, set by the frame theme",
    "#fafbfc": "light title bar, set by the frame theme",
    "#232328": "dark title bar, set by the frame theme",
    "#2c2c31": "dark title bar, set by the frame theme",
    "rgba(0, 0, 0, 0.45)": "title bar text, set by the frame theme",
    "rgba(255, 255, 255, 0.07)": "title bar line, set by the frame theme",
    "rgba(255, 255, 255, 0.06)": "address bar fill, set by the frame theme",
    "rgba(255, 255, 255, 0.62)": "title bar text, set by the frame theme",
    "rgba(255, 255, 255, 0.78)": "address text, set by the frame theme",
    "rgba(255, 255, 255, 0.1)": "address bar line, set by the frame theme",
    "#ff5f57": "close light, part of the window style",
    "#e0443e": "close light rim",
    "#febc2e": "minimise light, part of the window style",
    "#dea123": "minimise light rim",
    "#28c840": "zoom light, part of the window style",
    "#1aab29": "zoom light rim",
    "rgba(0,0,0,0.12)": "window light edge",
    "#fff": "window body behind the page",
    "rgba(255,255,255,0.9)": "window top highlight",
  },
  "elements/overlay/captions.tsx": { "rgba(0,0,0,0.55)": "caption text shadow" },
  "core/placeholder.tsx": {
    "rgba(255,255,255,0.05)": "empty clip sweep, never in a finished film",
    "rgba(255,255,255,0.18)": "empty clip border, never in a finished film",
  },
  "core/recording-clip.tsx": {
    "rgba(0,0,0,${look.shadow ?? TYPED_CHIP.shadow})":
      "typed text chip shadow, its strength a setting",
  },
  "elements/device/screen-3d.tsx": {
    "rgba(255,255,255,${strength})": "3D screen glare at the Glare setting's strength",
    "rgba(255,255,255,0)": "3D screen glare fading out",
  },
  "elements/scene/card-3d.tsx": {
    "rgba(0,0,0,0.55)": "3D card's soft shadow",
    "rgba(0,0,0,0)": "3D card's soft shadow fading out",
  },
};

const root = import.meta.dirname;

function sourceFiles(): string[] {
  return [
    ...ELEMENT_FILES,
    ...DIRS.flatMap((dir) =>
      readdirSync(path.join(root, dir), { recursive: true, withFileTypes: true })
        .filter((e) => e.isFile() && /\.tsx?$/.test(e.name) && !/\.test\.tsx?$/.test(e.name))
        .map((e) => path.relative(root, path.join(e.parentPath, e.name)).split(path.sep).join("/")),
    ),
  ];
}

function literalsIn(file: string): string[] {
  return [...new Set(readFileSync(path.join(root, file), "utf8").match(COLOR_LITERAL) ?? [])];
}

describe("colour literals in Opsis's own video code", () => {
  const found = new Map(sourceFiles().map((file) => [file, literalsIn(file)]));

  it("are settings, or internal constants listed here on purpose", () => {
    const unlisted = [...found].flatMap(([file, literals]) =>
      literals.filter((l) => !Object.hasOwn(ALLOWED[file] ?? {}, l)).map((l) => `${file}: ${l}`),
    );
    expect(unlisted).toEqual([]);
  });

  it("lists nothing that is gone", () => {
    const stale = Object.entries(ALLOWED).flatMap(([file, literals]) =>
      Object.keys(literals)
        .filter((l) => !(found.get(file) ?? []).includes(l))
        .map((l) => `${file}: ${l}`),
    );
    expect(stale).toEqual([]);
  });
});
