import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { ROOT } from "./lib/tool-config";

const ELEMENTS_DIR = path.join(ROOT, "crates", "compile", "src", "elements");

const DIRS = ["camera", "three"];

const ELEMENT_FILES = [
  "backgrounds.rs",
  "placeholder.rs",
  "framed_image.rs",
  "overlay_frame.rs",
  "background/backdrop.rs",
  "background/gradient.rs",
  "background/noise.rs",
  "background/paper.rs",
  "background/shapes_3d.rs",
  "background/solid.rs",
  "background/starburst.rs",
  "device/screen_3d.rs",
  "effect/redact.rs",
  "media/image.rs",
  "media/video.rs",
  "overlay/callout.rs",
  "overlay/captions.rs",
  "overlay/light_leak.rs",
  "overlay/lottie.rs",
  "overlay/lower_third.rs",
  "scene/card_3d.rs",
  "scene/logo_3d.rs",
  "scene/model_3d.rs",
  "text/kinetic_headline.rs",
  "text/text_3d.rs",
  "text/title.rs",
  "transition/clock_wipe.rs",
  "transition/fade.rs",
  "transition/flip.rs",
  "transition/iris.rs",
  "transition/slide.rs",
  "transition/wipe.rs",
];

const COLOR_LITERAL = /#[0-9a-fA-F]{3,8}\b|\b(?:rgba?|hsla?|oklch)\([^)]*\)/g;

const SETTING_START = "#[setting(";

const WINDOW_THEME = "title bar, set by the frame theme";

const ALLOWED: Record<string, Record<string, string>> = {
  "camera/annotations.rs": { "rgba(0,0,0,0.25)": "tooltip shadow" },
  "camera/keys.rs": { "rgba(0,0,0,{})": "typed text chip shadow, its strength a setting" },
  "camera/recording.rs": {
    "#000": "stage behind a take before anything is drawn",
    "#0b0b10": "bleed behind a full-page recording while it loads",
    "#faf6f1": "Stage style ground",
    "#fcfcfc": "page behind a full-page recording while it loads",
    "rgba(0,0,0,0.45)": "full-page recording shadow",
  },
  "camera/stage.rs": {
    "rgba(30,20,10,0.30)": "light rim glow",
    "rgba(255,255,255,0.10)": "dark rim glow",
    "rgba(255,255,255,0.14)": "dark rim line",
  },
  "camera/window.rs": {
    "#f0f2f5": `light ${WINDOW_THEME}`,
    "#fafbfc": `light ${WINDOW_THEME}`,
    "#232328": `dark ${WINDOW_THEME}`,
    "#2c2c31": `dark ${WINDOW_THEME}`,
    "rgba(0, 0, 0, 0.45)": `${WINDOW_THEME}: its edge`,
    "rgba(255, 255, 255, 0.06)": `${WINDOW_THEME}: the address field's edge`,
    "rgba(255, 255, 255, 0.07)": `${WINDOW_THEME}: the address field`,
    "rgba(255, 255, 255, 0.1)": `${WINDOW_THEME}: its buttons`,
    "rgba(255, 255, 255, 0.62)": `${WINDOW_THEME}: its text`,
    "rgba(255, 255, 255, 0.78)": `${WINDOW_THEME}: its icons`,
    "rgba({INK}, {})": "shadow and rim ink at a given strength",
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
  "device/screen_3d.rs": {
    "rgba(255,255,255,{strength})": "3D screen glare at the Glare setting's strength",
    "rgba(255,255,255,0)": "3D screen glare fading out",
  },
  "overlay/captions.rs": { "rgba(0,0,0,0.55)": "caption text shadow" },
  "overlay/light_leak.rs": {
    "#ff8a3d": "the leak's default colours, which the Light leak cut and the element share",
    "#ffd166": "the leak's default colours, which the Light leak cut and the element share",
    "#ff4f7b": "the leak's default colours, which the Light leak cut and the element share",
  },
  "placeholder.rs": {
    "#0c0c10": "empty clip ground, never in a finished film",
    "#e4e4e7": "empty clip title, never in a finished film",
    "#a1a1aa": "empty clip subtitle, never in a finished film",
    "rgba(255,255,255,0.05)": "empty clip sweep, never in a finished film",
    "rgba(255,255,255,0.18)": "empty clip border, never in a finished film",
  },
  "scene/card_3d.rs": { "#ffffff": "the card's face and its edge when Edge colour is empty" },
};

function sourceFiles(): string[] {
  return [
    ...ELEMENT_FILES,
    ...DIRS.flatMap((dir) =>
      readdirSync(path.join(ELEMENTS_DIR, dir), { recursive: true, withFileTypes: true })
        .filter((entry) => entry.isFile() && entry.name.endsWith(".rs"))
        .map((entry) =>
          path
            .relative(ELEMENTS_DIR, path.join(entry.parentPath, entry.name))
            .split(path.sep)
            .join("/"),
        ),
    ),
  ];
}

function withoutSettings(text: string): string {
  let out = "";
  let at = 0;
  for (;;) {
    const start = text.indexOf(SETTING_START, at);
    if (start < 0) return out + text.slice(at);
    out += text.slice(at, start);
    let depth = 1;
    let end = start + SETTING_START.length;
    while (depth > 0 && end < text.length) {
      if (text[end] === "(") depth++;
      if (text[end] === ")") depth--;
      end++;
    }
    at = end;
  }
}

function literalsIn(file: string): string[] {
  const text = withoutSettings(readFileSync(path.join(ELEMENTS_DIR, file), "utf8"));
  return [...new Set(text.match(COLOR_LITERAL) ?? [])];
}

describe("colour literals in the film's own element code", () => {
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
