type FontRole = "main" | "heading" | "code";

function roleStack(role: FontRole): string {
  const fallback = role === "code" ? "ui-monospace, monospace" : "system-ui, sans-serif";
  return `"df-${role}", ${fallback}`;
}

export function fontOr(family: string | undefined, role: FontRole): string {
  return family || roleStack(role);
}

export const MAIN_FONT = roleStack("main");

export const CODE_FONT = roleStack("code");
