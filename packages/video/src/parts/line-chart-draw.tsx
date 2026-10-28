import { getLength, getPointAtLength } from "@remotion/paths";
import { useId } from "react";
import { interpolate, useCurrentFrame, useVideoConfig } from "remotion";
import {
  buildAreaPath,
  buildLinePath,
  buildSmoothPath,
  formatAxisValue,
  getPlotArea,
  niceDomain,
  plotPoints,
  type ChartPoint,
} from "#video/lib/charts/chart-utils.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";
import { getPathDrawStyles } from "#video/lib/paths/path-utils.ts";

export type LineChartDrawProps = {
  points: ChartPoint[];
  width?: number;
  height?: number;
  color?: string;
  strokeWidth?: number;
  variant?: "smooth" | "linear";
  showAxis?: boolean;
  showXLabels?: boolean;
  showArea?: boolean;
  showDots?: boolean;
  showHead?: boolean;
  showEndLabel?: boolean;
  includeZero?: boolean;
  valueFormatter?: (value: number) => string;
  labelColor?: string;
  gridColor?: string;
  surfaceColor?: string;
  durationInFrames?: number;
  delayInFrames?: number;
  frame?: number;
};

const clamp01 = (value: number) => Math.min(1, Math.max(0, value));

export const LineChartDraw: React.FC<LineChartDrawProps> = ({
  points,
  width: widthProp,
  height: heightProp,
  color = "#e8b86d",
  strokeWidth: strokeWidthProp,
  variant = "smooth",
  showAxis = true,
  showXLabels = true,
  showArea = true,
  showDots = true,
  showHead = true,
  showEndLabel = false,
  includeZero = true,
  valueFormatter = formatAxisValue,
  labelColor = "rgba(250,250,250,0.52)",
  gridColor = "rgba(250,250,250,0.10)",
  surfaceColor = "#080810",
  durationInFrames = 70,
  delayInFrames = 0,
  frame: frameOverride,
}) => {
  const localFrame = useCurrentFrame();
  const frame = frameOverride ?? localFrame;
  const config = useVideoConfig();
  const width = widthProp ?? config.width;
  const height = heightProp ?? Math.round(config.height * 0.4);

  const unit = width / 960;
  const scale = (value: number) => Math.max(1, Math.round(value * unit));
  const labelSize = scale(22);
  const strokeWidth = strokeWidthProp ?? scale(5);

  const domain = niceDomain(
    points.map((point) => point.y),
    { includeZero, tickCount: 4 },
  );
  const axisLabels = domain.ticks.map((tick) => valueFormatter(tick));
  const longestLabel = axisLabels.reduce((longest, label) => Math.max(longest, label.length), 0);
  const gutter = showAxis ? Math.round(longestLabel * labelSize * 0.62) + scale(18) : scale(6);

  const plot = getPlotArea(width, height, {
    top: scale(showEndLabel ? 34 : 16),
    right: scale(showEndLabel ? 44 : 12),
    bottom: showXLabels ? labelSize + scale(20) : scale(10),
    left: gutter,
  });

  const plotted = plotPoints(points, plot, domain);
  const linePath = variant === "smooth" ? buildSmoothPath(plotted) : buildLinePath(plotted);
  const areaPath = buildAreaPath(linePath, plotted, plot.bottom);

  const progress = interpolate(frame, [delayInFrames, delayInFrames + durationInFrames], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: EASING.enter,
  });

  const pathLength = linePath === "" ? 0 : getLength(linePath);
  const fallbackTip = { x: plot.left, y: plot.bottom };
  const tip =
    (pathLength > 0 ? getPointAtLength(linePath, pathLength * progress) : null) ?? fallbackTip;

  const uid = useId().replace(/:/g, "");
  const gradientId = `${uid}-fill`;
  const clipId = `${uid}-clip`;

  const endPoint = plotted[plotted.length - 1];
  const endLabelOpacity = interpolate(progress, [0.88, 1], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <svg
      width={width}
      height={height}
      viewBox={`0 0 ${width} ${height}`}
      style={{ overflow: "visible" }}
    >
      <defs>
        <linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor={color} stopOpacity={0.32} />
          <stop offset="100%" stopColor={color} stopOpacity={0} />
        </linearGradient>
        <clipPath id={clipId}>
          <rect
            x={plot.left}
            y={plot.top - strokeWidth}
            width={plot.width * progress}
            height={plot.height + strokeWidth * 2}
          />
        </clipPath>
      </defs>

      {showAxis
        ? domain.ticks.map((tick, index) => {
            const y = plot.bottom - ((tick - domain.min) / domain.span) * plot.height;

            return (
              <g key={tick}>
                <line
                  x1={plot.left}
                  y1={y}
                  x2={plot.right}
                  y2={y}
                  stroke={gridColor}
                  strokeWidth={1}
                  opacity={index === 0 ? 1.8 : 1}
                />
                <text
                  x={plot.left - scale(12)}
                  y={y}
                  fill={labelColor}
                  fontSize={labelSize}
                  fontWeight={600}
                  textAnchor="end"
                  dominantBaseline="central"
                  style={{ fontVariantNumeric: "tabular-nums" }}
                >
                  {axisLabels[index]}
                </text>
              </g>
            );
          })
        : null}

      {showArea && areaPath !== "" ? (
        <path d={areaPath} fill={`url(#${gradientId})`} clipPath={`url(#${clipId})`} />
      ) : null}

      <path
        d={linePath}
        fill="none"
        stroke={color}
        strokeWidth={strokeWidth}
        strokeLinecap="round"
        strokeLinejoin="round"
        style={getPathDrawStyles(progress, linePath)}
      />

      {showXLabels
        ? plotted.map((point, index) => (
            <text
              key={`${point.x}-label`}
              x={point.x}
              y={plot.bottom + scale(14) + labelSize / 2}
              fill={labelColor}
              fontSize={labelSize}
              fontWeight={600}
              textAnchor={index === 0 ? "start" : index === plotted.length - 1 ? "end" : "middle"}
              dominantBaseline="central"
            >
              {point.label ?? ""}
            </text>
          ))
        : null}

      {showDots
        ? plotted.map((point) => {
            const window = Math.max(1, plot.width * 0.04);
            const reveal = clamp01((tip.x - point.x + window) / window);

            return (
              <circle
                key={`${point.x}-dot`}
                cx={point.x}
                cy={point.y}
                r={strokeWidth * 0.95 * reveal}
                fill={color}
                stroke={surfaceColor}
                strokeWidth={strokeWidth * 0.5 * reveal}
              />
            );
          })
        : null}

      {showHead && progress > 0 && progress < 1 ? (
        <g>
          <circle cx={tip.x} cy={tip.y} r={strokeWidth * 2.6} fill={color} opacity={0.18} />
          <circle cx={tip.x} cy={tip.y} r={strokeWidth * 1.05} fill={color} />
        </g>
      ) : null}

      {showEndLabel && endPoint ? (
        <text
          x={endPoint.x}
          y={endPoint.y - scale(22)}
          fill={color}
          fontSize={scale(28)}
          fontWeight={700}
          textAnchor="end"
          dominantBaseline="central"
          opacity={endLabelOpacity}
          style={{ fontVariantNumeric: "tabular-nums" }}
        >
          {valueFormatter(endPoint.value)}
        </text>
      ) : null}
    </svg>
  );
};
