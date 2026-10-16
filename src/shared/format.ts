const UNITS = ["B", "KB", "MB", "GB"];

export function formatBytes(n: number): string {
  if (!Number.isFinite(n) || n < 0) return "";
  let value = n;
  let unit = 0;
  while (value >= 1024 && unit < UNITS.length - 1) {
    value /= 1024;
    unit++;
  }
  return unit === 0 ? `${Math.round(value)} B` : `${value.toFixed(1)} ${UNITS[unit]}`;
}

export type DurationStyle = "short" | "precise";

export function formatDuration(ms: number, style: DurationStyle = "short"): string {
  if (!Number.isFinite(ms)) return "";
  const precise = style === "precise";
  const toMs = (v: number) => Number(v.toFixed(3));
  const s = precise ? toMs(ms / 1000) : ms / 1000;
  if (s >= 60) {
    const total = precise ? s : Math.round(s);
    const m = Math.floor(total / 60);
    const rest = precise ? toMs(total - m * 60) : total - m * 60;
    return rest ? `${m} min ${rest} s` : `${m} min`;
  }
  if (precise) return `${s} s`;
  const rounded = s >= 10 ? Math.round(s * 10) / 10 : Math.round(s * 100) / 100;
  return `${rounded} s`;
}

export function humanize(key: string): string {
  const text = key
    .replace(/[_-]+/g, " ")
    .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
    .trim()
    .split(/\s+/)
    .map((word) => (/^[A-Z0-9]{2,}$/.test(word) ? word : word.toLowerCase()))
    .join(" ");
  return text.charAt(0).toUpperCase() + text.slice(1);
}

export function displayAddress(url: string, opts: { keepQuery?: boolean } = {}): string {
  const text = url.trim();
  if (/^https?:\/\//i.test(text) && URL.canParse(text)) {
    const u = new URL(text);
    const pathname = u.pathname === "/" ? "" : u.pathname.replace(/\/$/, "");
    return `${u.host.replace(/^www\./, "")}${pathname}${opts.keepQuery ? u.search : ""}`;
  }
  return text
    .replace(/^https?:\/\//i, "")
    .replace(/^www\./, "")
    .replace(/\/$/, "");
}
