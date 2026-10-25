import { isGroupNode } from "@opsis/core/graph";
import { findById } from "@opsis/core/lists";
import { CATEGORY_LABELS, type NodeDef } from "@opsis/core/nodes/define";
import { customLabel, getNodeDef, nodeName } from "@opsis/core/nodes/registry";
import { artifactName } from "@opsis/core/schema/values";
import type { Workflow, WorkflowNode } from "@opsis/core/schema/workflow";
import { CircleAlert, ExternalLink, Play, RotateCcw, Trash2, Ungroup } from "lucide-react";
import type { ReactNode } from "react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
  FieldLegend,
  FieldSet,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { SwapFade } from "../app/motion";
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { NodeBadge } from "./category-badge";
import { ProjectInspector } from "./project-inspector";
import { ReplayButtons } from "./replay-buttons";
import { startRun } from "./run";
import { useFormContext } from "./use-form-context";
import { useNodeProblems } from "./use-node-problems";
import { useTypingAsOneUndoStep } from "./use-typing-as-one-undo-step";
import { ZodForm } from "./zod-form";

export function Inspector() {
  const selected = useUiStore((s) => s.selectedIds);
  const workflow = useWorkflowStore((s) => s.workflow);
  const typing = useTypingAsOneUndoStep();
  if (!workflow) return null;
  const present = selected.flatMap((id) => findById(workflow.nodes, id) ?? []);
  const node = present.length === 1 ? present[0] : undefined;
  const def = node && getNodeDef(node.type);
  let body: ReactNode;
  if (!node) body = <ProjectInspector workflow={workflow} multi={present.length} />;
  else if (def) body = <NodeDetails workflow={workflow} node={node} def={def} />;
  else body = <p className="p-4 text-sm">This node type isn't available.</p>;
  return (
    <ScrollArea className="h-full" {...typing}>
      <SwapFade id={node?.id ?? "project"}>{body}</SwapFade>
    </ScrollArea>
  );
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
      <NodeBadge def={def} size="xl" />
      <div className="flex min-w-0 flex-col gap-1">
        <h2 className="truncate font-semibold">{nodeName(node)}</h2>
        <div className="flex flex-wrap gap-1">
          <Badge variant="secondary">{CATEGORY_LABELS[def.category]}</Badge>
          {customLabel(node) ? <Badge variant="outline">{def.title}</Badge> : null}
        </div>
      </div>
    </div>
  );
}

function NodeProblems({ nodeId }: { nodeId: string }) {
  const { errors, warnings } = useNodeProblems(nodeId);
  return (
    <>
      <ProblemAlert
        problems={errors}
        variant="destructive"
        title={(n) => (n === 1 ? "Something needs attention" : `${n} things need attention`)}
      />
      <ProblemAlert
        problems={warnings}
        variant="default"
        title={(n) => (n === 1 ? "Something to check" : `${n} things to check`)}
      />
    </>
  );
}

function ProblemAlert({
  problems,
  variant,
  title,
}: {
  problems: string[];
  variant: "default" | "destructive";
  title: (count: number) => string;
}) {
  if (!problems.length) return null;
  return (
    <Alert variant={variant}>
      <CircleAlert />
      <AlertTitle>{title(problems.length)}</AlertTitle>
      <AlertDescription>
        {problems.map((p) => (
          <p key={p} className="whitespace-pre-wrap">
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
  const removeNodes = useWorkflowStore((s) => s.removeNodes);
  const ungroup = useWorkflowStore((s) => s.ungroup);
  const setSelected = useUiStore((s) => s.setSelected);
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
      <ReplayButtons nodeId={node.id} nodeType={node.type} size="sm" />
      {isGroupNode(node) ? (
        <Button size="sm" variant="outline" onClick={() => ungroup(node.id)}>
          <Ungroup data-icon="inline-start" />
          Ungroup
        </Button>
      ) : null}
      <Button
        size="sm"
        variant="ghost"
        onClick={() => {
          setSelected([]);
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
          onClick={() => void window.opsis.revealInFolder(projectDir, a.path)}
        >
          <ExternalLink data-icon="inline-start" />
          {artifactName(a)}
        </Button>
      ))}
    </FieldSet>
  );
}
