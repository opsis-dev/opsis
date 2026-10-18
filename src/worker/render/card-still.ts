import { mkdir } from "node:fs/promises";
import path from "node:path";
import type { HeadlessBrowser } from "@remotion/renderer";
import { writeViaTemp } from "../../engine/atomic";
import { keepLatest } from "../../shared/memo";
import type { MainMessage } from "../../shared/worker-protocol";
import { startAssetServer, type AssetServer } from "./asset-server";
import { getServeUrl, openRenderBrowser, renderFrame, type GlRenderer } from "./render";

type StillRequest = MainMessage<"still">;

const CARD_GL: GlRenderer = "angle";
const CARD_JPEG_QUALITY = 82;

const projectServer = keepLatest<AssetServer>((server) => server.close());
const cardBrowser = keepLatest<HeadlessBrowser>();

export async function renderCardStill(msg: StillRequest, signal: AbortSignal): Promise<void> {
  const serveUrl = await getServeUrl(msg.env);
  const server = await projectServer(msg.projectDir, () => startAssetServer(msg.projectDir));
  const browser = await cardBrowser(CARD_GL, () => openRenderBrowser(msg.env, CARD_GL));
  await mkdir(path.dirname(msg.output), { recursive: true });
  await writeViaTemp(msg.output, (temp) =>
    renderFrame({
      env: msg.env,
      serveUrl,
      spec: { ...msg.spec, assets: { projectBase: server.url } },
      frame: msg.frame,
      output: temp,
      format: "jpeg",
      gl: CARD_GL,
      signal,
      browser,
      scale: Math.min(1, msg.width / msg.spec.width),
      jpegQuality: CARD_JPEG_QUALITY,
    }),
  );
}
