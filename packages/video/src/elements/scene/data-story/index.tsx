import { TransitionSeries } from "@remotion/transitions";
import { AbsoluteFill, useVideoConfig } from "remotion";
import { MetricTicker, type MetricTickerItem } from "#video/elements/data/metric-ticker/index.tsx";
import { AnimatedBarChart } from "#video/elements/scene/data-story/animated-bar-chart/index.tsx";
import { AutoFitTitle } from "#video/elements/scene/data-story/auto-fit-title/index.tsx";
import { CaptionBumper } from "#video/elements/scene/data-story/caption-bumper/index.tsx";
import {
  TimelineSteps,
  type TimelineStep,
} from "#video/elements/scene/data-story/timeline-steps/index.tsx";
import { transitionFade } from "#video/elements/scene/data-story/transition-fade.tsx";
import { EndCard } from "#video/elements/scene/end-card/index.tsx";
import type { ChartDatum } from "#video/lib/charts/chart-utils.ts";
import { DURATION } from "#video/lib/motion/motion-tokens.ts";

const COLORS = {
  bg: "#080810",
  hookBg: "#0c0c14",
  hookAccent: "#e8b86d",
  chartBg: "#080810",
  chartBar: "#2dd4bf",
  tickerBg: "#0a1014",
  tickerAccent: "#2dd4bf",
  timelineBg: "#0c0c14",
  timelineAccent: "#f59e0b",
  insightBg: "#080810",
  insightAccent: "#2dd4bf",
  endBg: "#080810",
  endAccent: "#e8b86d",
} as const;

const SCENE_DURATIONS = {
  hook: 84,
  chart: 92,
  metrics: 88,
  timeline: 88,
  insight: 66,
  end: 62,
} as const;

const FADE = transitionFade({ durationInFrames: DURATION.fast });

const SCENE_COUNT = Object.keys(SCENE_DURATIONS).length;
const OVERLAP = (SCENE_COUNT - 1) * DURATION.fast;
const AUTHORED_TOTAL =
  Object.values(SCENE_DURATIONS).reduce((sum, frames) => sum + frames, 0) - OVERLAP;
const HOOK_EXIT_LEAD = 26;

function sceneLengths(total: number) {
  const factor = (total + OVERLAP) / (AUTHORED_TOTAL + OVERLAP);
  const scaled = (frames: number) => Math.max(2 * DURATION.fast + 1, Math.round(frames * factor));
  return {
    hook: scaled(SCENE_DURATIONS.hook),
    chart: scaled(SCENE_DURATIONS.chart),
    metrics: scaled(SCENE_DURATIONS.metrics),
    timeline: scaled(SCENE_DURATIONS.timeline),
    insight: scaled(SCENE_DURATIONS.insight),
    end: scaled(SCENE_DURATIONS.end),
  };
}

export type DataStoryProps = {
  title?: string;
  subtitle?: string;
  barData: ChartDatum[];
  metrics: MetricTickerItem[];
  steps: TimelineStep[];
  chartTitle?: string;
  metricsTitle?: string;
  timelineTitle?: string;
  insight?: string;
  insightEyebrow?: string;
  ctaTitle?: string;
  ctaLabel?: string;
  theme?: "dark" | "light";
  backgroundColor?: string;
  accentColor?: string;
  dataColor?: string;
  timelineColor?: string;
};

export const DataStory: React.FC<DataStoryProps> = ({
  title = "Quarterly reach by channel",
  subtitle = "Short-form climbed while long-form held attention",
  barData,
  metrics,
  steps,
  chartTitle = "Views by format",
  metricsTitle = "Signals that mattered",
  timelineTitle = "How we read the quarter",
  insight = "Short-form drove reach; long-form kept attention.",
  insightEyebrow = "Takeaway",
  ctaTitle = "Turn your data into motion",
  ctaLabel,
  theme = "dark",
  backgroundColor,
  accentColor = COLORS.hookAccent,
  dataColor = COLORS.chartBar,
  timelineColor = COLORS.timelineAccent,
}) => {
  const { durationInFrames, fps } = useVideoConfig();
  const scenes = sceneLengths(durationInFrames);
  const dark = theme === "dark";
  const sceneBg = (darkTint: string) => backgroundColor ?? (dark ? darkTint : undefined);
  return (
    <AbsoluteFill style={{ backgroundColor: sceneBg(COLORS.bg) }}>
      <TransitionSeries>
        <TransitionSeries.Sequence durationInFrames={scenes.hook}>
          <AutoFitTitle
            title={title}
            subtitle={subtitle}
            holdSeconds={(scenes.hook - HOOK_EXIT_LEAD) / fps}
            maxFontSize={88}
            theme={theme}
            accentColor={accentColor}
            backgroundColor={sceneBg(COLORS.hookBg)}
          />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition {...FADE} />
        <TransitionSeries.Sequence durationInFrames={scenes.chart}>
          <AnimatedBarChart
            title={chartTitle}
            data={barData}
            theme={theme}
            backgroundColor={sceneBg(COLORS.chartBg)}
            barColor={dataColor}
          />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition {...FADE} />
        <TransitionSeries.Sequence durationInFrames={scenes.metrics}>
          <MetricTicker
            title={metricsTitle}
            metrics={metrics}
            theme={theme}
            backgroundColor={sceneBg(COLORS.tickerBg)}
            accentColor={dataColor}
          />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition {...FADE} />
        <TransitionSeries.Sequence durationInFrames={scenes.timeline}>
          <TimelineSteps
            speed={1.5}
            title={timelineTitle}
            steps={steps}
            theme={theme}
            backgroundColor={sceneBg(COLORS.timelineBg)}
            accentColor={timelineColor}
          />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition {...FADE} />
        <TransitionSeries.Sequence durationInFrames={scenes.insight}>
          <CaptionBumper
            text={insight}
            eyebrow={insightEyebrow}
            theme={theme}
            backgroundColor={sceneBg(COLORS.insightBg)}
            accentColor={dataColor}
          />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition {...FADE} />
        <TransitionSeries.Sequence durationInFrames={scenes.end}>
          <EndCard
            speed={1.4}
            title={ctaTitle}
            cta={ctaLabel}
            theme={theme}
            backgroundColor={sceneBg(COLORS.endBg)}
            accentColor={accentColor}
          />
        </TransitionSeries.Sequence>
      </TransitionSeries>
    </AbsoluteFill>
  );
};
