type FontRole = "main" | "heading" | "code";

function roleStack(role: FontRole): string {
  return `"df-${role}"`;
}

export function fontOr(family: string | undefined, role: FontRole): string {
  return family || roleStack(role);
}

export const MAIN_FONT = roleStack("main");

export const CODE_FONT = roleStack("code");
