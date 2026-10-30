import { TransitionSeries } from "@remotion/transitions";
import { AbsoluteFill, useVideoConfig } from "remotion";
import { MetricTicker } from "#video/elements/data/metric-ticker/index.tsx";
import { AnimatedBarChart } from "#video/elements/scene/data-story/animated-bar-chart/index.tsx";
import { AutoFitTitle } from "#video/elements/scene/data-story/auto-fit-title/index.tsx";
import { CaptionBumper } from "#video/elements/scene/data-story/caption-bumper/index.tsx";
import { storyLook } from "#video/elements/scene/data-story/look.ts";
import { TimelineSteps } from "#video/elements/scene/data-story/timeline-steps/index.tsx";
import { transitionFade } from "#video/elements/scene/data-story/transition-fade.tsx";
import { EndCard } from "#video/elements/scene/end-card/index.tsx";
import type { ElementSettings } from "#video/generated/spec.ts";
import { DURATION } from "#video/lib/motion/motion-tokens.ts";

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
  function scaled(frames: number) {
    return Math.max(2 * DURATION.fast + 1, Math.round(frames * factor));
  }
  return {
    hook: scaled(SCENE_DURATIONS.hook),
    chart: scaled(SCENE_DURATIONS.chart),
    metrics: scaled(SCENE_DURATIONS.metrics),
    timeline: scaled(SCENE_DURATIONS.timeline),
    insight: scaled(SCENE_DURATIONS.insight),
    end: scaled(SCENE_DURATIONS.end),
  };
}

export function DataStory(settings: ElementSettings["data-story"]) {
  const {
    title,
    subtitle,
    barData,
    metrics,
    steps,
    chartTitle,
    metricsTitle,
    timelineTitle,
    insight,
    insightEyebrow,
    ctaTitle,
    ctaLabel,
    theme,
    backgroundColor,
    accentColor,
    dataColor,
    timelineColor,
  } = settings;
  const { durationInFrames, fps } = useVideoConfig();
  const scenes = sceneLengths(durationInFrames);
  const look = storyLook(settings);
  function sceneBg(themed: string) {
    return backgroundColor || themed;
  }
  return (
    <AbsoluteFill style={{ backgroundColor: sceneBg(look.palette.page) }}>
      <TransitionSeries>
        <TransitionSeries.Sequence durationInFrames={scenes.hook}>
          <AutoFitTitle
            title={title}
            subtitle={subtitle}
            logoSrc={settings.logoSrc || undefined}
            holdSeconds={(scenes.hook - HOOK_EXIT_LEAD) / fps}
            maxFontSize={88}
            look={look}
            accentColor={accentColor}
            backgroundColor={sceneBg(look.palette.hookBg)}
          />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition {...FADE} />
        <TransitionSeries.Sequence durationInFrames={scenes.chart}>
          <AnimatedBarChart
            title={chartTitle}
            subtitle={settings.chartSubtitle || undefined}
            data={barData}
            look={look}
            backgroundColor={sceneBg(look.palette.page)}
            barColor={dataColor}
            accentColor={accentColor}
            highlightLabel={settings.highlightBar || undefined}
            showAxis={settings.showScale}
            numberFormat={settings.numberFormat}
          />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition {...FADE} />
        <TransitionSeries.Sequence durationInFrames={scenes.metrics}>
          <MetricTicker
            title={metricsTitle}
            eyebrow=""
            metrics={metrics}
            maxCards={4}
            theme={theme}
            backgroundColor={sceneBg(look.palette.tickerBg)}
            accentColor={dataColor}
            showArrows
            numberFormat={settings.numberFormat}
            glow={7 * look.glow}
            uppercaseEyebrow={look.eyebrowCaps}
            cornerRadius={18}
            trendFill
            fontFamily={settings.fontFamily}
          />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition {...FADE} />
        <TransitionSeries.Sequence durationInFrames={scenes.timeline}>
          <TimelineSteps
            speed={1.5}
            title={timelineTitle}
            steps={steps}
            look={look}
            backgroundColor={sceneBg(look.palette.timelineBg)}
            accentColor={timelineColor}
          />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition {...FADE} />
        <TransitionSeries.Sequence durationInFrames={scenes.insight}>
          <CaptionBumper
            text={insight}
            eyebrow={insightEyebrow}
            maxFontSize={84}
            look={look}
            backgroundColor={sceneBg(look.palette.page)}
            accentColor={dataColor}
          />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition {...FADE} />
        <TransitionSeries.Sequence durationInFrames={scenes.end}>
          <EndCard
            speed={1.4}
            title={ctaTitle}
            subtitle={settings.ctaSubtitle}
            eyebrow=""
            cta={ctaLabel}
            url={settings.ctaUrl}
            handles={[]}
            logoSrc=""
            theme={theme}
            backgroundColor={sceneBg(look.palette.page)}
            accentColor={accentColor}
            fontFamily={settings.fontFamily}
            glow={Math.round(12 * look.glow)}
            textSize={look.textSize}
            titleWeight={700}
            eyebrowCaps={look.eyebrowCaps}
            buttonRadius={999}
            buttonArrow
          />
        </TransitionSeries.Sequence>
      </TransitionSeries>
    </AbsoluteFill>
  );
}
