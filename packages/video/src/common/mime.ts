export function extensionOf(address: string): string {
  const path = address.split(/[?#]/, 1)[0];
  const name = path.slice(path.lastIndexOf("/") + 1);
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(dot + 1).toLowerCase() : "";
}

export const IMAGE_MIME: Readonly<Record<string, string>> = {
  png: "image/png",
  jpg: "image/jpeg",
  jpeg: "image/jpeg",
  webp: "image/webp",
  gif: "image/gif",
  svg: "image/svg+xml",
};

export const IMAGE_EXTENSIONS: ReadonlySet<string> = new Set(Object.keys(IMAGE_MIME));

const AUDIO_MIME: Readonly<Record<string, string>> = {
  mp3: "audio/mpeg",
  m4a: "audio/mp4",
  aac: "audio/aac",
  wav: "audio/wav",
  ogg: "audio/ogg",
  flac: "audio/flac",
  opus: "audio/ogg",
  oga: "audio/ogg",
  weba: "audio/webm",
  aif: "audio/aiff",
  aiff: "audio/aiff",
  caf: "audio/x-caf",
  wma: "audio/x-ms-wma",
  mka: "audio/x-matroska",
};

export const AUDIO_EXTENSIONS: ReadonlySet<string> = new Set(Object.keys(AUDIO_MIME));

export function kindFromPath(path: string): "video" | "audio" | "image" {
  const ext = extensionOf(path);
  if (IMAGE_EXTENSIONS.has(ext)) return "image";
  return AUDIO_EXTENSIONS.has(ext) ? "audio" : "video";
}
