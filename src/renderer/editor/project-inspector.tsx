import { Plus, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { Alert, AlertDescription, AlertTitle } from "@/renderer/components/ui/alert";
import { Button } from "@/renderer/components/ui/button";
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
  FieldLegend,
  FieldSet,
} from "@/renderer/components/ui/field";
import { Input } from "@/renderer/components/ui/input";
import { ToggleGroup, ToggleGroupItem } from "@/renderer/components/ui/toggle-group";
import { whenFirstOneOf } from "@/renderer/lib/one-of";
import { attempt } from "@/renderer/lib/toast";
import { useItemKeys } from "@/renderer/lib/use-item-keys";
import { THEME_BACKGROUNDS } from "@/shared/compile/clip";
import type { FontRole } from "@/shared/fonts";
import { firstFree, uniqueKeys } from "@/shared/lists";
import { VARIANTS, variantCanvas } from "@/shared/nodes/output";
import type { Workflow, WorkflowSettings } from "@/shared/schema/workflow";
import { FontPicker } from "../fonts/font-picker";
import { useMedia } from "../store/media";
import { useRunStore } from "../store/run";
import { useStillsStore } from "../store/stills";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { InlineEdit } from "./inline-edit";
import { useCompiledContext } from "./use-compiled";

const CANVAS_SHORT_SIDE = 1080;

const FRAME_RATES = [30, 60] as const;

const FONT_ROLES: Array<{ role: FontRole; label: string; empty?: string }> = [
  { role: "main", label: "Text" },
  { role: "heading", label: "Headings", empty: "Same as text" },
  { role: "code", label: "Code" },
];

const MAX_PROBLEMS = 30;

export function ProjectInspector({ workflow, multi }: { workflow: Workflow; multi: number }) {
  const rename = useWorkflowStore((s) => s.rename);
  return (
    <div className="flex flex-col gap-5 p-4">
      {multi > 1 ? <GroupSelection count={multi} /> : null}
      <FieldGroup className="gap-4">
        <Field>
          <FieldLabel htmlFor="wf-name">Project name</FieldLabel>
          <Input id="wf-name" value={workflow.name} onChange={(e) => rename(e.target.value)} />
        </Field>
        <CanvasSettings settings={workflow.settings} />
      </FieldGroup>
      <FontSettings fonts={workflow.settings.fonts} />
      <VariablesEditor variables={workflow.variables} />
      <SavedResults />
      <ProblemList />
    </div>
  );
}

function GroupSelection({ count }: { count: number }) {
  const selected = useUiStore((s) => s.selectedIds);
  const groupNodes = useWorkflowStore((s) => s.groupNodes);
  return (
    <Alert>
      <AlertTitle>{count} nodes selected</AlertTitle>
      <AlertDescription className="flex flex-col items-start gap-2">
        <span>Group them to keep a long flow tidy.</span>
        <Button size="sm" variant="outline" onClick={() => groupNodes(selected)}>
          Group selection
        </Button>
      </AlertDescription>
    </Alert>
  );
}

function SizeInput({
  id,
  label,
  value,
  onChange,
}: {
  id: string;
  label: string;
  value: number;
  onChange: (value: number) => void;
}) {
  return (
    <Field>
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
      <Input
        id={id}
        type="number"
        value={value}
        onChange={(e) => {
          const size = Number(e.target.value);
          if (size > 0) onChange(size);
        }}
      />
    </Field>
  );
}

