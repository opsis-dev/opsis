import {
  CircleAlert,
  ExternalLink,
  FileSearch,
  Film,
  Play,
  RotateCcw,
  Trash2,
  Ungroup,
} from "lucide-react";
import { CATEGORY_LABELS, type NodeDef } from "@/shared/nodes/define";
import { getNodeDef, nodeName } from "@/shared/nodes/registry";
import type { Workflow, WorkflowNode } from "@/shared/schema/workflow";
import { Alert, AlertDescription, AlertTitle } from "@/renderer/components/ui/alert";
import { Badge } from "@/renderer/components/ui/badge";
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
import { ScrollArea } from "@/renderer/components/ui/scroll-area";
import { Separator } from "@/renderer/components/ui/separator";
import { Switch } from "@/renderer/components/ui/switch";
import { SwapFade } from "../app/motion";
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { CategoryBadge } from "./category-badge";
import { ProjectInspector } from "./project-inspector";
import { startRun } from "./run";
import { useCompiledContext } from "./use-compiled";
import { useFormContext } from "./use-form-context";
import { ZodForm } from "./zod-form";

export function Inspector() {
  const selected = useUiStore((s) => s.selectedIds);
  const workflow = useWorkflowStore((s) => s.workflow);
  if (!workflow) return null;
  const present = selected.filter((id) => workflow.nodes.some((n) => n.id === id));
  const node = present.length === 1 ? workflow.nodes.find((n) => n.id === present[0]) : undefined;
  return (
    <ScrollArea className="h-full">
      <SwapFade id={node?.id ?? "project"}>
        {node ? (
          <NodeInspector workflow={workflow} nodeId={node.id} />
        ) : (
          <ProjectInspector workflow={workflow} multi={present.length} />
        )}
      </SwapFade>
    </ScrollArea>
  );
}

function NodeInspector({ workflow, nodeId }: { workflow: Workflow; nodeId: string }) {
  const node = workflow.nodes.find((n) => n.id === nodeId);
  if (!node) return null;
  const def = getNodeDef(node.type);
  if (!def) return <p className="p-4 text-sm">This node type isn't available.</p>;
  return <NodeDetails workflow={workflow} node={node} def={def} />;
}

function NodeDetails({
  workflow,
  node,
  def,
}: {
  workflow: Workflow;
  node: WorkflowNode;
  def: NodeDef;
}) {
  const ctx = useFormContext(workflow, node.id, def);
  const updateParams = useWorkflowStore((s) => s.updateParams);
  return (
    <div className="flex flex-col gap-4 p-4">
      <NodeHeader node={node} def={def} />
      {def.description ? <p className="text-muted-foreground text-sm">{def.description}</p> : null}
      <NodeProblems nodeId={node.id} />
      <NodeMetaFields node={node} def={def} />
      <Separator />
      <ZodForm
        schema={def.params}
        value={node.params}
        onChange={(patch) => updateParams(node.id, patch)}
        ctx={ctx}
      />
      <Separator />
      <NodeActions node={node} def={def} />
      <ArtifactList nodeId={node.id} />
    </div>
  );
}

function NodeHeader({ node, def }: { node: WorkflowNode; def: NodeDef }) {
  return (
    <div className="flex items-start gap-3">
      <CategoryBadge category={def.category} icon={def.icon} size="xl" />
      <div className="flex min-w-0 flex-col gap-1">
        <h2 className="truncate font-semibold">{nodeName(node)}</h2>
        <div className="flex flex-wrap gap-1">
          <Badge variant="secondary">{CATEGORY_LABELS[def.category]}</Badge>
          {node.label?.trim() ? <Badge variant="outline">{def.title}</Badge> : null}
        </div>
      </div>
    </div>
  );
}

function NodeProblems({ nodeId }: { nodeId: string }) {
  const { result, issues } = useCompiledContext();
  const runError = useRunStore((s) => s.errors[nodeId]);
  const problems = [
    runError,
    result?.errors[nodeId],
    ...issues.filter((i) => i.nodeId === nodeId).map((i) => i.message),
  ].filter((p): p is string => !!p);
  if (!problems.length) return null;
  return (
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
  );
}

function NodeMetaFields({ node, def }: { node: WorkflowNode; def: NodeDef }) {
  const setNodeMeta = useWorkflowStore((s) => s.setNodeMeta);
  return (
    <FieldGroup className="gap-4">
      <Field>
        <FieldLabel htmlFor="node-label">Name</FieldLabel>
        <Input
          id="node-label"
          value={node.label ?? ""}
          placeholder={def.title}
          onChange={(e) => setNodeMeta(node.id, { label: e.target.value || undefined })}
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
          onCheckedChange={(v) => setNodeMeta(node.id, { disabled: v || undefined })}
        />
      </Field>
    </FieldGroup>
  );
}

function NodeActions({ node, def }: { node: WorkflowNode; def: NodeDef }) {
  const running = useRunStore((s) => s.running);
  const nodeResult = useRunStore((s) => s.results[node.id]);
  const traceId = useRunStore((s) => s.traces[node.id]);
  const projectDir = useWorkflowStore((s) => s.projectDir);
  const setReplay = useUiStore((s) => s.setReplay);
  const { removeNodes, ungroup } = useWorkflowStore.getState();
  return (
    <div className="flex flex-wrap gap-2">
      {def.executes || def.virtual ? (
        <>
          <Button
            size="sm"
            disabled={running}
            onClick={() => void startRun({ targetNodeIds: [node.id] })}
          >
            <Play data-icon="inline-start" />
            Run this
          </Button>
          <Button
            size="sm"
            variant="outline"
            disabled={running}
            onClick={() => void startRun({ fromNodeId: node.id })}
          >
            <RotateCcw data-icon="inline-start" />
            Re-run from here
          </Button>
        </>
      ) : null}
      {def.category === "Capture" && (nodeResult || traceId) ? (
        <Button size="sm" variant="outline" onClick={() => setReplay(node.id)}>
          <Film data-icon="inline-start" />
          Replay
        </Button>
      ) : null}
      {traceId && projectDir ? (
        <Button size="sm" variant="outline" onClick={() => setReplay(node.id, "trace")}>
          <FileSearch data-icon="inline-start" />
          Trace
        </Button>
      ) : null}
      {node.type === "core.group" ? (
        <Button size="sm" variant="outline" onClick={() => ungroup(node.id)}>
          <Ungroup data-icon="inline-start" />
          Ungroup
        </Button>
      ) : null}
      <Button
        size="sm"
        variant="ghost"
        onClick={() => {
          useUiStore.getState().setSelected([]);
          removeNodes([node.id]);
        }}
      >
        <Trash2 data-icon="inline-start" />
        Delete
      </Button>
    </div>
  );
}

function ArtifactList({ nodeId }: { nodeId: string }) {
  const nodeResult = useRunStore((s) => s.results[nodeId]);
  const projectDir = useWorkflowStore((s) => s.projectDir);
  if (!nodeResult?.artifacts.length || !projectDir) return null;
  return (
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
  );
}
