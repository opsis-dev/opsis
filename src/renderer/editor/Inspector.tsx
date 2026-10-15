import {
  CircleAlert,
  ExternalLink,
  FileSearch,
  Film,
  Play,
  Plus,
  RotateCcw,
  Trash2,
  Ungroup,
} from "lucide-react";
import { toast } from "sonner";
import { downstreamOf, sessionChain, upstreamOf } from "@/shared/graph";
import { CATEGORY_LABELS } from "@/shared/nodes/define";
import { getNodeDef } from "@/shared/nodes/registry";
import type { RecordingValue } from "@/shared/schema/values";
import type { Workflow } from "@/shared/schema/workflow";
import { PROJECT_NAME_VAR, workflowVars } from "@/shared/compile/compile";
import { interpolate, renameVariable } from "@/shared/vars";
import { Alert, AlertDescription, AlertTitle } from "@/renderer/components/ui/alert";
import { Badge } from "@/renderer/components/ui/badge";
import { Button } from "@/renderer/components/ui/button";
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
  FieldSet,
  FieldLegend,
} from "@/renderer/components/ui/field";
import { Input } from "@/renderer/components/ui/input";
import { ScrollArea } from "@/renderer/components/ui/scroll-area";
import { Separator } from "@/renderer/components/ui/separator";
import { Switch } from "@/renderer/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "@/renderer/components/ui/toggle-group";
import { useRunStore } from "../store/run";
import { useSecretsStore } from "../store/secrets";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { NodeIcon } from "./icons";
import { fileUrl } from "./file-url";
import { CATEGORY_COLORS } from "./ports";
import { startRun } from "./run";
import { useCompiledContext } from "./use-compiled";
import { ZodForm, type FormContext, type SpotBackground } from "./ZodForm";

const SITE_VAR = "Site address";

export function variableList(wf: Workflow) {
  const vars = workflowVars(wf);
  return [
    ...Object.entries(wf.variables).map(([name, value]) => ({ name, value })),
    { name: PROJECT_NAME_VAR, value: vars[PROJECT_NAME_VAR] },
  ];
}

function spotBackground(
  wf: Workflow,
  nodeId: string,
  results: Record<string, { outputs: Record<string, unknown> }>,
): SpotBackground {
  for (const id of upstreamOf(wf, nodeId)) {
    const rec = results[id]?.outputs.recording as RecordingValue | undefined;
    if (rec?.src) return { src: fileUrl(rec.src), timeMs: 0 };
  }
  return null;
}

function pickerUrl(wf: Workflow, nodeId: string): string | null {
  const capture = [nodeId, ...downstreamOf(wf, nodeId)].find((id) => {
    const t = wf.nodes.find((n) => n.id === id)?.type;
    return t === "capture.record" || t === "capture.still";
  });
  let chain: Workflow["nodes"] = [];
  try {
    chain = capture ? sessionChain(wf, capture) : [];
  } catch {
    chain = [];
  }
  const idx = chain.findIndex((n) => n.id === nodeId);
  const before = idx >= 0 ? chain.slice(0, idx + 1) : chain;
  const nav = before.toReversed().find((n) => n.type === "browser.navigate");
  const url =
    (nav?.params.url as string | undefined) ?? wf.variables[SITE_VAR] ?? wf.variables.BASE_URL;
  return url ? interpolate(url, workflowVars(wf)) : null;
}

export function Inspector() {
  const selected = useUiStore((s) => s.selectedIds);
  const workflow = useWorkflowStore((s) => s.workflow);
  if (!workflow) return null;
  const node = selected.length === 1 ? workflow.nodes.find((n) => n.id === selected[0]) : undefined;
  return (
    <ScrollArea className="h-full">
      {node ? <NodeInspector nodeId={node.id} /> : <ProjectInspector multi={selected.length} />}
    </ScrollArea>
  );
}

