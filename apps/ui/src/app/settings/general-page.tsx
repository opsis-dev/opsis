import { START_OPTION_LABELS, START_OPTIONS, type AppSettings } from "@opsis/core/settings";
import { Field, FieldGroup, FieldLabel } from "@/components/ui/field";
import { patchAppSettings } from "../../store/app-settings";
import { OptionToggle } from "../option-toggle";
import { SwitchField } from "./fields";
import { SettingsCard } from "./settings-card";

export function GeneralPage({ app }: { app: AppSettings["app"] }) {
  return (
    <SettingsCard title="Starting and closing">
      <FieldGroup>
        <Field>
          <FieldLabel>When Opsis starts</FieldLabel>
          <OptionToggle
            options={START_OPTIONS}
            labels={START_OPTION_LABELS}
            value={app.onStart}
            onChange={(onStart) => void patchAppSettings({ app: { onStart } })}
          />
        </Field>
        <SwitchField
          id="confirm-exit"
          label="Ask before closing"
          description="When this is off, Opsis still asks if a run, an engine setup or a save hasn't finished."
          checked={app.confirmExit}
          onChange={(v) => void patchAppSettings({ app: { confirmExit: v } })}
        />
      </FieldGroup>
    </SettingsCard>
  );
}
