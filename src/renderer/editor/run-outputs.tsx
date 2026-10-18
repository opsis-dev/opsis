import { FolderOpen } from "lucide-react";
import { useMemo } from "react";
import { Button } from "@/renderer/components/ui/button";
import { ScrollArea } from "@/renderer/components/ui/scroll-area";
import { exportedArtifacts } from "@/shared/run-progress";
import { artifactName, type Artifact } from "@/shared/schema/values";
import type { WorkflowNode } from "@/shared/schema/workflow";
import { useRunStore } from "../store/run";
import { fileUrl } from "./file-url";
import { ReplayButtons } from "./replay-buttons";
import { revealOutputs, revealProjectFile } from "./reveal";

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
        alt={artifactName(artifact)}
        className="w-full rounded-md"
      />
    );
  if (artifact.kind === "audio")
    return <audio src={fileUrl(artifact.path)} controls className="w-full" />;
  return null;
}

export function RunOutputs({
  nodes,
  filterNode,
  projectDir,
}: {
  nodes: WorkflowNode[];
  filterNode: WorkflowNode | null;
  projectDir: string;
}) {
  const results = useRunStore((s) => s.results);
  const exported = useMemo(() => exportedArtifacts(nodes, results), [nodes, results]);
  const artifacts = filterNode ? (results[filterNode.id]?.artifacts ?? []) : exported;

  return (
    <ScrollArea className="h-full">
      <div className="flex flex-col gap-3 p-3">
        <div className="flex items-center justify-between">
          <span className="text-sm font-medium">{filterNode ? "Artifacts" : "Outputs"}</span>
          <Button size="xs" variant="outline" onClick={() => void revealOutputs(projectDir)}>
            <FolderOpen data-icon="inline-start" />
            Output folder
          </Button>
        </div>
        {filterNode ? (
          <div className="flex flex-wrap gap-2 empty:hidden">
            <ReplayButtons nodeId={filterNode.id} nodeType={filterNode.type} size="xs" />
          </div>
        ) : null}
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
              onClick={() => void revealProjectFile(projectDir, a.path)}
            >
              {artifactName(a)}
            </Button>
          </div>
        ))}
      </div>
    </ScrollArea>
  );
}
