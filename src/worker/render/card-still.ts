import { mkdir, rename } from "node:fs/promises";
import path from "node:path";
import type { HeadlessBrowser } from "@remotion/renderer";
import type { MainToWorker } from "../../shared/worker-protocol";
import { startAssetServer, type AssetServer } from "./asset-server";
import { getServeUrl, openRenderBrowser, renderFrame } from "./render";

type StillRequest = Extract<MainToWorker, { type: "still" }>;

let browser: Promise<HeadlessBrowser> | null = null;
const servers = new Map<string, Promise<AssetServer>>();

function serverFor(projectDir: string): Promise<AssetServer> {
  let server = servers.get(projectDir);
  if (!server) {
    server = startAssetServer(projectDir).catch((error: unknown) => {
      servers.delete(projectDir);
      throw error;
    });
    servers.set(projectDir, server);
  }
  return server;
}

function renderBrowser(env: StillRequest["env"]): Promise<HeadlessBrowser> {
  browser ??= openRenderBrowser(env, "angle").catch((error: unknown) => {
    browser = null;
    throw error;
  });
  return browser;
}

export async function renderCardStill(msg: StillRequest, signal: AbortSignal): Promise<void> {
  const serveUrl = await getServeUrl(msg.env);
  const server = await serverFor(msg.projectDir);
  const instance = await renderBrowser(msg.env);
  await mkdir(path.dirname(msg.output), { recursive: true });
  const tmp = `${msg.output}.partial.jpg`;
  await renderFrame({
    env: msg.env,
    serveUrl,
    spec: { ...msg.spec, assets: { projectBase: server.url } },
    frame: msg.frame,
    output: tmp,
    format: "jpeg",
    gl: "angle",
    signal,
    browser: instance,
    scale: Math.min(1, msg.width / msg.spec.width),
    jpegQuality: 82,
  });
  await rename(tmp, msg.output);
}
