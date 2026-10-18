export const VIDEO_MIME: Readonly<Record<string, string>> = {
  mp4: "video/mp4",
  m4v: "video/mp4",
  mov: "video/quicktime",
  webm: "video/webm",
  mkv: "video/x-matroska",
};

export const IMAGE_MIME: Readonly<Record<string, string>> = {
  png: "image/png",
  jpg: "image/jpeg",
  jpeg: "image/jpeg",
  webp: "image/webp",
  gif: "image/gif",
  svg: "image/svg+xml",
};

export const IMAGE_EXTENSIONS: ReadonlySet<string> = new Set(Object.keys(IMAGE_MIME));

export const AUDIO_MIME: Readonly<Record<string, string>> = {
  mp3: "audio/mpeg",
  m4a: "audio/mp4",
  aac: "audio/aac",
  wav: "audio/wav",
  ogg: "audio/ogg",
  flac: "audio/flac",
};

export const AUDIO_EXTENSIONS: ReadonlySet<string> = new Set(Object.keys(AUDIO_MIME));