function CanvasSettings({ settings }: { settings: WorkflowSettings }) {
  const setSettings = useWorkflowStore((s) => s.setSettings);
  const preset = VARIANTS.find((v) => {
    const size = variantCanvas(v, CANVAS_SHORT_SIDE);
    return size.width === settings.width && size.height === settings.height;
  });
  return (
    <>
      <Field>
        <FieldLabel>Canvas</FieldLabel>
        <ToggleGroup
          variant="outline"
          size="sm"
          value={[preset ?? "custom"]}
          onValueChange={whenFirstOneOf(VARIANTS, (v) =>
            setSettings(variantCanvas(v, CANVAS_SHORT_SIDE)),
          )}
        >
          {VARIANTS.map((v) => (
            <ToggleGroupItem key={v} value={v}>
              {v}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      </Field>
      <div className="grid grid-cols-2 gap-3">
        <SizeInput
          id="wf-w"
          label="Width"
          value={settings.width}
          onChange={(width) => setSettings({ width })}
        />
        <SizeInput
          id="wf-h"
          label="Height"
          value={settings.height}
          onChange={(height) => setSettings({ height })}
        />
      </div>
      <FrameRateField fps={settings.fps} />
      <BackgroundField background={settings.background} theme={settings.theme} />
    </>
  );
}

function FrameRateField({ fps }: { fps: number }) {
  const setSettings = useWorkflowStore((s) => s.setSettings);
  return (
    <Field>
      <FieldLabel>Frame rate</FieldLabel>
      <ToggleGroup
        variant="outline"
        size="sm"
        value={[String(fps)]}
        onValueChange={whenFirstOneOf(FRAME_RATES, (rate) => setSettings({ fps: rate }))}
      >
        {FRAME_RATES.map((r) => (
          <ToggleGroupItem key={r} value={String(r)}>
            {r} fps
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
    </Field>
  );
}

function BackgroundField({ background, theme }: Pick<WorkflowSettings, "background" | "theme">) {
  const setSettings = useWorkflowStore((s) => s.setSettings);
  return (
    <Field>
      <FieldLabel htmlFor="wf-bg">Background</FieldLabel>
      <Input
        id="wf-bg"
        value={background ?? ""}
        placeholder={THEME_BACKGROUNDS[theme]}
        onChange={(e) => setSettings({ background: e.target.value || undefined })}
      />
    </Field>
  );
}

function FontSettings({ fonts }: { fonts: WorkflowSettings["fonts"] }) {
  const setSettings = useWorkflowStore((s) => s.setSettings);
  const pick = (role: FontRole, family: string) =>
    setSettings({ fonts: { ...fonts, [role]: family } });
  return (
    <FieldSet>
      <FieldLegend variant="label">Fonts</FieldLegend>
      <FieldDescription>
        Every text in the video follows these unless a node picks its own font.
      </FieldDescription>
      <FieldGroup className="gap-3">
        {FONT_ROLES.map(({ role, label, empty }) => (
          <Field key={role}>
            <FieldLabel htmlFor={`wf-font-${role}`}>{label}</FieldLabel>
            <FontPicker
              id={`wf-font-${role}`}
              value={fonts[role]}
              emptyLabel={empty}
              onChange={(family) => pick(role, family)}
            />
          </Field>
        ))}
      </FieldGroup>
    </FieldSet>
  );
}

function VariablesEditor({ variables }: { variables: Record<string, string> }) {
  const { setVariables, renameVariable } = useWorkflowStore.getState();
  const list = useItemKeys(Object.entries(variables));
  const renameAt = (from: string, raw: string) => {
    const to = raw.replace(/[{}]/g, "").trim();
    if (!to) return;
    if (to in variables) {
      toast.error(`There's already a variable called “${to}”.`);
      return;
    }
    renameVariable(from, to);
  };
  const addVariable = () => {
    const name = firstFree(
      (n) => `Variable ${n}`,
      (taken) => taken in variables,
    );
    setVariables(Object.fromEntries(list.add([name, ""])));
  };
  return (
    <FieldSet>
      <FieldLegend variant="label">Variables</FieldLegend>
      <p className="text-muted-foreground text-xs">
        Reuse values like your site's address. Insert them into any text with Insert variable.
      </p>
      <div className="flex flex-col gap-2">
        {list.rows.map(({ item: [key, value], key: rowKey }, i) => (
          <div key={rowKey} className="flex gap-2">
            <InlineEdit
              key={key}
              label="Variable name"
              autoFocus={false}
              className="w-2/5 text-xs"
              value={key}
              onCommit={(text) => renameAt(key, text)}
            />
            <Input
              aria-label="Variable value"
              className="flex-1 text-xs"
              value={value}
              onChange={(e) => setVariables({ ...variables, [key]: e.target.value })}
            />
            <Button
              size="icon"
              variant="ghost"
              aria-label="Remove variable"
              onClick={() => setVariables(Object.fromEntries(list.remove(i)))}
            >
              <Trash2 />
            </Button>
          </div>
        ))}
        <Button size="sm" variant="outline" onClick={addVariable}>
          <Plus data-icon="inline-start" />
          Add variable
        </Button>
      </div>
    </FieldSet>
  );
}

function SavedResults() {
  const running = useRunStore((s) => s.running);
  const clear = () =>
    attempt(async () => {
      const dir = useWorkflowStore.getState().projectDir;
      if (!dir) return;
      await window.demoflow.clearCache(dir);
      useRunStore.getState().seed(null);
      useMedia.getState().reset(dir);
      await useStillsStore.getState().load(dir);
      toast.success("Cleared. The next run records and exports everything again.");
    }, "Couldn't clear the saved results");
  return (
    <FieldSet>
      <FieldLegend variant="label">Saved results</FieldLegend>
      <p className="text-muted-foreground text-xs">
        Demoflow keeps the result of every node so unchanged parts don't run again. Clear them to
        record and export everything again on the next run.
      </p>
      <Button size="sm" variant="outline" disabled={running} onClick={() => void clear()}>
        <Trash2 data-icon="inline-start" />
        Clear saved results
      </Button>
    </FieldSet>
  );
}

function ProblemList() {
  const { issues } = useCompiledContext();
  if (!issues.length) return null;
  const shown = issues.slice(0, MAX_PROBLEMS);
  const keys = uniqueKeys(shown.map((issue) => issue.message));
  return (
    <FieldSet>
      <FieldLegend variant="label">Problems</FieldLegend>
      <ul className="flex flex-col gap-1 text-xs">
        {shown.map((issue, i) => (
          <li
            key={keys[i]}
            className={issue.level === "error" ? "text-destructive" : "text-muted-foreground"}
          >
            {issue.message}
          </li>
        ))}
      </ul>
    </FieldSet>
  );
}
