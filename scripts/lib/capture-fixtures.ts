import { pressedChord } from "@opsis/core/chord";
import type { Point } from "@opsis/core/schema/actions";
import {
  cameraEase,
  captureCanvas,
  captureScale,
  paddedRegion,
  tallestShot,
} from "@opsis/video/capture";
import { scaledViewport } from "../../src/worker/capture/capture-scale";
import { cursorKeyword } from "../../src/worker/capture/cursor-keyword";
import {
  boundsOf,
  bulgedCurve,
  dragDuration,
  glidePlan,
  nudgeToward,
  offsetPath,
  polylineAt,
  thinPath,
} from "../../src/worker/capture/drag-path";
import { beforeFirstLoad, hidingCss } from "../../src/worker/capture/page-scripts";
import { playwrightInternals } from "./playwright-bundle";

const STEPS = Array.from({ length: 21 }, (_, i) => i / 20);

const sampled = (at: (p: number) => Point) => STEPS.map((p) => at(p));

function easeCases() {
  return [-0.5, ...Array.from({ length: 101 }, (_, i) => i / 100), 1.5].map((t) => ({
    t,
    eased: cameraEase(t),
  }));
}

function dragCases() {
  const viewport = { width: 1920, height: 1080 };
  const glides = [
    { from: { x: 960, y: 540 }, to: { x: 960, y: 207 }, scale: 1, viewport, travelMs: undefined },
    { from: { x: 0, y: 0 }, to: { x: 1900, y: 1070 }, scale: 1, viewport, travelMs: undefined },
    { from: { x: 100, y: 900 }, to: { x: 1300, y: 50 }, scale: 2, viewport, travelMs: 700 },
    { from: { x: 10, y: 10 }, to: { x: 12, y: 11 }, scale: 0.5, viewport: null, travelMs: 0 },
    { from: { x: 500, y: 500 }, to: { x: 500, y: 500 }, scale: 1, viewport, travelMs: undefined },
  ].map((g) => {
    const plan = glidePlan(g.from, g.to, g.scale, g.viewport, g.travelMs);
    return {
      ...g,
      durationMs: plan.durationMs,
      aim: plan.aim,
      overshooting: plan.overshooting,
      samples: sampled(plan.at),
    };
  });
  const curves = [
    [
      { x: 0, y: 0 },
      { x: 300, y: 200 },
    ],
    [
      { x: 300, y: 0 },
      { x: 0, y: 200 },
    ],
    [
      { x: 5, y: 5 },
      { x: 5, y: 5 },
    ],
  ].map(([from, to]) => ({ from, to, samples: sampled(bulgedCurve(from, to)) }));
  const paths = [
    [
      { x: 0, y: 0 },
      { x: 100, y: 0 },
      { x: 100, y: 50 },
    ],
    [{ x: 7, y: 7 }],
    [
      { x: 0, y: 0 },
      { x: 0, y: 0 },
      { x: 30, y: 40 },
    ],
  ];
  const long = Array.from({ length: 200 }, (_, i) => ({ x: i * 1.25, y: Math.sin(i / 9) * 40.5 }));
  return {
    glides,
    curves,
    polylines: paths.map((points) => ({ points, samples: sampled(polylineAt(points)) })),
    durations: [
      { points: paths[0], scale: 1, dragMs: 0 },
      {
        points: [
          { x: 0, y: 0 },
          { x: 3000, y: 0 },
        ],
        scale: 1,
        dragMs: 0,
      },
      {
        points: [
          { x: 0, y: 0 },
          { x: 10, y: 0 },
        ],
        scale: 2,
        dragMs: 0,
      },
      { points: paths[0], scale: 1, dragMs: 900 },
    ].map((c) => ({ ...c, durationMs: dragDuration(c.points, c.scale, c.dragMs) })),
    nudges: [
      { from: { x: 10, y: 10 }, to: { x: 110, y: 10 }, px: 6 },
      { from: { x: 10, y: 10 }, to: { x: 12, y: 13 }, px: 12 },
      { from: { x: 10, y: 10 }, to: { x: 10, y: 10 }, px: 6 },
    ].map((c) => ({ ...c, point: nudgeToward(c.from, c.to, c.px) })),
    offsets: [
      {
        start: { x: 100, y: 100 },
        offsets: [
          { x: 50, y: 0 },
          { x: 50, y: 40 },
        ],
      },
      {
        start: { x: 100, y: 100 },
        offsets: [
          { x: 0, y: 0 },
          { x: 20, y: 0 },
        ],
      },
    ].map((c) => ({ ...c, points: offsetPath(c.start, c.offsets) })),
    thins: [paths[0], long].map((points) => ({ points, thinned: thinPath(points) })),
    bounds: [paths[0], long].map((points) => ({ points, rect: boundsOf(points) })),
  };
}

