import { existsSync } from "node:fs";
import { chmod, rm } from "node:fs/promises";
import { createServer, type Server } from "node:http";
import { connect } from "node:net";
import { userInfo } from "node:os";
import path from "node:path";
import { app, type BrowserWindow } from "electron";
import type { ApiStatus } from "../shared/ipc";
import { errorMessage } from "../shared/errors";
import { endRunEventStreams, initApiEditor } from "./api-editor";
import { handleApiRequest } from "./api-routes";

type Host = {
  editor: () => BrowserWindow | null;
  onStatus: (status: ApiStatus) => void;
};

const apiAddress = () =>
  process.platform === "win32"
    ? `\\\\.\\pipe\\demoflow-api-${userInfo().username}`
    : path.join(app.getPath("userData"), "api.sock");

let host: Host | null = null;
let server: Server | null = null;
let status: ApiStatus = { enabled: false, listening: false, address: "" };

export function initApi(next: Host, isEditorSender: (id: number) => boolean): void {
  host = next;
  status = { ...status, address: apiAddress() };
  initApiEditor(next.editor, isEditorSender);
}

export const getApiStatus = (): ApiStatus => status;

function setStatus(next: ApiStatus): void {
  status = next;
  host?.onStatus(status);
}

function inUse(address: string): Promise<boolean> {
  return new Promise((resolve) => {
    const socket = connect(address);
    socket.once("connect", () => {
      socket.destroy();
      resolve(true);
    });
    socket.once("error", () => resolve(false));
  });
}

export async function setApiEnabled(enabled: boolean): Promise<void> {
  const address = apiAddress();
  if (!enabled) {
    await stopServer();
    setStatus({ enabled: false, listening: false, address });
    return;
  }
  if (server) return;
  if (await inUse(address)) {
    setStatus({
      enabled: true,
      listening: false,
      address,
      error: "Another copy of Demoflow is already using this address.",
    });
    return;
  }
  if (process.platform !== "win32" && existsSync(address)) await rm(address, { force: true });
  const next = createServer((req, res) => void handleApiRequest(req, res));
  try {
    await new Promise<void>((resolve, reject) => {
      next.once("error", reject);
      next.listen(address, () => {
        next.off("error", reject);
        resolve();
      });
    });
    if (process.platform !== "win32") await chmod(address, 0o600);
    server = next;
    setStatus({ enabled: true, listening: true, address });
  } catch (e) {
    next.close();
    setStatus({
      enabled: true,
      listening: false,
      address,
      error: errorMessage(e),
    });
  }
}

async function stopServer(): Promise<void> {
  const current = server;
  server = null;
  endRunEventStreams();
  if (!current) return;
  await new Promise<void>((resolve) => current.close(() => resolve()));
  current.closeAllConnections();
  if (process.platform !== "win32") await rm(apiAddress(), { force: true });
}

export function stopApi(): void {
  void stopServer();
}
