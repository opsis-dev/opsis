import { Field, FieldDescription, FieldGroup, FieldLabel } from "@/renderer/components/ui/field";
import { clamp } from "@/shared/math";
import {
  KEEP_LAST_TRACES,
  LIVE_FPS,
  TRACE_RETENTION,
  TRACE_RETENTION_LABELS,
  type AppSettings,
} from "@/shared/settings";
import { parseNumberText } from "@/shared/text-input";
import { patchAppSettings } from "../../store/app-settings";
import { OptionToggle } from "../option-toggle";
import { SingleSlider } from "../single-slider";
import { CommitInput, SwitchField } from "./fields";
import { SettingsCard } from "./settings-card";

function setKeepLast(text: string): void {
  const n = parseNumberText(text);
  if (n === null) return;
  void patchAppSettings({
    traces: { keepLast: clamp(Math.round(n), KEEP_LAST_TRACES.min, KEEP_LAST_TRACES.max) },
  });
}

function TracesCard({ traces }: { traces: AppSettings["traces"] }) {
  return (
    <SettingsCard
      title="Traces"
      description="A step-by-step record of every take, for finding out what went wrong. Open it from the Trace tab in Replay. Stored compressed inside the project."
    >
      <FieldGroup>
        <Field>
          <FieldLabel>Keep traces</FieldLabel>
          <OptionToggle
            options={TRACE_RETENTION}
            labels={TRACE_RETENTION_LABELS}
            value={traces.retention}
            onChange={(retention) => void patchAppSettings({ traces: { retention } })}
          />
        </Field>
        {traces.retention === "last-n" ? (
          <Field>
            <FieldLabel htmlFor="keep-last">Runs to keep per node</FieldLabel>
            <CommitInput
              id="keep-last"
              type="number"
              min={KEEP_LAST_TRACES.min}
              max={KEEP_LAST_TRACES.max}
              value={String(traces.keepLast)}
              onCommit={setKeepLast}
            />
          </Field>
        ) : null}
      </FieldGroup>
    </SettingsCard>
  );
}

function LiveViewCard({ live }: { live: AppSettings["live"] }) {
  return (
    <SettingsCard
      title="Live view"
      description="What the recording browser is doing, shown in the editor while a run is going."
    >
      <FieldGroup>
        <Field>
          <div className="flex items-center justify-between">
            <FieldLabel>Preview frame rate</FieldLabel>
            <span className="text-muted-foreground text-sm tabular-nums">{live.fps} fps</span>
          </div>
          <SingleSlider
            min={LIVE_FPS.min}
            max={LIVE_FPS.max}
            step={1}
            value={live.fps}
            onChange={(fps) => void patchAppSettings({ live: { fps } })}
          />
          <FieldDescription>
            Only affects the live view; recordings always use the Record node's frame rate.
          </FieldDescription>
        </Field>
        <SwitchField
          id="live-auto"
          label="Open the live view when a take starts"
          checked={live.autoOpen}
          onChange={(v) => void patchAppSettings({ live: { autoOpen: v } })}
        />
      </FieldGroup>
    </SettingsCard>
  );
}

export function RecordingPage({
  traces,
  live,
}: {
  traces: AppSettings["traces"];
  live: AppSettings["live"];
}) {
  return (
    <>
      <TracesCard traces={traces} />
      <LiveViewCard live={live} />
    </>
  );
}
