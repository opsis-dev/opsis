import { FileSearch, Film, FolderOpen } from "lucide-react";
import { useMemo } from "react";
import { getNodeDef } from "@/shared/nodes/registry";
import { exportedArtifacts } from "@/shared/run-progress";
import type { Artifact } from "@/shared/schema/values";
import type { WorkflowNode } from "@/shared/schema/workflow";
import { Button } from "@/renderer/components/ui/button";
import { ScrollArea } from "@/renderer/components/ui/scroll-area";
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";
import { fileUrl } from "./file-url";
import { revealOutputs } from "./reveal";

function ArtifactPreview({ artifact }: { artifact: Artifact }) {
  if (artifact.kind === "video")
    return (
      <video
        src={fileUrl(artifact.path)}
        controls
        muted
        className="aspect-video w-full rounded-md bg-black"
      />
    );
  if (artifact.kind === "image")
    return (
      <img
        src={fileUrl(artifact.path)}
        alt={artifact.label ?? artifact.path}
        className="w-full rounded-md"
      />
    );
  if (artifact.kind === "audio")
    return <audio src={fileUrl(artifact.path)} controls className="w-full" />;
  return null;
}

function ReplayButtons({ node, projectDir }: { node: WorkflowNode; projectDir: string | null }) {
  const hasResult = useRunStore((s) => !!s.results[node.id]);
  const hasTrace = useRunStore((s) => !!s.traces[node.id]);
  const setReplay = useUiStore((s) => s.setReplay);
  if (!hasResult && !hasTrace) return null;
  const capture = getNodeDef(node.type)?.category === "Capture";
  return (
    <div className="flex flex-wrap gap-2">
      {capture ? (
        <Button size="xs" variant="outline" onClick={() => setReplay(node.id)}>
          <Film data-icon="inline-start" />
          Replay
        </Button>
      ) : null}
      {hasTrace && projectDir ? (
        <Button size="xs" variant="outline" onClick={() => setReplay(node.id, "trace")}>
          <FileSearch data-icon="inline-start" />
          Trace
        </Button>
      ) : null}
    </div>
  );
}

export function RunOutputs({
  nodes,
  filterNode,
  projectDir,
}: {
  nodes: WorkflowNode[];
  filterNode: WorkflowNode | null;
  projectDir: string | null;
}) {
  const results = useRunStore((s) => s.results);
  const exported = useMemo(() => exportedArtifacts(nodes, results), [nodes, results]);
  const artifacts = filterNode ? (results[filterNode.id]?.artifacts ?? []) : exported;

  return (
    <ScrollArea className="h-full">
      <div className="flex flex-col gap-3 p-3">
        <div className="flex items-center justify-between">
          <span className="text-sm font-medium">{filterNode ? "Artifacts" : "Outputs"}</span>
          {projectDir ? (
            <Button size="xs" variant="outline" onClick={() => void revealOutputs(projectDir)}>
              <FolderOpen data-icon="inline-start" />
              Output folder
            </Button>
          ) : null}
        </div>
        {filterNode ? <ReplayButtons node={filterNode} projectDir={projectDir} /> : null}
        {artifacts.length === 0 ? (
          <p className="text-muted-foreground text-xs">Nothing yet.</p>
        ) : null}
        {artifacts.map((a) => (
          <div key={a.path} className="flex flex-col gap-1.5">
            <ArtifactPreview artifact={a} />
            <Button
              variant="link"
              size="xs"
              className="h-auto justify-start p-0"
              onClick={() => projectDir && void window.demoflow.revealInFolder(projectDir, a.path)}
            >
              {a.label ?? a.path}
            </Button>
          </div>
        ))}
      </div>
    </ScrollArea>
  );
}
