import path from "node:path";
import { bundleRemotionProject, getPrebuiltRemotionBundlePath } from "@opsis/sidecar/bundle";
import { ROOT } from "./lib/tool-config";

const outDir = getPrebuiltRemotionBundlePath(ROOT);
let shown = -1;
await bundleRemotionProject({
  projectRoot: ROOT,
  outDir,
  onProgress: (percent) => {
    const step = Math.floor(percent / 10) * 10;
    if (step > shown) console.log(`${(shown = step)}%`);
  },
});
console.log(`wrote ${path.relative(ROOT, outDir)}`);
