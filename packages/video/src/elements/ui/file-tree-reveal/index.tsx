import type React from "react";
import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { isString } from "#video/common/guards.ts";
import { type ElementProps } from "#video/common/spec.ts";
import { CODE_THEMES } from "#video/lib/code/code-syntax.tsx";
import { getSafeAreaPadding } from "#video/lib/layout/layout.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";
import { treeFromPaths } from "#video/lib/settings/shape.ts";

const fontFamily = '"Inter"';

export type FileNode = {
  name: string;
  children?: FileNode[];
};

export type FileTreeRevealProps = {
  nodes?: FileNode[];
  title?: string;
  selectedPath?: string;
  rowStagger?: number;
  holdSeconds?: number;
  accentColor?: string;
  backgroundColor?: string;
  theme?: "dark" | "light";
  speed?: number;
};

const DEFAULT_NODES: FileNode[] = [
  {
    name: "src",
    children: [
      {
        name: "scenes",
        children: [{ name: "hook-card.tsx" }, { name: "lower-third.tsx" }],
      },
      { name: "lib", children: [{ name: "motion-tokens.ts" }] },
      { name: "index.ts" },
    ],
  },
  { name: "public", children: [{ name: "logo.svg" }] },
  { name: "remotion.config.ts" },
  { name: "package.json" },
];

const T = {
  panel: 0,
  panelFor: 0.46,
  header: 0.16,
  rows: 0.42,
  rowFor: 0.34,
  beforeSelect: 0.24,
  selectFor: 0.4,
  exitFor: 0.42,
} as const;

const clamp = {
  extrapolateLeft: "clamp",
  extrapolateRight: "clamp",
} as const;

const HEADER_U = 42;
const ROW_U = 31;
const PAD_U = 24;

type FlatRow = {
  name: string;
  path: string;
  depth: number;
  isFolder: boolean;
  firstChildIndex: number | null;
};

function flatten(nodes: FileNode[], depth = 0, parent = "", out: FlatRow[] = []) {
  for (const node of nodes) {
    const path = parent ? `${parent}/${node.name}` : node.name;
    const row: FlatRow = {
      name: node.name,
      path,
      depth,
      isFolder: node.children !== undefined,
      firstChildIndex: null,
    };
    out.push(row);
    if (node.children && node.children.length > 0) {
      row.firstChildIndex = out.length;
      flatten(node.children, depth + 1, path, out);
    }
  }
  return out;
}

function extensionColor(name: string, palette: (typeof CODE_THEMES)["dark"]) {
  const extension = name.slice(name.lastIndexOf(".") + 1);
  if (extension === "tsx" || extension === "jsx") return palette.token.type;
  if (extension === "ts" || extension === "js" || extension === "mjs") return palette.token.call;
  if (extension === "json") return palette.token.number;
  if (extension === "css") return palette.token.prop;
  if (extension === "md" || extension === "mdx") return palette.token.string;
  if (extension === "svg" || extension === "png") return palette.token.keyword;
  return palette.fg;
}

const FolderGlyph: React.FC<{ size: number; color: string }> = ({ size, color }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
    <path
      d="M3.4 6.6a1.8 1.8 0 0 1 1.8-1.8h3.6l2 2.4h8a1.8 1.8 0 0 1 1.8 1.8v8.4a1.8 1.8 0 0 1-1.8 1.8H5.2a1.8 1.8 0 0 1-1.8-1.8V6.6Z"
      stroke={color}
      strokeWidth={1.6}
      strokeLinejoin="round"
    />
  </svg>
);

const FileGlyph: React.FC<{ size: number; color: string }> = ({ size, color }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
    <path
      d="M13.6 3.4H7a1.8 1.8 0 0 0-1.8 1.8v13.6A1.8 1.8 0 0 0 7 20.6h10a1.8 1.8 0 0 0 1.8-1.8V8.6l-5.2-5.2Z"
      stroke={color}
      strokeWidth={1.6}
      strokeLinejoin="round"
    />
    <path d="M13.4 3.6v5.2h5.2" stroke={color} strokeWidth={1.6} />
  </svg>
);

const ChevronGlyph: React.FC<{ size: number; color: string }> = ({ size, color }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
    <path
      d="M9 6.5L15.5 12 9 17.5"
      stroke={color}
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
    />
  </svg>
);

