export const ndjson = (events: object[]): string => events.map((e) => JSON.stringify(e)).join("\n");

export const FRAME = "screencast/page@1-1000.jpeg";

export const LATER = "screencast/page@1-2000.jpeg";
