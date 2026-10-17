import { chmod, rm } from "node:fs/promises";
import { createServer, type Server } from "node:http";
import { connect } from "node:net";
import { userInfo } from "node:os";
import path from "node:path";
import { app } from "electron";
import { IPC, type ApiStatus } from "../shared/ipc";
import { errorMessage } from "../shared/errors";
import { endRunEventStreams } from "./api-editor";
import { handleApiRequest } from "./api-routes";
import { broadcast } from "./app-windows";
import { onSettingsChange, readSettings } from "./settings";

const OWNER_ONLY_UMASK = 0o077;

const apiAddress = () =>
  process.platform === "win32"
    ? `\\\\.\\pipe\\demoflow-api-${userInfo().username}`
    : path.join(app.getPath("userData"), "api.sock");

let server: Server | null = null;
let status: ApiStatus | null = null;

export const getApiStatus = (): ApiStatus =>
  (status ??= { enabled: false, listening: false, address: apiAddress() });

function setStatus(next: ApiStatus): void {
  status = next;
  broadcast(IPC.apiStatusChanged, next);
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

async function removeSocket(address: string): Promise<void> {
  if (process.platform !== "win32") await rm(address, { force: true });
}

function listenOwnerOnly(next: Server, address: string): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    next.once("error", reject);
    const umask = process.platform === "win32" ? null : process.umask(OWNER_ONLY_UMASK);
    try {
      next.listen(address, () => {
        next.off("error", reject);
        resolve();
      });
    } finally {
      if (umask !== null) process.umask(umask);
    }
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
  await removeSocket(address);
  const next = createServer((req, res) => void handleApiRequest(req, res));
  try {
    await listenOwnerOnly(next, address);
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

export function followApiSetting(): void {
  onSettingsChange((settings, previous) => {
    if (settings.api.enabled !== previous.api.enabled) void setApiEnabled(settings.api.enabled);
  });
  void readSettings().then((settings) => setApiEnabled(settings.api.enabled));
}

async function stopServer(): Promise<void> {
  const current = server;
  server = null;
  endRunEventStreams();
  if (!current) return;
  const closed = new Promise<void>((resolve) => current.close(() => resolve()));
  current.closeAllConnections();
  await closed;
  await removeSocket(apiAddress());
}

export function stopApi(): void {
  void stopServer();
}
