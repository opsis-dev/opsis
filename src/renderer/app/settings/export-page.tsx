import { Field, FieldDescription, FieldGroup, FieldLabel } from "@/renderer/components/ui/field";
import type { AppSettings } from "@/shared/settings";
import { patchAppSettings } from "../../store/app-settings";
import { CommitInput } from "./fields";
import { SettingsCard } from "./settings-card";

function WebLink({ href, children }: { href: string; children: string }) {
  return (
    <a href={href} target="_blank" rel="noreferrer" className="underline underline-offset-2">
      {children}
    </a>
  );
}

export function ExportPage({ render }: { render: AppSettings["render"] }) {
  return (
    <SettingsCard
      title="Video export licence"
      description="Demoflow renders videos with Remotion, which has its own licence. It's free for individuals, non-profits and companies with up to 3 employees. Larger companies need a Remotion company licence and must enter its key here."
    >
      <FieldGroup>
        <Field>
          <FieldLabel htmlFor="remotion-key">Company licence key</FieldLabel>
          <CommitInput
            id="remotion-key"
            type="password"
            autoComplete="off"
            placeholder="Leave empty to use the free licence"
            value={render.licenseKey}
            onCommit={(licenseKey) => void patchAppSettings({ render: { licenseKey } })}
          />
          <FieldDescription>
            Read the terms at{" "}
            <WebLink href="https://www.remotion.dev/license">remotion.dev/license</WebLink>; company
            licences are at <WebLink href="https://www.remotion.pro">remotion.pro</WebLink>.
          </FieldDescription>
        </Field>
      </FieldGroup>
    </SettingsCard>
  );
}