function stillCases() {
  const looks = ["window", "cutout", "bleed"] as const;
  const regions = [
    { width: 1280, height: 800 },
    { width: 390, height: 5200 },
    { width: 3840, height: 2160 },
    { width: 320, height: 48 },
  ];
  const projects = [
    { width: 1920, height: 1080 },
    { width: 1080, height: 1920 },
  ];
  const scales = looks.flatMap((look) =>
    regions.flatMap((region) =>
      projects.flatMap((project) =>
        [1, 2].flatMap((least) =>
          [undefined, { x: 24, y: 24, width: region.width - 48, height: region.height - 48 }].map(
            (body) => {
              const canvas = captureCanvas(project);
              return {
                look,
                region,
                project,
                least,
                body,
                canvas,
                scale: captureScale(look, region, canvas, { body, fit: 0.86, least }),
              };
            },
          ),
        ),
      ),
    ),
  );
  return {
    scales,
    tallest: looks.flatMap((look) =>
      projects.map((project) => ({
        look,
        width: 1440,
        project,
        tallest: tallestShot(look, 1440, captureCanvas(project)),
      })),
    ),
    padded: [
      { box: { x: 10, y: 20, width: 300, height: 120 }, scroll: { x: 0, y: 400 }, padding: 24 },
      { box: { x: 5, y: 5, width: 30, height: 10 }, scroll: { x: 0, y: 0 }, padding: 40 },
    ].map((c) => ({ ...c, ...paddedRegion(c.box, c.scroll, c.padding) })),
    viewports: [
      { logical: { width: 1280, height: 800 }, scale: 1.5 },
      { logical: { width: 1441, height: 901 }, scale: 1.3 },
    ].map((c) => ({ ...c, physical: scaledViewport(c.logical, c.scale) })),
  };
}

function pageCases() {
  return {
    cursors: [
      { cursor: "url(https://x.test/a.png) 4 4, pointer", editable: false },
      { cursor: "auto", editable: true },
      { cursor: "auto", editable: false },
      { cursor: "url(a.cur), url('b.png') 1 2, ", editable: false },
      { cursor: "Grab", editable: false },
      { cursor: "-webkit-grab", editable: false },
      { cursor: "x y", editable: false },
    ].map((under) => ({ under, keyword: cursorKeyword(under) })),
    chords: ["Ctrl+Shift+K", "cmd+a", "Enter", "Shift+Tab", "alt+ArrowDown", "Mod+S", "a"].map(
      (chord) => ({
        chord,
        pressed: pressedChord(chord),
      }),
    ),
    hiding: [["#a", ".b c", "[id*='x' i]"]].map((selectors) => ({
      selectors,
      css: hidingCss(selectors),
    })),
    scripts: [{ id: "n1", script: "localStorage.setItem('a', '1');" }].map((c) => ({
      ...c,
      text: beforeFirstLoad(c.id, c.script),
    })),
  };
}

function playwrightCases() {
  const pw = playwrightInternals();
  const names = [
    "What needs to be done?",
    "Toggle Todo",
    "Save (draft) [2]",
    'a"b\\c',
    "Ünïcode ✓",
    "a/b >> c",
    "it's `x`",
  ];
  const escape = (s: string) => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const roles = ["button", "link", "textbox", "combobox", "option", "tab"];
  return {
    keyMap: [...pw.keyMap.entries()].map(([name, description]) => ({ name, description })),
    selectors: names.map((name) => {
      const exact = new RegExp(`^${escape(name)}$`, "i");
      const loose = new RegExp(escape(name), "i");
      return {
        name,
        roles: roles.map((role) => pw.getByRoleSelector(role, { name: exact })),
        label: pw.getByLabelSelector(exact),
        placeholder: pw.getByPlaceholderSelector(loose),
        testId: pw.getByTestIdSelector("data-testid", name),
        text: pw.getByTextSelector(loose),
        hasText: pw.escapeForTextSelector(loose, false),
        exactText: pw.getByTextSelector(name, { exact: true }),
        looseText: pw.getByTextSelector(name, { exact: false }),
      };
    }),
    globs: [
      ["**/done*", "https://a.test/x/done1"],
      ["**/done*", "https://a.test/x/notdone/y"],
      ["*/dashboard*", "https://app.test/dashboard"],
      ["https://a.test/*", "https://a.test/b"],
      ["https://a.test/*", "https://a.test/b/c"],
      ["**/api/**", "https://a.test/api/v1/items?x=1"],
      ["**/*.{png,jpg}", "https://a.test/i/x.jpg"],
      ["https://a.test/?a=1", "https://a.test/?a=1"],
      ["http://localhost:3000/[id]", "http://localhost:3000/[id]"],
    ].map(([glob, url]) => ({
      glob,
      url,
      pattern: pw.globToRegexPattern(glob),
      matches: pw.urlMatches(undefined, url, glob),
    })),
  };
}

export function captureFixtures(): Record<string, unknown> {
  return {
    "capture/ease.json": easeCases(),
    "capture/drag.json": dragCases(),
    "capture/still.json": stillCases(),
    "capture/page.json": pageCases(),
    "capture/playwright.json": playwrightCases(),
  };
}
