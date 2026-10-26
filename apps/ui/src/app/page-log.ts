import { commands, type PageLogLevel } from "@/generated/bindings";

const describe = (value: unknown): string =>
  value instanceof Error ? (value.stack ?? `${value.name}: ${value.message}`) : String(value);

const send = (level: PageLogLevel, parts: unknown[]) =>
  void commands.pageLog(level, parts.map(describe).join(" ")).catch(() => undefined);

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
