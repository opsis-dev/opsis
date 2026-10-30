import { commands, type PageLogLevel } from "@/generated/bindings";

function describe(value: unknown): string {
  return value instanceof Error
    ? (value.stack ?? `${value.name}: ${value.message}`)
    : String(value);
}

function send(level: PageLogLevel, parts: unknown[]) {
  return void commands.pageLog(level, parts.map(describe).join(" ")).catch(() => undefined);
}

export function forwardConsole(): void {
  const error = console.error.bind(console);
  const warn = console.warn.bind(console);
  console.error = (...parts: unknown[]) => {
    error(...parts);
    send("error", parts);
  };
  console.warn = (...parts: unknown[]) => {
    warn(...parts);
    send("warn", parts);
  };
}
