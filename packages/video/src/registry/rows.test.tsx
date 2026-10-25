import { ELEMENTS } from "@opsis/core/elements";
import type { ElementProps } from "@opsis/core/elements/types";
import { Thumbnail } from "@remotion/player";
import { render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { BarChartRace } from "../elements/remotion-ui/bar-chart-race";
import { ChangelogEntry } from "../elements/remotion-ui/changelog-entry";
import { CommitGraph } from "../elements/remotion-ui/commit-graph";
import { ComparisonBars } from "../elements/remotion-ui/comparison-bars";
import { ComparisonTable } from "../elements/remotion-ui/comparison-table";
import { DonutChart } from "../elements/remotion-ui/donut-chart";
import { EndCard } from "../elements/remotion-ui/end-card";
import { FunnelChart } from "../elements/remotion-ui/funnel-chart";
import { KanbanMove } from "../elements/remotion-ui/kanban-move";
import { LogoWall } from "../elements/remotion-ui/logo-wall";
import { MetricTicker } from "../elements/remotion-ui/metric-ticker";
import { PricingCard } from "../elements/remotion-ui/pricing-card";
import type { ElementComponent } from "./types";

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
  const meta = ELEMENTS.find((e) => e.key === `remotion-ui/${key}`);
  if (!meta) throw new Error(key);
  const props: ElementProps = { ...meta.defaultProps, ...meta.fixedProps };
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
    <Thumbnail
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
