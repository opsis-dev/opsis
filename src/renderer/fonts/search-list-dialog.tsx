import type { ReactNode } from "react";
import { Input } from "@/renderer/components/ui/input";
import { ScrollArea } from "@/renderer/components/ui/scroll-area";
import { FontDialog } from "./font-dialog";

const MAX_LISTED = 100;

export function matchesQuery(query: string, ...texts: string[]): boolean {
  const q = query.trim().toLowerCase();
  return !q || texts.some((text) => text.toLowerCase().includes(q));
}

export function SearchListDialog<T>({
  onClose,
  title,
  description,
  placeholder,
  query,
  onQuery,
  items,
  status,
  footer,
  children,
}: {
  onClose: () => void;
  title: string;
  description: ReactNode;
  placeholder: string;
  query: string;
  onQuery: (query: string) => void;
  items: readonly T[];
  status?: ReactNode;
  footer?: ReactNode;
  children: (item: T) => ReactNode;
}) {
  return (
    <FontDialog onClose={onClose} title={title} description={description} footer={footer}>
      <Input
        placeholder={placeholder}
        value={query}
        onChange={(e) => onQuery(e.target.value)}
        autoFocus
      />
      <ScrollArea className="h-80">
        <div className="flex flex-col pr-3">
          {status}
          {items.slice(0, MAX_LISTED).map(children)}
          {items.length > MAX_LISTED ? (
            <p className="text-muted-foreground px-2 py-1 text-xs">
              Showing the first {MAX_LISTED} matches. Type to narrow the list.
            </p>
          ) : null}
        </div>
      </ScrollArea>
    </FontDialog>
  );
}
