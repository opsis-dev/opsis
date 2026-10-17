import type { ReactNode } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/renderer/components/ui/dialog";
import { Input } from "@/renderer/components/ui/input";
import { ScrollArea } from "@/renderer/components/ui/scroll-area";
import { Spinner } from "@/renderer/components/ui/spinner";

export const MAX_LISTED = 100;

export function BusyCount({ busy, children }: { busy: boolean; children: ReactNode }) {
  return (
    <span className="text-muted-foreground flex items-center gap-2 text-xs">
      {busy ? <Spinner /> : null}
      {children}
    </span>
  );
}

export function SearchListDialog({
  open,
  onClose,
  title,
  description,
  placeholder,
  query,
  onQuery,
  matches,
  status,
  footer,
  children,
}: {
  open: boolean;
  onClose: () => void;
  title: string;
  description: ReactNode;
  placeholder: string;
  query: string;
  onQuery: (query: string) => void;
  matches: number;
  status?: ReactNode;
  footer?: ReactNode;
  children: ReactNode;
}) {
  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>
        <Input
          placeholder={placeholder}
          value={query}
          onChange={(e) => onQuery(e.target.value)}
          autoFocus
        />
        <ScrollArea className="h-80">
          <div className="flex flex-col pr-3">
            {status}
            {children}
            {matches > MAX_LISTED ? (
              <p className="text-muted-foreground px-2 py-1 text-xs">
                Showing the first {MAX_LISTED} matches. Type to narrow the list.
              </p>
            ) : null}
          </div>
        </ScrollArea>
        {footer ? <DialogFooter>{footer}</DialogFooter> : null}
      </DialogContent>
    </Dialog>
  );
}
