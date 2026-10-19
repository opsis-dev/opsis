import { chmod, lstat, rm } from "node:fs/promises";
import { createServer, type Server } from "node:http";
import { connect } from "node:net";
import { homedir, userInfo } from "node:os";
import path from "node:path";
import { app } from "electron";
import { errorCode, errorMessage } from "../shared/errors";
import { IPC, type ApiServerStatus } from "../shared/ipc";
import { API_SOCKET_ENV, apiAddress } from "./api-address";
import { endRunEventStreams } from "./api-editor";
import { handleApiRequest } from "./api-routes";
import { broadcast } from "./app-windows";
import { onSettingsChange, readSettings } from "./settings";

const OWNER_ONLY_UMASK = 0o077;
const OWNER_ONLY_MODE = 0o600;
const USES_SOCKET_FILE = process.platform !== "win32";

const resolveAddress = () =>
  apiAddress({
    platform: process.platform,
    userData: app.getPath("userData"),
    home: homedir(),
    username: userInfo().username,
    env: process.env,
  });

let server: Server | null = null;
let status: ApiServerStatus | null = null;

export function getApiStatus(): ApiServerStatus {
  if (status === null)
    status = { enabled: false, listening: false, address: resolveAddress().address };
  return status;
}

function setStatus(next: ApiServerStatus): void {
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
  if (!USES_SOCKET_FILE) return;
  const stats = await lstat(address).catch(() => null);
  if (stats?.isSocket()) await rm(address, { force: true });
}

function listenError(e: unknown, address: string): string {
  if (!USES_SOCKET_FILE) return errorMessage(e);
  const folder = path.dirname(address);
  switch (errorCode(e)) {
    case "ENOENT":
      return `The folder ${folder} doesn't exist. Create it, or set ${API_SOCKET_ENV} to a path in a folder that does, and restart Opsis.`;
    case "EACCES":
      return `Opsis can't create files in ${folder}. Set ${API_SOCKET_ENV} to a path in a folder you own, and restart Opsis.`;
    case "EADDRINUSE":
      return `A file that isn't Opsis's API is already at ${address}. Move it, or set ${API_SOCKET_ENV} to another path, and restart Opsis.`;
    default:
      return errorMessage(e);
  }
}

function listenOwnerOnly(next: Server, address: string): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    next.once("error", reject);
    const umask = USES_SOCKET_FILE ? process.umask(OWNER_ONLY_UMASK) : null;
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

async function setApiEnabled(enabled: boolean): Promise<void> {
  const { address, error } = resolveAddress();
  if (!enabled) {
    await stopApi();
    setStatus({ enabled: false, listening: false, address });
    return;
  }
  if (server) return;
  if (error) {
    setStatus({ enabled: true, listening: false, address, error });
    return;
  }
  if (await inUse(address)) {
    setStatus({
      enabled: true,
      listening: false,
      address,
      error: "Another copy of Opsis is already using this address.",
    });
    return;
  }
  await removeSocket(address);
  const next = createServer((req, res) => void handleApiRequest(req, res));
  try {
    await listenOwnerOnly(next, address);
    if (USES_SOCKET_FILE) await chmod(address, OWNER_ONLY_MODE);
    server = next;
    setStatus({ enabled: true, listening: true, address });
  } catch (e) {
    next.close();
    setStatus({
      enabled: true,
      listening: false,
      address,
      error: listenError(e, address),
    });
  }
}

export function followApiSetting(): void {
  onSettingsChange((settings, previous) => {
    if (settings.api.enabled !== previous.api.enabled) void setApiEnabled(settings.api.enabled);
  });
  void setApiEnabled(readSettings().api.enabled);
}

export async function stopApi(): Promise<void> {
  const current = server;
  server = null;
  endRunEventStreams();
  if (!current) return;
  const closed = new Promise<void>((resolve) => current.close(() => resolve()));
  current.closeAllConnections();
  await closed;
  await removeSocket(resolveAddress().address);
}
