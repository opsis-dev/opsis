import { render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { ElementProps } from "#video/common/spec.ts";
import { TestFrame } from "#video/core/test-frame.tsx";
import { BarChartRace } from "#video/elements/data/bar-chart-race.tsx";
import { ComparisonBars } from "#video/elements/data/comparison-bars.tsx";
import { DonutChart } from "#video/elements/data/donut-chart.tsx";
import { FunnelChart } from "#video/elements/data/funnel-chart.tsx";
import { MetricTicker } from "#video/elements/data/metric-ticker/index.tsx";
import { ChangelogEntry } from "#video/elements/scene/changelog-entry/index.tsx";
import { ComparisonTable } from "#video/elements/scene/comparison-table/index.tsx";
import { EndCard } from "#video/elements/scene/end-card/index.tsx";
import { LogoWall } from "#video/elements/scene/logo-wall/index.tsx";
import { PricingCard } from "#video/elements/scene/pricing-card/index.tsx";
import { CommitGraph } from "#video/elements/ui/commit-graph/index.tsx";
import { KanbanMove } from "#video/elements/ui/kanban-move/index.tsx";
import DEFAULT_PROPS from "#video/registry/default-props.generated.json";
import type { ElementComponent } from "#video/registry/types.ts";

const ROW_ELEMENTS: [string, ElementComponent][] = [
  ["bar-chart-race", BarChartRace as ElementComponent],
  ["changelog-entry", ChangelogEntry as ElementComponent],
  ["commit-graph", CommitGraph as ElementComponent],
  ["comparison-bars", ComparisonBars as ElementComponent],
  ["comparison-table", ComparisonTable as ElementComponent],
  ["donut-chart", DonutChart as ElementComponent],
  ["end-card", EndCard as ElementComponent],
  ["funnel-chart", FunnelChart as ElementComponent],
  ["kanban-move", KanbanMove as ElementComponent],
  ["logo-wall", LogoWall as ElementComponent],
  ["metric-ticker", MetricTicker as ElementComponent],
  ["pricing-card", PricingCard as ElementComponent],
];

function withRepeatedRow(key: string): ElementProps {
  const defaults: Record<string, ElementProps> = DEFAULT_PROPS;
  const known = defaults[key];
  if (!known) throw new Error(key);
  const props: ElementProps = { ...known };
  for (const [name, value] of Object.entries(props))
    if (Array.isArray(value) && value.length > 0) props[name] = [...value, value[0]];
  if (key === "comparison-bars") props.afterColor = props.beforeColor;
  return props;
}

afterEach(() => {
  vi.restoreAllMocks();
});

it.each(ROW_ELEMENTS)("%s draws a repeated row as a row of its own", (key, Component) => {
  const errors: string[] = [];
  vi.spyOn(console, "error").mockImplementation((...args: unknown[]) => {
    errors.push(args.map(String).join(" "));
  });
  const props = withRepeatedRow(key);
  const frame = (at: number) => (
    <TestFrame
      component={Component}
      inputProps={props}
      durationInFrames={240}
      fps={30}
      compositionWidth={1920}
      compositionHeight={1080}
      frameToDisplay={at}
    />
  );
  const view = render(frame(60));
  for (const at of [90, 150, 230]) view.rerender(frame(at));
  expect(errors.filter((error) => /same key/.test(error))).toEqual([]);
});
