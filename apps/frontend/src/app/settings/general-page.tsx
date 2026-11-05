import { Field, FieldDescription, FieldGroup, FieldLabel } from "@/components/ui/field";
import type { AppSettings } from "@/generated/bindings";
import { patchAppSettings } from "@/store/app-settings";
import { settingsBook } from "@/store/catalog";
import { OptionToggle } from "../option-toggle";
import { SwitchField } from "./fields";
import { SettingsCard } from "./settings-card";

function StartingCard({ app }: { app: AppSettings["app"] }) {
  return (
    <SettingsCard title="Starting and closing">
      <FieldGroup>
        <Field>
          <FieldLabel>When Opsis starts</FieldLabel>
          <OptionToggle
            choices={settingsBook().startOptions}
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

function WorkingFilesCard({ media }: { media: AppSettings["media"] }) {
  return (
    <SettingsCard
      title="Working files"
      description="The videos nodes such as Trim, Reverse, Speed, Convert and Apply grade make along the way."
    >
      <FieldGroup>
        <Field>
          <FieldLabel>Speed vs file size</FieldLabel>
          <OptionToggle
            choices={settingsBook().encodeSpeeds}
            value={media.encodeSpeed}
            onChange={(encodeSpeed) => void patchAppSettings({ media: { encodeSpeed } })}
          />
          <FieldDescription>
            Faster makes bigger files. Files already made stay as they are, and exports keep the
            Render node's own setting.
          </FieldDescription>
        </Field>
      </FieldGroup>
    </SettingsCard>
  );
}

export function GeneralPage({
  app,
  media,
}: {
  app: AppSettings["app"];
  media: AppSettings["media"];
}) {
  return (
    <>
      <StartingCard app={app} />
      <WorkingFilesCard media={media} />
    </>
  );
}
