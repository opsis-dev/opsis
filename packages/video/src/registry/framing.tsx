import { isThemeValue, themeColors } from "@opsis/core/elements/theme";
import type { ElementProps } from "@opsis/core/elements/types";
import { isRecord, isString } from "@opsis/core/guards";
import { numberOr } from "@opsis/core/math";
import type { TransitionPresentation } from "@remotion/transitions";
import type React from "react";
import { useLayoutEffect, useRef, useState } from "react";
import { AbsoluteFill, Img, Sequence, useVideoConfig } from "remotion";
import type { Size } from "#video/core/still-layout.ts";
import {
  defaultDarkTheme,
  defaultLightTheme,
  RemocnUIProvider,
} from "#video/lib/remocn-ui/theme.ts";
import { SnapCnUIProvider } from "#video/lib/snap-cn-ui/theme.ts";
import type {
  ElementComponent,
  Presentation,
  PresentationFactory,
  SceneSequencer,
} from "#video/registry/types.ts";

export const component = (element: React.FC<never>): ElementComponent =>
  element as ElementComponent;

export const sequencer = (Scenes: React.FC<never>, Item: React.FC<never>): SceneSequencer => ({
  Scenes: Scenes as SceneSequencer["Scenes"],
  Item: Item as SceneSequencer["Item"],
});

export const presentation = <P extends Record<string, unknown>>(
  make: (props: never, size: Size) => TransitionPresentation<P>,
): PresentationFactory => ({
  create: make as (props: ElementProps, size: Size) => Presentation,
});

export function centered(Component: ElementComponent): ElementComponent {
  const Centered: ElementComponent = (props) => (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", textAlign: "center" }}>
      <Component {...props} />
    </AbsoluteFill>
  );
  return Centered;
}

const ScreenFit: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const box = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState<{ width: number; height: number } | null>(null);
  useLayoutEffect(() => {
    const el = box.current;
    if (!el) return;
    const measure = () => {
      const width = Math.round(el.clientWidth);
      const height = Math.round(el.clientHeight);
      if (width > 0 && height > 0)
        setSize((prev) =>
          prev && prev.width === width && prev.height === height ? prev : { width, height },
        );
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    return () => observer.disconnect();
  }, []);
  return (
    <div ref={box} style={{ position: "absolute", inset: 0, overflow: "hidden" }}>
      {size ? (
        <Sequence layout="none" width={size.width} height={size.height}>
          {children}
        </Sequence>
      ) : null}
    </div>
  );
};

export function fitScreen(Component: ElementComponent): ElementComponent {
  const Fitted: ElementComponent = ({ children, ...props }) => (
    <Component {...props}>{children ? <ScreenFit>{children}</ScreenFit> : undefined}</Component>
  );
  return Fitted;
}

export function placedAt(Component: ElementComponent): ElementComponent {
  const Placed: ElementComponent = ({ x, y, ...props }) => {
    const { width, height } = useVideoConfig();
    const left = numberOr(x, 0.5) * width;
    const top = numberOr(y, 0.5) * height;
    return (
      <AbsoluteFill>
        <div style={{ position: "absolute", left, top, translate: "-50% -50%" }}>
          <Component {...props} />
        </div>
      </AbsoluteFill>
    );
  };
  return Placed;
}

export function pointsFromSpots(Component: ElementComponent): ElementComponent {
  const Pointed: ElementComponent = ({ fromX, fromY, toX, toY, ...props }) => {
    const { width, height } = useVideoConfig();
    const point = (x: unknown, y: unknown) => ({
      x: numberOr(x, 0.5) * width,
      y: numberOr(y, 0.5) * height,
    });
    return (
      <AbsoluteFill>
        <Component {...props} from={point(fromX, fromY)} to={point(toX, toY)} />
      </AbsoluteFill>
    );
  };
  return Pointed;
}

const PAINT_RULE = ".df-painted > div { background: var(--df-paint) !important; }";

export const PaintRule: React.FC = () => <style>{PAINT_RULE}</style>;

export function painted(Component: ElementComponent): ElementComponent {
  const Painted: ElementComponent = ({ background, ...props }) => {
    const paint: React.CSSProperties & Record<"--df-paint", string> = {
      "--df-paint": isString(background) ? background : "transparent",
    };
    return (
      <AbsoluteFill className="df-painted" style={paint}>
        <Component {...props} />
      </AbsoluteFill>
    );
  };
  return Painted;
}

export function playsAtEnd(Component: ElementComponent): ElementComponent {
  const AtEnd: ElementComponent = (props) => {
    const { durationInFrames } = useVideoConfig();
    const length = numberOr(props.durationInFrames, 0);
    return <Component {...props} delay={Math.max(0, durationInFrames - length)} />;
  };
  return AtEnd;
}

