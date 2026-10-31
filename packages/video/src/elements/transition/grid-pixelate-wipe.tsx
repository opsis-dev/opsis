import { useFilm } from "@opsis/runtime/clock.tsx";
import { Easing } from "@opsis/runtime/easing.ts";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { useMemo } from "react";
import { CLAMPED } from "#video/core/anim.ts";
import type { ElementSettings, PixelateOrder, PixelShape } from "#video/generated/spec.ts";
import type { TransitionPresentation } from "#video/lib/transitions/presentation.ts";
import { transitionPhase } from "#video/lib/transitions/transition-timing.ts";

type GridPixelateWipeProps = ElementSettings["grid-pixelate-wipe"];

type GridMask = {
  cols: number;
  rows: number;
  order: PixelateOrder;
  shape: PixelShape;
  stagger: number;
  reverse: boolean;
  aspect: number;
};

function cellRank(
  order: PixelateOrder,
  col: number,
  row: number,
  cols: number,
  rows: number,
): number {
  const u = cols <= 1 ? 0 : col / (cols - 1);
  const v = rows <= 1 ? 0 : row / (rows - 1);

  switch (order) {
    case "from-left":
      return u;
    case "from-top":
      return v;
    case "diagonal":
      return (u + v) / 2;
    case "center":
      return Math.min(1, Math.hypot(u - 0.5, v - 0.5) / Math.SQRT1_2);
  }
}

function cellOpacity(progress: number, rank: number, stagger: number): number {
  const revealAt = rank * stagger;
  const span = Math.max(1 - stagger, 0.001);
  const raw = Math.min(1, Math.max(0, (progress - revealAt) / span));

  return interpolate(raw, [0, 1], [0, 1], { ...CLAMPED, easing: Easing.out(Easing.cubic) });
}

function buildGridMask(
  progress: number,
  { cols, rows, order, shape, stagger, reverse, aspect }: GridMask,
): string {
  const cellW = 100 / cols;
  const cellH = 100 / rows;
  const cells: string[] = [];

  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      const rank = cellRank(order, col, row, cols, rows);
      const opacity = cellOpacity(progress, reverse ? 1 - rank : rank, stagger);

      if (opacity < 0.004) {
        continue;
      }

      const alpha = opacity.toFixed(3);

      if (shape === "dot") {
        const r = (Math.hypot(cellW * aspect, cellH) / 2) * opacity;
        cells.push(
          `<ellipse cx="${(col + 0.5) * cellW}" cy="${(row + 0.5) * cellH}" rx="${(r / aspect).toFixed(3)}" ry="${r.toFixed(3)}" fill="white" fill-opacity="${alpha}"/>`,
        );
        continue;
      }

      cells.push(
        `<rect x="${col * cellW}" y="${row * cellH}" width="${cellW}" height="${cellH}" fill="white" fill-opacity="${alpha}"/>`,
      );
    }
  }

  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" preserveAspectRatio="none">${cells.join("")}</svg>`;
  return `url("data:image/svg+xml,${encodeURIComponent(svg)}")`;
}

function GridPixelateWipePresentation({
  children,
  presentationProgress,
  presentationDirection,
  passedProps: { cols, rows, order, shape, stagger, reverse },
}: React.ComponentProps<NonNullable<TransitionPresentation<GridPixelateWipeProps>["component"]>>) {
  const phase = transitionPhase(presentationProgress, presentationDirection, {
    lead: 1,
  });
  const filled = 1 - phase.displace;
  const { width, height } = useFilm();
  const shapedRows = Math.max(1, Math.round((rows * (height / width) * 16) / 9));

  const style = useMemo(() => {
    if (!phase.isEntering || filled >= 1) {
      return undefined;
    }

    const mask = buildGridMask(filled, {
      cols,
      rows: shapedRows,
      order,
      shape,
      stagger,
      reverse,
      aspect: width / height,
    });

    return {
      WebkitMaskImage: mask,
      maskImage: mask,
      WebkitMaskSize: "100% 100%",
      maskSize: "100% 100%",
      WebkitMaskRepeat: "no-repeat",
      maskRepeat: "no-repeat",
    };
  }, [cols, filled, order, phase.isEntering, reverse, shapedRows, shape, stagger, width, height]);

  return <AbsoluteFill style={style}>{children}</AbsoluteFill>;
}

export function gridPixelateWipe(
  props: GridPixelateWipeProps,
): TransitionPresentation<GridPixelateWipeProps> {
  return {
    component: GridPixelateWipePresentation,
    props,
  };
}