const FileTreeRevealView: React.FC<FileTreeRevealProps> = ({
  nodes = DEFAULT_NODES,
  title = "northstar-studio",
  selectedPath = "src/scenes/lower-third.tsx",
  rowStagger = 0.18,
  holdSeconds,
  accentColor = "#E8B86D",
  backgroundColor,
  theme = "dark",
  speed = 1,
}) => {
  const frame = useCurrentFrame();
  const { fps, width, height } = useVideoConfig();
  const palette = CODE_THEMES[theme];
  const safe = getSafeAreaPadding({ width, height });

  const at = (seconds: number) => (seconds * fps) / speed;
  const ease = (from: number, to: number, easing = EASING.enter) =>
    interpolate(frame, [at(from), at(to)], [0, 1], { easing, ...clamp });

  const portrait = height > width;
  const rows = flatten(nodes);

  const panelUnits = HEADER_U + PAD_U + rows.length * ROW_U;
  const fitU = portrait
    ? Math.min(width / 560, height / 1000)
    : Math.min(width / 640, height / 360);
  const u = Math.min(fitU, (height - safe.paddingTop - safe.paddingBottom) / panelUnits);
  const revealAt = (index: number) => T.rows + rowStagger * index;
  const lastRowEnd = rows.length > 0 ? revealAt(rows.length - 1) + T.rowFor : T.rows;
  const selectAt = lastRowEnd + T.beforeSelect;
  const select = selectedPath ? ease(selectAt, selectAt + T.selectFor, EASING.editorial) : 0;
  const wantedPath = (selectedPath ?? "")
    .trim()
    .replace(/\\/g, "/")
    .replace(/^\/+|\/+$/g, "")
    .toLowerCase();

  const panel = spring({
    frame: frame - at(T.panel),
    fps,
    config: { damping: 18, stiffness: 130, mass: 0.85 },
  });
  const open = ease(T.panel, T.panel + T.panelFor, EASING.editorial);
  const headerIn = ease(T.header, T.header + 0.4);

  const exit =
    holdSeconds === undefined
      ? 0
      : interpolate(frame, [at(holdSeconds), at(holdSeconds + T.exitFor)], [0, 1], {
          easing: EASING.exit,
          ...clamp,
        });

  const panelW = Math.min(width - safe.paddingLeft - safe.paddingRight, 480 * u);
  const rowH = ROW_U * u;
  const indent = 19 * u;
  const rowsH = rows.length * rowH;

  return (
    <AbsoluteFill
      style={{
        background: backgroundColor ?? palette.page,
        fontFamily,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: `${safe.paddingTop}px ${safe.paddingRight}px`,
      }}
    >
      <div
        style={{
          width: panelW,
          borderRadius: 18 * u,
          background: palette.window,
          border: `1px solid ${palette.border}`,
          boxShadow: `inset 0 1px 0 ${palette.highlight}, 0 ${24 * u}px ${
            60 * u
          }px ${palette.shadow}`,
          overflow: "hidden",
          clipPath: `inset(0 0 ${(1 - open) * 100}% 0 round ${18 * u}px)`,
          opacity: 1 - exit,
          translate: `0 ${(1 - panel) * 22 * u + exit * 30 * u}px`,
        }}
      >
        {title ? (
          <div
            style={{
              height: HEADER_U * u,
              display: "flex",
              alignItems: "center",
              gap: 9 * u,
              padding: `0 ${16 * u}px`,
              background: palette.header,
              borderBottom: `1px solid ${palette.border}`,
              opacity: headerIn,
            }}
          >
            <FolderGlyph size={17 * u} color={accentColor} />
            <span
              style={{
                color: palette.fg,
                fontSize: 15 * u,
                fontWeight: 600,
                letterSpacing: "0.01em",
              }}
            >
              {title}
            </span>
            <span
              style={{
                marginLeft: "auto",
                color: palette.faint,
                fontSize: 13 * u,
                fontWeight: 500,
                fontVariantNumeric: "tabular-nums",
              }}
            >
              {rows.filter((row) => !row.isFolder).length} files
            </span>
          </div>
        ) : null}

        <div
          style={{
            padding: `${10 * u}px ${12 * u}px ${14 * u}px`,
            minHeight: rowsH,
            display: "flex",
            flexDirection: "column",
          }}
        >
          {rows.map((row, index) => {
            const rowIn = ease(revealAt(index), revealAt(index) + T.rowFor);
            const childAt = row.firstChildIndex === null ? null : revealAt(row.firstChildIndex);
            const turn = childAt === null ? 0 : ease(childAt - 0.08, childAt + 0.22);
            const selected = wantedPath !== "" && row.path.toLowerCase() === wantedPath;
            const lit = selected ? select : 0;
            const nameColor = row.isFolder ? palette.fg : extensionColor(row.name, palette);

            return (
              <div
                key={row.path}
                style={{
                  position: "relative",
                  height: rowH * rowIn,
                  display: "flex",
                  alignItems: "center",
                  gap: 8 * u,
                  paddingLeft: 8 * u + row.depth * indent,
                  paddingRight: 10 * u,
                  borderRadius: 8 * u,
                  background: selected
                    ? `color-mix(in srgb, ${accentColor} ${lit > 0.4 ? 11 : 0}%, transparent)`
                    : "none",
                  opacity: rowIn,
                  overflow: "hidden",
                }}
              >
                {selected ? (
                  <span
                    style={{
                      position: "absolute",
                      left: 0,
                      top: 4 * u,
                      bottom: 4 * u,
                      width: 2.5 * u,
                      borderRadius: 999,
                      background: accentColor,
                      opacity: lit,
                      scale: `1 ${interpolate(lit, [0, 1], [0.3, 1])}`,
                    }}
                  />
                ) : null}

                <span
                  style={{
                    display: "flex",
                    width: 13 * u,
                    justifyContent: "center",
                    opacity: row.isFolder ? 1 : 0,
                    rotate: `${turn * 90}deg`,
                  }}
                >
                  <ChevronGlyph size={13 * u} color={palette.faint} />
                </span>

                {row.isFolder ? (
                  <FolderGlyph size={16 * u} color={palette.dim} />
                ) : (
                  <FileGlyph
                    size={16 * u}
                    color={`color-mix(in srgb, ${nameColor} 70%, transparent)`}
                  />
                )}

                <span
                  style={{
                    color: selected && lit > 0.4 ? accentColor : nameColor,
                    fontSize: 16 * u,
                    fontWeight: row.isFolder ? 600 : 500,
                    whiteSpace: "nowrap",
                    translate: `${(1 - rowIn) * -14 * u}px`,
                  }}
                >
                  {row.name}
                </span>
              </div>
            );
          })}
        </div>
      </div>
    </AbsoluteFill>
  );
};

export function FileTreeReveal(props: ElementProps & { children?: React.ReactNode }) {
  return (
    <FileTreeRevealView
      {...({
        ...props,
        nodes: treeFromPaths(Array.isArray(props.nodes) ? props.nodes.filter(isString) : []),
      } as unknown as React.ComponentProps<typeof FileTreeRevealView>)}
    />
  );
}
