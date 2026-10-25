import { exportedArtifacts } from "@opsis/core/run-progress";
import { artifactName, type Artifact } from "@opsis/core/schema/values";
import type { WorkflowNode } from "@opsis/core/schema/workflow";
import { FolderOpen } from "lucide-react";
import { useMemo } from "react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useRunStore } from "../store/run";
import { AudioFilePlayer, VideoFilePlayer } from "./file-player";
import { fileUrl } from "./file-url";
import { ImagePreview } from "./image-preview";
import { ReplayButtons } from "./replay-buttons";
import { revealOutputs, revealProjectFile } from "./reveal";

function ArtifactPreview({ artifact }: { artifact: Artifact }) {
  if (artifact.kind === "video")
    return (
      <VideoFilePlayer
        src={fileUrl(artifact.path)}
        label={artifactName(artifact)}
        muted
        className="rounded-md"
        videoClassName="aspect-video"
      />
    );
  if (artifact.kind === "image")
    return <ImagePreview src={fileUrl(artifact.path)} alt={artifactName(artifact)} />;
  if (artifact.kind === "audio")
    return <AudioFilePlayer src={fileUrl(artifact.path)} label={artifactName(artifact)} />;
  return null;
}

export function RunOutputs({
  nodes,
  filterNode,
  projectDir,
}: {
  nodes: WorkflowNode[];
  filterNode: WorkflowNode | undefined;
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
