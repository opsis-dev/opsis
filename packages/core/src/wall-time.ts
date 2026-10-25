const WALL_TIME = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2})(?::(\d{2}))?$/;

type WallParts = [number, number, number, number, number, number];

function wallParts(time: string): WallParts | null {
  const match = WALL_TIME.exec(time.trim());
  if (!match) return null;
  const [y, mo, d, h, mi, s = "0"] = match.slice(1);
  const parts: WallParts = [Number(y), Number(mo), Number(d), Number(h), Number(mi), Number(s)];
  const utc = new Date(Date.UTC(parts[0], parts[1] - 1, parts[2], parts[3], parts[4], parts[5]));
  const same =
    utc.getUTCFullYear() === parts[0] &&
    utc.getUTCMonth() === parts[1] - 1 &&
    utc.getUTCDate() === parts[2] &&
    utc.getUTCHours() === parts[3] &&
    utc.getUTCMinutes() === parts[4];
  return same ? parts : null;
}

export const isWallTime = (time: string): boolean => wallParts(time) !== null;

function zoneOffsetMs(epochMs: number, timeZone: string): number {
  const fields = new Intl.DateTimeFormat("en-US", {
    timeZone,
    hourCycle: "h23",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  }).formatToParts(new Date(epochMs));
  const get = (type: Intl.DateTimeFormatPartTypes) =>
    Number(fields.find((f) => f.type === type)?.value);
  const wall = Date.UTC(
    get("year"),
    get("month") - 1,
    get("day"),
    get("hour"),
    get("minute"),
    get("second"),
  );
  return wall - Math.floor(epochMs / 1000) * 1000;
}

export function wallTimeIn(time: string, timeZone: string): Date {
  const parts = wallParts(time);
  if (!parts) throw new Error(`“${time}” isn't a date and time.`);
  const asUtc = Date.UTC(parts[0], parts[1] - 1, parts[2], parts[3], parts[4], parts[5]);
  const first = asUtc - zoneOffsetMs(asUtc, timeZone);
  return new Date(asUtc - zoneOffsetMs(first, timeZone));
}
