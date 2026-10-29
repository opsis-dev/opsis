import { uniqueKeys } from "@opsis/video/common/lists";
import type { CardRow, WorkflowNode } from "@/generated/bindings";
import { cn } from "@/lib/utils";
import { NodeIcon } from "./node-icon";
import { useCompiledContext } from "./use-compiled";

const TONE_CLASS: Record<NonNullable<CardRow["tone"]> | "plain", string> = {
  warn: "text-warning",
  muted: "text-muted-foreground",
  plain: "text-foreground/85",
};

const NO_ROWS: CardRow[] = [];

export function useCardRows(node: WorkflowNode): CardRow[] {
  const { cards } = useCompiledContext();
  return cards[node.id] ?? NO_ROWS;
}

const MAX_SWATCHES = 8;

export function CardRows({ rows }: { rows: CardRow[] }) {
  if (!rows.length) return null;
  const keys = uniqueKeys(rows.map((r) => `${r.icon}:${r.text}`));
  return (
    <ul className="flex flex-col gap-1 px-3">
      {rows.map((r, i) => (
        <li
          key={keys[i]}
          title={r.title ?? r.text}
          className={cn("flex min-w-0 items-center gap-1.5 text-xs", TONE_CLASS[r.tone ?? "plain"])}
        >
          <NodeIcon
            name={r.icon}
            className={cn("size-3.5 shrink-0", r.tone ? undefined : "text-muted-foreground")}
          />
          {r.swatches?.length ? (
            <span className="flex shrink-0 -space-x-1" aria-hidden>
              {r.swatches.slice(0, MAX_SWATCHES).map((color, j) => (
                <span
                  key={j}
                  className="ring-card size-3 rounded-full ring-1"
                  style={{ background: color }}
                />
              ))}
            </span>
          ) : null}
          <span className="truncate">{r.text}</span>
        </li>
      ))}
    </ul>
  );
}
