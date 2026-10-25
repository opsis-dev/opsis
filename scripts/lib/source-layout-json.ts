import { writeFileSync } from "node:fs";
import path from "node:path";
import {
  DEPENDENCY_MANIFESTS,
  EXPORTED_PACKAGES,
  EXPORTED_SOURCES,
  packageManifest,
  VIDEO_SOURCE,
} from "@opsis/sidecar/source-layout";
import { ROOT } from "./tool-config";

export const SOURCE_LAYOUT_JSON = path.join(
  ROOT,
  "crates",
  "render",
  "src",
  "source-layout.generated.json",
);

export const sourceLayoutJsonText = (): string =>
  `${JSON.stringify(
    {
      videoSource: VIDEO_SOURCE,
      exportedSources: EXPORTED_SOURCES,
      exportedPackages: Object.entries(EXPORTED_PACKAGES).map(([name, folder]) => ({
        name,
        folder,
        manifest: packageManifest(folder),
      })),
      dependencyManifests: DEPENDENCY_MANIFESTS,
    },
    null,
    1,
  )}\n`;

export function writeSourceLayoutJson(): void {
  writeFileSync(SOURCE_LAYOUT_JSON, sourceLayoutJsonText());
  console.log(`wrote ${path.relative(ROOT, SOURCE_LAYOUT_JSON)}`);
}
