export const fileUrl = (ref: string) =>
  `demoflow-file://project/${ref
    .replace(/^project:/, "")
    .split("/")
    .map(encodeURIComponent)
    .join("/")}`;
