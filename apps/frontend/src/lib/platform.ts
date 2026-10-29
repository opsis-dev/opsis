export type Platform = "darwin" | "win32" | "linux";

function platformOf(agent: string): Platform {
  if (/Mac/i.test(agent)) return "darwin";
  if (/Windows/i.test(agent)) return "win32";
  return "linux";
}

export const platform = platformOf(navigator.userAgent);
export const isMac = platform === "darwin";
export const isWindows = platform === "win32";
