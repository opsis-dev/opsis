import { z } from "zod";

const PACKAGES: Partial<Record<string, string>> = {
  "darwin-arm64": "@remotion/compositor-darwin-arm64",
  "darwin-x64": "@remotion/compositor-darwin-x64",
  "linux-x64-gnu": "@remotion/compositor-linux-x64-gnu",
  "linux-x64-musl": "@remotion/compositor-linux-x64-musl",
  "linux-arm64-gnu": "@remotion/compositor-linux-arm64-gnu",
  "linux-arm64-musl": "@remotion/compositor-linux-arm64-musl",
  "win32-x64": "@remotion/compositor-win32-x64-msvc",
};

const reportSchema = z.object({
  header: z.object({ glibcVersionRuntime: z.string().optional() }),
});

function linuxLibc(): "gnu" | "musl" {
  const report = reportSchema.safeParse(process.report.getReport());
  return report.success && report.data.header.glibcVersionRuntime ? "gnu" : "musl";
}

export function getCompositorPackage({
  arch,
  platform,
}: {
  arch: string;
  platform: string;
}): string {
  const key = platform === "linux" ? `${platform}-${arch}-${linuxLibc()}` : `${platform}-${arch}`;
  const pkg = PACKAGES[key];
  if (!pkg) throw new Error(`Unsupported platform/arch combination: ${platform}/${arch}`);
  return pkg;
}
