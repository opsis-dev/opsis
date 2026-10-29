import { HEADER_W } from "./timeline-layout";

export function TimelineRow({
  top,
  height,
  label,
  width,
  children,
}: {
  top: number;
  height: number;
  label: string;
  width: number;
  children: React.ReactNode;
}) {
  return (
    <div
      className="absolute left-0 flex border-b border-foreground/5"
      style={{ top, height, width: HEADER_W + width }}
    >
      <div
        className="bg-sidebar text-muted-foreground sticky left-0 z-20 flex shrink-0 items-center border-r px-2 text-[11px]"
        style={{ width: HEADER_W }}
      >
        {label}
      </div>
      <div className="relative flex-1">{children}</div>
    </div>
  );
}
