import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { spring } from "@opsis/runtime/spring.ts";
import { fontOr } from "#video/common/fonts.ts";
import { CLAMPED } from "#video/core/anim.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { codePalette, type CodeTheme } from "#video/lib/code/code-syntax.tsx";
import { getSafeAreaPadding } from "#video/lib/layout/layout.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";
import { treeFromPaths } from "#video/lib/settings/shape.ts";

export type FileNode = {
  name: string;
  children?: FileNode[];
};

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

function extensionColor(name: string, palette: CodeTheme) {
  const extension = name.slice(name.lastIndexOf(".") + 1);
  if (extension === "tsx" || extension === "jsx") return palette.token.type;
  if (extension === "ts" || extension === "js" || extension === "mjs") return palette.token.call;
  if (extension === "json") return palette.token.number;
  if (extension === "css") return palette.token.prop;
  if (extension === "md" || extension === "mdx") return palette.token.string;
  if (extension === "svg" || extension === "png") return palette.token.keyword;
  return palette.fg;
}

function FolderGlyph({ size, color }: { size: number; color: string }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <path
        d="M3.4 6.6a1.8 1.8 0 0 1 1.8-1.8h3.6l2 2.4h8a1.8 1.8 0 0 1 1.8 1.8v8.4a1.8 1.8 0 0 1-1.8 1.8H5.2a1.8 1.8 0 0 1-1.8-1.8V6.6Z"
        stroke={color}
        strokeWidth={1.6}
        strokeLinejoin="round"
      />
    </svg>
  );
}

function FileGlyph({ size, color }: { size: number; color: string }) {
  return (
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
}

function ChevronGlyph({ size, color }: { size: number; color: string }) {
  return (
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
}

function FileTreeRevealView({
  nodes,
  title,
  selectedPath,
  rowStagger,
  holdSeconds,
  accentColor,
  backgroundColor,
  theme,
  speed,
  countLabel,
  panelWidth,
  fontFamily,
}: Omit<ElementSettings["file-tree-reveal"], "nodes"> & {
  nodes: FileNode[];
  holdSeconds?: number;
}) {
  const frame = useFrame();
  const { fps, width, height } = useFilm();
  const palette = codePalette(theme);
  const safe = getSafeAreaPadding({ width, height });

  function at(seconds: number) {
    return (seconds * fps) / speed;
  }
  function ease(from: number, to: number, easing = EASING.enter) {
    return interpolate(frame, [at(from), at(to)], [0, 1], { easing, ...CLAMPED });
  }

  const portrait = height > width;
  const rows = flatten(nodes);

  const panelUnits = HEADER_U + PAD_U + rows.length * ROW_U;
  const fitU = portrait
    ? Math.min(width / 560, height / 1000)
    : Math.min(width / 640, height / 360);
  const u = Math.min(fitU, (height - safe.paddingTop - safe.paddingBottom) / panelUnits);
  function revealAt(index: number) {
    return T.rows + rowStagger * index;
  }
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
          ...CLAMPED,
        });

  const panelW = Math.min(width - safe.paddingLeft - safe.paddingRight, panelWidth * u);
  const rowH = ROW_U * u;
  const indent = 19 * u;
  const rowsH = rows.length * rowH;

  return (
    <AbsoluteFill
      style={{
        background: backgroundColor ?? palette.page,
        fontFamily: fontOr(fontFamily, "main"),
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
              {rows.filter((row) => !row.isFolder).length} {countLabel}
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
}

export function FileTreeReveal({ nodes, ...settings }: ElementSettings["file-tree-reveal"]) {
  return <FileTreeRevealView {...settings} nodes={treeFromPaths(nodes)} />;
}