function NodeInspector({ nodeId }: { nodeId: string }) {
  const workflow = useWorkflowStore((s) => s.workflow)!;
  const projectDir = useWorkflowStore((s) => s.projectDir);
  const { updateParams, setNodeMeta, removeNodes, ungroup } = useWorkflowStore.getState();
  const node = workflow.nodes.find((n) => n.id === nodeId)!;
  const def = getNodeDef(node.type);
  const { result, issues } = useCompiledContext();
  const runError = useRunStore((s) => s.errors[nodeId]);
  const running = useRunStore((s) => s.running);
  const nodeResult = useRunStore((s) => s.results[nodeId]);
  const traceId = useRunStore((s) => s.traces[nodeId]);
  const setReplay = useUiStore((s) => s.setReplay);
  const secrets = useSecretsStore((s) => s.names);
  const results = useRunStore((s) => s.results);
  if (!def) return <p className="p-4 text-sm">This node type isn't available.</p>;
  const problems = [
    runError,
    result?.errors[nodeId],
    ...issues.filter((i) => i.nodeId === nodeId).map((i) => i.message),
  ].filter(Boolean) as string[];

  const layers = workflow.edges.filter(
    (e) => e.target === nodeId && /^in\d$/.test(e.targetHandle),
  ).length;
  const ctx: FormContext = {
    projectDir,
    fps: workflow.settings.fps,
    aspect: workflow.settings.width / workflow.settings.height,
    vars: variableList(workflow),
    secrets,
    layers: Math.max(1, layers),
    spotBackground: def.category === "Camera" ? spotBackground(workflow, nodeId, results) : null,
    pickTarget: async (key) => {
      const url = pickerUrl(workflow, nodeId);
      if (!url) {
        toast.error("Add a Go to page action before this one, so there's a page to pick from.");
        return;
      }
      toast.info("A browser window opened. Hover over the element and click it.");
      try {
        const picked = await window.demoflow.pickElement({ url });
        if (picked) {
          updateParams(nodeId, { [key]: picked.target, [`${key}Role`]: picked.role || undefined });
          toast.success(`Picked “${picked.target}”`);
        }
      } catch (e) {
        toast.error("Couldn't pick the element", {
          description: e instanceof Error ? e.message : String(e),
        });
      }
    },
    chooseAsset: async (key) => {
      if (!projectDir) return;
      const [path] = await window.demoflow.importAssets(projectDir);
      if (path) updateParams(nodeId, { [key]: path });
    },
  };

  return (
    <div className="flex flex-col gap-4 p-4">
      <div className="flex items-start gap-3">
        <span
          className="grid size-8 shrink-0 place-items-center rounded-md text-black"
          style={{ background: CATEGORY_COLORS[def.category] }}
        >
          <NodeIcon name={def.icon} className="size-4" />
        </span>
        <div className="flex min-w-0 flex-col gap-1">
          <h2 className="truncate font-semibold">{node.label || def.title}</h2>
          <div className="flex flex-wrap gap-1">
            <Badge variant="secondary">{CATEGORY_LABELS[def.category]}</Badge>
            {node.label ? <Badge variant="outline">{def.title}</Badge> : null}
          </div>
        </div>
      </div>
      {def.description ? <p className="text-muted-foreground text-sm">{def.description}</p> : null}
      {problems.length ? (
        <Alert variant="destructive">
          <CircleAlert />
          <AlertTitle>
            {problems.length === 1
              ? "Something needs attention"
              : `${problems.length} things need attention`}
          </AlertTitle>
          <AlertDescription>
            {problems.map((p, i) => (
              <p key={i} className="whitespace-pre-wrap">
                {p}
              </p>
            ))}
          </AlertDescription>
        </Alert>
      ) : null}

      <FieldGroup className="gap-4">
        <Field>
          <FieldLabel htmlFor="node-label">Name</FieldLabel>
          <Input
            id="node-label"
            value={node.label ?? ""}
            placeholder={def.title}
            onChange={(e) => setNodeMeta(nodeId, { label: e.target.value || undefined })}
          />
          {def.category === "Browser" ? (
            <FieldDescription>Also used by the camera, chapters and captions.</FieldDescription>
          ) : null}
        </Field>
        <Field orientation="horizontal" className="justify-between">
          <FieldLabel htmlFor="node-disabled" className="font-normal">
            Skip this node
          </FieldLabel>
          <Switch
            id="node-disabled"
            checked={!!node.disabled}
            onCheckedChange={(v) => setNodeMeta(nodeId, { disabled: v || undefined })}
          />
        </Field>
      </FieldGroup>

      <Separator />
      <ZodForm
        schema={def.params}
        value={node.params}
        onChange={(patch) => updateParams(nodeId, patch)}
        ctx={ctx}
      />
      <Separator />

      <div className="flex flex-wrap gap-2">
        {def.executes || def.virtual ? (
          <>
            <Button
              size="sm"
              disabled={running}
              onClick={() => void startRun({ targetNodeIds: [nodeId] })}
            >
              <Play data-icon="inline-start" />
              Run this
            </Button>
            <Button
              size="sm"
              variant="outline"
              disabled={running}
              onClick={() => void startRun({ fromNodeId: nodeId })}
            >
              <RotateCcw data-icon="inline-start" />
              Re-run from here
            </Button>
          </>
        ) : null}
        {def.category === "Capture" && (nodeResult || traceId) ? (
          <Button size="sm" variant="outline" onClick={() => setReplay(nodeId)}>
            <Film data-icon="inline-start" />
            Replay
          </Button>
        ) : null}
        {traceId && projectDir ? (
          <Button size="sm" variant="outline" onClick={() => setReplay(nodeId, "trace")}>
            <FileSearch data-icon="inline-start" />
            Trace
          </Button>
        ) : null}
        {node.type === "core.group" ? (
          <Button size="sm" variant="outline" onClick={() => ungroup(nodeId)}>
            <Ungroup data-icon="inline-start" />
            Ungroup
          </Button>
        ) : null}
        <Button size="sm" variant="ghost" onClick={() => removeNodes([nodeId])}>
          <Trash2 data-icon="inline-start" />
          Delete
        </Button>
      </div>

      {nodeResult?.artifacts.length && projectDir ? (
        <FieldSet>
          <FieldLegend variant="label">Files</FieldLegend>
          {nodeResult.artifacts.map((a) => (
            <Button
              key={a.path}
              variant="link"
              size="sm"
              className="h-auto justify-start p-0"
              onClick={() => void window.demoflow.revealInFolder(projectDir, a.path)}
            >
              <ExternalLink data-icon="inline-start" />
              {a.label ?? a.path}
            </Button>
          ))}
        </FieldSet>
      ) : null}
    </div>
  );
}

