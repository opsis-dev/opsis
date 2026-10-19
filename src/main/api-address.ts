import path from "node:path";

export const API_SOCKET_ENV = "DEMOFLOW_API_SOCKET";
export const MAX_SOCKET_PATH_BYTES = 100;

type ApiAddress = { address: string; error?: string };

type Host = {
  platform: NodeJS.Platform;
  userData: string;
  home: string;
  username: string;
  env: Record<string, string | undefined>;
};

const exampleSocket = (home: string) => path.join(home, ".demoflow-api.sock");

export function apiAddress({ platform, userData, home, username, env }: Host): ApiAddress {
  if (platform === "win32") return { address: `\\\\.\\pipe\\demoflow-api-${username}` };
  const custom = env[API_SOCKET_ENV]?.trim();
  const address = custom || path.join(userData, "api.sock");
  if (custom && !path.isAbsolute(custom))
    return {
      address,
      error: `${API_SOCKET_ENV} has to be a full path, like ${exampleSocket(home)}. Change it and restart Demoflow.`,
    };
  const bytes = Buffer.byteLength(address);
  if (bytes > MAX_SOCKET_PATH_BYTES)
    return {
      address,
      error: `The API address is ${bytes} bytes long, and it can be at most ${MAX_SOCKET_PATH_BYTES}. Set ${API_SOCKET_ENV} to a shorter full path, like ${exampleSocket(home)}, and restart Demoflow.`,
    };
  return { address };
}
