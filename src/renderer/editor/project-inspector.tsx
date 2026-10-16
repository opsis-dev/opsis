import { Plus, Trash2 } from "lucide-react";
import { toast } from "sonner";
import type { Workflow, WorkflowSettings } from "@/shared/schema/workflow";
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
import { FontPicker } from "../fonts/font-picker";
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { useCompiledContext } from "./use-compiled";

const CANVAS_SIZES = [
  { name: "16:9", width: 1920, height: 1080 },
  { name: "1:1", width: 1080, height: 1080 },
  { name: "4:5", width: 1080, height: 1350 },
  { name: "9:16", width: 1080, height: 1920 },
];

const FRAME_RATES = [30, 60] as const;

type FontRole = "main" | "heading" | "code";

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
        onChange={(e) => Number(e.target.value) > 0 && onChange(Number(e.target.value))}
      />
    </Field>
  );
}

function CanvasSettings({ settings: s }: { settings: WorkflowSettings }) {
  const setSettings = useWorkflowStore((st) => st.setSettings);
  const preset = CANVAS_SIZES.find((c) => c.width === s.width && c.height === s.height);
  return (
    <>
      <Field>
        <FieldLabel>Canvas</FieldLabel>
        <ToggleGroup
          variant="outline"
          size="sm"
          value={[preset?.name ?? "custom"]}
          onValueChange={(v) => {
            const hit = CANVAS_SIZES.find((c) => c.name === v[0]);
            if (hit) setSettings({ width: hit.width, height: hit.height });
          }}
        >
          {CANVAS_SIZES.map((c) => (
            <ToggleGroupItem key={c.name} value={c.name}>
              {c.name}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      </Field>
      <div className="grid grid-cols-2 gap-3">
        <SizeInput
          id="wf-w"
          label="Width"
          value={s.width}
          onChange={(width) => setSettings({ width })}
        />
        <SizeInput
          id="wf-h"
          label="Height"
          value={s.height}
          onChange={(height) => setSettings({ height })}
        />
      </div>
      <Field>
        <FieldLabel>Frame rate</FieldLabel>
        <ToggleGroup
          variant="outline"
          size="sm"
          value={[String(s.fps)]}
          onValueChange={(v) => {
            const fps = FRAME_RATES.find((r) => String(r) === v[0]);
            if (fps) setSettings({ fps });
          }}
        >
          {FRAME_RATES.map((r) => (
            <ToggleGroupItem key={r} value={String(r)}>
              {r} fps
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      </Field>
      <Field>
        <FieldLabel htmlFor="wf-bg">Background</FieldLabel>
        <Input
          id="wf-bg"
          value={s.background ?? ""}
          placeholder={s.theme === "light" ? "#fafafa" : "#09090b"}
          onChange={(e) => setSettings({ background: e.target.value || undefined })}
        />
      </Field>
    </>
  );
}

function FontSettings({ fonts }: { fonts: WorkflowSettings["fonts"] }) {
  const setSettings = useWorkflowStore((s) => s.setSettings);
  const pick = (role: FontRole, family: string) => {
    const latest = useWorkflowStore.getState().workflow?.settings.fonts ?? fonts;
    setSettings({
      fonts: { ...latest, [role]: family || (role === "heading" ? "" : latest[role]) },
    });
  };
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
  const vars = Object.entries(variables);
  const renameAt = (from: string, raw: string) => {
    const to = raw.replace(/[{}]/g, "");
    if (to !== from && to in variables) {
      toast.error(`There's already a variable called “${to}”.`);
      return;
    }
    renameVariable(from, to);
  };
  return (
    <FieldSet>
      <FieldLegend variant="label">Variables</FieldLegend>
      <p className="text-muted-foreground text-xs">
        Reuse values like your site's address. Insert them into any text with Insert variable.
      </p>
      <div className="flex flex-col gap-2">
        {vars.map(([key, value], i) => (
          <div key={i} className="flex gap-2">
            <Input
              aria-label="Variable name"
              className="w-2/5 text-xs"
              value={key}
              onChange={(e) => renameAt(key, e.target.value)}
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
              onClick={() => setVariables(Object.fromEntries(vars.filter((_, j) => j !== i)))}
            >
              <Trash2 />
            </Button>
          </div>
        ))}
        <Button
          size="sm"
          variant="outline"
          onClick={() => setVariables({ ...variables, [`Variable ${vars.length + 1}`]: "" })}
        >
          <Plus data-icon="inline-start" />
          Add variable
        </Button>
      </div>
    </FieldSet>
  );
}

function SavedResults() {
  const running = useRunStore((s) => s.running);
  const clear = async () => {
    const dir = useWorkflowStore.getState().projectDir;
    if (!dir) return;
    await window.demoflow.clearCache(dir);
    useRunStore.getState().seed(null);
    toast.success("Cleared. The next run records and exports everything again.");
  };
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
  return (
    <FieldSet>
      <FieldLegend variant="label">Problems</FieldLegend>
      <ul className="flex flex-col gap-1 text-xs">
        {issues.slice(0, MAX_PROBLEMS).map((issue, i) => (
          <li
            key={i}
            className={issue.level === "error" ? "text-destructive" : "text-muted-foreground"}
          >
            {issue.message}
          </li>
        ))}
      </ul>
    </FieldSet>
  );
}