const SIZES: Array<[string, number, number]> = [
  ["16:9", 1920, 1080],
  ["1:1", 1080, 1080],
  ["4:5", 1080, 1350],
  ["9:16", 1080, 1920],
];

function ProjectInspector({ multi }: { multi: number }) {
  const workflow = useWorkflowStore((s) => s.workflow)!;
  const { setSettings, setVariables, rename, groupNodes } = useWorkflowStore.getState();
  const selected = useUiStore((s) => s.selectedIds);
  const { issues } = useCompiledContext();
  const s = workflow.settings;
  const vars = Object.entries(workflow.variables);
  const preset = SIZES.find(([, w, h]) => w === s.width && h === s.height)?.[0] ?? "custom";

  return (
    <div className="flex flex-col gap-5 p-4">
      {multi > 1 ? (
        <Alert>
          <AlertTitle>{multi} nodes selected</AlertTitle>
          <AlertDescription className="flex flex-col items-start gap-2">
            <span>Group them to keep a long flow tidy.</span>
            <Button size="sm" variant="outline" onClick={() => groupNodes(selected)}>
              Group selection
            </Button>
          </AlertDescription>
        </Alert>
      ) : null}
      <FieldGroup className="gap-4">
        <Field>
          <FieldLabel htmlFor="wf-name">Project name</FieldLabel>
          <Input id="wf-name" value={workflow.name} onChange={(e) => rename(e.target.value)} />
        </Field>
        <Field>
          <FieldLabel>Canvas</FieldLabel>
          <ToggleGroup
            variant="outline"
            size="sm"
            value={[preset]}
            onValueChange={(v) => {
              const hit = SIZES.find(([k]) => k === v[0]);
              if (hit) setSettings({ width: hit[1], height: hit[2] });
            }}
          >
            {SIZES.map(([k]) => (
              <ToggleGroupItem key={k} value={k}>
                {k}
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
        </Field>
        <div className="grid grid-cols-2 gap-3">
          <Field>
            <FieldLabel htmlFor="wf-w">Width</FieldLabel>
            <Input
              id="wf-w"
              type="number"
              value={s.width}
              onChange={(e) =>
                Number(e.target.value) > 0 && setSettings({ width: Number(e.target.value) })
              }
            />
          </Field>
          <Field>
            <FieldLabel htmlFor="wf-h">Height</FieldLabel>
            <Input
              id="wf-h"
              type="number"
              value={s.height}
              onChange={(e) =>
                Number(e.target.value) > 0 && setSettings({ height: Number(e.target.value) })
              }
            />
          </Field>
        </div>
        <Field>
          <FieldLabel>Frame rate</FieldLabel>
          <ToggleGroup
            variant="outline"
            size="sm"
            value={[String(s.fps)]}
            onValueChange={(v) => v[0] && setSettings({ fps: Number(v[0]) as 30 | 60 })}
          >
            <ToggleGroupItem value="30">30 fps</ToggleGroupItem>
            <ToggleGroupItem value="60">60 fps</ToggleGroupItem>
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
      </FieldGroup>

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
                onChange={(e) => {
                  const next = e.target.value.replace(/[{}]/g, "");
                  useWorkflowStore.getState().update((wf) => ({
                    ...wf,
                    variables: Object.fromEntries(
                      vars.map(([k, v], j) => (j === i ? [next, v] : [k, v])),
                    ),
                    nodes: next.trim()
                      ? wf.nodes.map((n) => ({ ...n, params: renameVariable(n.params, key, next) }))
                      : wf.nodes,
                  }));
                }}
              />
              <Input
                aria-label="Variable value"
                className="flex-1 text-xs"
                value={value}
                onChange={(e) => setVariables({ ...workflow.variables, [key]: e.target.value })}
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
            onClick={() =>
              setVariables({ ...workflow.variables, [`Variable ${vars.length + 1}`]: "" })
            }
          >
            <Plus data-icon="inline-start" />
            Add variable
          </Button>
        </div>
      </FieldSet>

      {issues.length ? (
        <FieldSet>
          <FieldLegend variant="label">Problems</FieldLegend>
          <ul className="flex flex-col gap-1 text-xs">
            {issues.slice(0, 30).map((issue, i) => (
              <li
                key={i}
                className={issue.level === "error" ? "text-destructive" : "text-muted-foreground"}
              >
                {issue.message}
              </li>
            ))}
          </ul>
        </FieldSet>
      ) : null}
    </div>
  );
}
