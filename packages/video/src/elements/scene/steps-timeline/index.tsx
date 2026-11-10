import { stepsLook } from "#video/elements/scene/steps-timeline/look.ts";
import { TimelineSteps } from "#video/elements/scene/steps-timeline/timeline-steps.tsx";
import type { ElementSettings } from "#video/generated/spec.ts";

export function StepsTimeline(settings: ElementSettings["steps-timeline"]) {
  const look = stepsLook(settings);
  return (
    <TimelineSteps
      steps={settings.steps}
      title={settings.title}
      eyebrow={settings.eyebrow}
      look={look}
      backgroundColor={settings.backgroundColor || look.palette.page}
      accentColor={settings.accentColor}
      speed={settings.speed}
    />
  );
}