function setPath(target: ElementProps, path: string[], value: unknown): void {
  const [head, ...rest] = path;
  if (!head) return;
  if (rest.length === 0) {
    target[head] = value;
    return;
  }
  const inner = isRecord(target[head]) ? { ...target[head] } : {};
  setPath(inner, rest, value);
  target[head] = inner;
}

export function nested(
  Component: ElementComponent,
  paths: Record<string, string>,
): ElementComponent {
  const Nested: ElementComponent = (props) => {
    const out: ElementProps & { children?: React.ReactNode } = {};
    for (const [key, value] of Object.entries(props))
      if (!Object.hasOwn(paths, key)) out[key] = value;
    for (const [key, path] of Object.entries(paths))
      if (props[key] !== undefined) setPath(out, path.split("."), props[key]);
    return <Component {...out} />;
  };
  return Nested;
}

const rowsOf = (value: unknown): ElementProps[] =>
  Array.isArray(value) ? value.filter(isRecord) : [];

export function listsFromRows(
  Component: ElementComponent,
  lists: Record<string, Record<string, string>>,
): ElementComponent {
  const Unzipped: ElementComponent = (props) => {
    const out: ElementProps & { children?: React.ReactNode } = { ...props };
    for (const [key, columns] of Object.entries(lists)) {
      const rows = rowsOf(props[key]);
      delete out[key];
      for (const [column, prop] of Object.entries(columns)) {
        const values = rows.map((r) => r[column]);
        if (values.some((v) => v !== undefined && v !== "")) out[prop] = values;
      }
    }
    return <Component {...out} />;
  };
  return Unzipped;
}

export function marksFromImages(
  Component: ElementComponent,
  { from, to }: { from: string; to: string },
): ElementComponent {
  const Marked: ElementComponent = (props) => {
    const marks = rowsOf(props[from]).map((row) => ({
      mark:
        isString(row.image) && row.image ? (
          <Img src={row.image} style={{ width: "100%", height: "100%", objectFit: "contain" }} />
        ) : null,
      ...(isString(row.background) && row.background ? { bg: row.background } : {}),
    }));
    const rest = Object.fromEntries(Object.entries(props).filter(([k]) => k !== from));
    return <Component {...rest} {...{ [to]: marks }} />;
  };
  return Marked;
}

const TICK_WORDS: Partial<Record<string, boolean>> = { yes: true, no: false };

export function ticksFromWords(Component: ElementComponent, key: string): ElementComponent {
  const Ticked: ElementComponent = (props) => {
    const rows = rowsOf(props[key]).map((row) => ({
      ...row,
      cells: Array.isArray(row.cells)
        ? row.cells.map((cell) =>
            isString(cell) ? (TICK_WORDS[cell.trim().toLowerCase()] ?? cell) : cell,
          )
        : row.cells,
    }));
    return <Component {...props} {...{ [key]: rows }} />;
  };
  return Ticked;
}

type TreeNode = { name: string; children?: TreeNode[] };

export function treeFromPaths(paths: readonly string[]): TreeNode[] {
  const root: TreeNode = { name: "", children: [] };
  for (const path of paths) {
    const parts = path.split("/").map((p) => p.trim());
    const folder = parts.at(-1) === "";
    const names = parts.filter(Boolean);
    let node = root;
    names.forEach((name, i) => {
      node.children ??= [];
      let child = node.children.find((c) => c.name === name);
      if (!child) {
        child = { name, ...(i < names.length - 1 || folder ? { children: [] } : {}) };
        node.children.push(child);
      } else if (i < names.length - 1) child.children ??= [];
      node = child;
    });
  }
  return root.children ?? [];
}

export function treeFrom(Component: ElementComponent, key: string): ElementComponent {
  const Tree: ElementComponent = (props) => {
    const paths = Array.isArray(props[key]) ? props[key].filter(isString) : [];
    return <Component {...props} {...{ [key]: treeFromPaths(paths) }} />;
  };
  return Tree;
}

const REMOCN_PRESETS = { light: defaultLightTheme, dark: defaultDarkTheme } as const;

export function themed(Component: ElementComponent, family: "context" | "prop"): ElementComponent {
  const Themed: ElementComponent = (props) => {
    const { theme } = props;
    if (!isThemeValue(theme)) return <Component {...props} />;
    if (family === "prop")
      return (
        <SnapCnUIProvider mode={theme.mode}>
          <Component {...props} theme={themeColors(theme)} mode={theme.mode} />
        </SnapCnUIProvider>
      );
    const tokens = { ...REMOCN_PRESETS[theme.mode], ...themeColors(theme) };
    return (
      <RemocnUIProvider mode={theme.mode} theme={tokens}>
        <Component {...props} theme={tokens} />
      </RemocnUIProvider>
    );
  };
  return Themed;
}
