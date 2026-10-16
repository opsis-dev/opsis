import { encodePath, projectRelPath } from "@/shared/asset-ref";

export const fileUrl = (ref: string) =>
  ref.startsWith("app:")
    ? new URL(ref.slice("app:".length), document.baseURI).href
    : `demoflow-file://project/${encodePath(projectRelPath(ref))}`;
