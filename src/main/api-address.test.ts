import { describe, expect, it } from "vitest";
import { API_SOCKET_ENV, MAX_SOCKET_PATH_BYTES, apiAddress } from "./api-address";

const host = (
  env: Record<string, string | undefined> = {},
  userData = "/home/ada/.config/Demoflow",
) => ({
  platform: "linux" as const,
  userData,
  home: "/home/ada",
  username: "ada",
  env,
});

const pathOfBytes = (bytes: number) => `/${"a".repeat(bytes - "/api.sock".length - 1)}/api.sock`;

describe("api address", () => {
  it("uses the socket in the app's data folder by default", () => {
    expect(apiAddress(host())).toEqual({ address: "/home/ada/.config/Demoflow/api.sock" });
  });

  it("uses the path the environment variable names", () => {
    expect(apiAddress(host({ [API_SOCKET_ENV]: " /run/user/1000/df.sock " }))).toEqual({
      address: "/run/user/1000/df.sock",
    });
  });

  it("allows exactly the maximum length and refuses one byte more", () => {
    const fits = pathOfBytes(MAX_SOCKET_PATH_BYTES);
    const over = pathOfBytes(MAX_SOCKET_PATH_BYTES + 1);
    expect(apiAddress(host({ [API_SOCKET_ENV]: fits })).error).toBeUndefined();
    expect(apiAddress(host({ [API_SOCKET_ENV]: over })).error).toContain(
      `101 bytes long, and it can be at most ${MAX_SOCKET_PATH_BYTES}`,
    );
  });

  it("counts a long default path and says how to fix it", () => {
    const { error } = apiAddress(host({}, `/home/ada/${"deep/".repeat(20)}Demoflow`));
    expect(error).toContain(
      `Set ${API_SOCKET_ENV} to a shorter full path, like /home/ada/.demoflow-api.sock`,
    );
  });

  it("counts bytes, not characters", () => {
    const address = `/home/ada/${"é".repeat(45)}/api.sock`;
    expect(address.length).toBeLessThanOrEqual(MAX_SOCKET_PATH_BYTES);
    expect(apiAddress(host({ [API_SOCKET_ENV]: address })).error).toContain("bytes long");
  });

  it("refuses a relative path", () => {
    expect(apiAddress(host({ [API_SOCKET_ENV]: "df.sock" })).error).toContain(
      "has to be a full path",
    );
  });

  it("uses the named pipe on Windows and ignores the variable", () => {
    expect(apiAddress({ ...host({ [API_SOCKET_ENV]: "/x.sock" }), platform: "win32" })).toEqual({
      address: "\\\\.\\pipe\\demoflow-api-ada",
    });
  });
});
