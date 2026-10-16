import { Copy, Menu, Minus, Square, X } from "lucide-react";
import { useEffect, useState, type ReactNode } from "react";
import type { WindowState } from "@/shared/ipc";
import logoUrl from "../../../brand/logo.svg";
import {
  acceleratorLabel,
  appMenu,
  matchesAccelerator,
  type MenuEntry,
  type WindowRole,
} from "@/shared/menu";
import { Button } from "@/renderer/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from "@/renderer/components/ui/dropdown-menu";
import {
  Menubar,
  MenubarContent,
  MenubarGroup,
  MenubarItem,
  MenubarMenu,
  MenubarSeparator,
  MenubarShortcut,
  MenubarTrigger,
} from "@/renderer/components/ui/menubar";
import { Badge } from "@/renderer/components/ui/badge";
import { Separator } from "@/renderer/components/ui/separator";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { useAppSettings } from "../store/app-settings";
import { cn } from "@/renderer/lib/utils";

const platform = window.demoflow?.platform ?? "linux";
export const isMac = platform === "darwin";
const SECTIONS = appMenu(platform);

function run(entry: MenuEntry) {
  if (entry.kind === "command") window.demoflow.dispatchMenuCommand(entry.command);
  else if (entry.kind === "role") void window.demoflow.windowRole(entry.role);
}

function groups(items: MenuEntry[]): MenuEntry[][] {
  const out: MenuEntry[][] = [[]];
  for (const item of items) {
    if (item.kind === "separator") out.push([]);
    else out.at(-1)!.push(item);
  }
  return out.filter((g) => g.length);
}

const shortcut = (entry: MenuEntry) =>
  entry.kind !== "separator" && entry.accelerator
    ? acceleratorLabel(entry.accelerator, platform)
    : null;

function WideMenu() {
  return (
    <Menubar className="hidden h-8 border-0 bg-transparent p-0 xl:flex">
      {SECTIONS.map((section) => (
        <MenubarMenu key={section.label}>
          <MenubarTrigger className="px-2 font-normal">{section.label}</MenubarTrigger>
          <MenubarContent className="min-w-56">
            {groups(section.items).map((group, i) => (
              <MenubarGroup key={i}>
                {i > 0 ? <MenubarSeparator /> : null}
                {group.map((entry) =>
                  entry.kind === "separator" ? null : (
                    <MenubarItem key={entry.label} onClick={() => run(entry)}>
                      {entry.label}
                      {shortcut(entry) ? (
                        <MenubarShortcut>{shortcut(entry)}</MenubarShortcut>
                      ) : null}
                    </MenubarItem>
                  ),
                )}
              </MenubarGroup>
            ))}
          </MenubarContent>
        </MenubarMenu>
      ))}
    </Menubar>
  );
}

function CompactMenu() {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={<Button variant="ghost" size="icon-sm" aria-label="Menu" className="xl:hidden" />}
      >
        <Menu />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-48">
        <DropdownMenuGroup>
          {SECTIONS.map((section) => (
            <DropdownMenuSub key={section.label}>
              <DropdownMenuSubTrigger>{section.label}</DropdownMenuSubTrigger>
              <DropdownMenuSubContent className="min-w-56">
                {groups(section.items).map((group, i) => (
                  <DropdownMenuGroup key={i}>
                    {i > 0 ? <DropdownMenuSeparator /> : null}
                    {group.map((entry) =>
                      entry.kind === "separator" ? null : (
                        <DropdownMenuItem key={entry.label} onClick={() => run(entry)}>
                          {entry.label}
                          {shortcut(entry) ? (
                            <DropdownMenuShortcut>{shortcut(entry)}</DropdownMenuShortcut>
                          ) : null}
                        </DropdownMenuItem>
                      ),
                    )}
                  </DropdownMenuGroup>
                ))}
              </DropdownMenuSubContent>
            </DropdownMenuSub>
          ))}
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

const editing = () => {
  const el = document.activeElement as HTMLElement | null;
  return (
    !!el &&
    (el.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(el.tagName)) &&
    !(el instanceof HTMLInputElement && /^(checkbox|radio|button|range)$/.test(el.type))
  );
};

const NATIVE_WHILE_EDITING = new Set([
  "cut",
  "copy",
  "paste",
  "selectAll",
  "edit:undo",
  "edit:redo",
]);

export function useMenuShortcuts(): void {
  useEffect(() => {
    if (isMac) return;
    const entries = SECTIONS.flatMap((s) => s.items).filter(
      (e): e is Exclude<MenuEntry, { kind: "separator" }> =>
        e.kind !== "separator" && !!e.accelerator,
    );
    const onKey = (e: KeyboardEvent) => {
      if (e.defaultPrevented) return;
      const entry = entries.find((x) => matchesAccelerator(x.accelerator!, e, platform));
      if (!entry) return;
      const id = entry.kind === "command" ? entry.command : entry.role;
      if (editing() && NATIVE_WHILE_EDITING.has(id)) return;
      e.preventDefault();
      run(entry);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
}

export function useWindowState(): WindowState {
  const [state, setState] = useState<WindowState>({
    maximized: false,
    fullscreen: false,
    focused: true,
  });
  useEffect(() => {
    if (isMac) return;
    void window.demoflow.getWindowState().then(setState);
    return window.demoflow.onWindowState(setState);
  }, []);
  return state;
}

function WindowControls() {
  const { maximized } = useWindowState();
  const role = (r: WindowRole) => () => void window.demoflow.windowRole(r);
  const base =
    "text-muted-foreground hover:text-foreground grid h-full w-12 place-items-center transition-colors";
  return (
    <div className="flex h-full shrink-0 items-stretch">
      <button
        type="button"
        aria-label="Minimize"
        className={cn(base, "hover:bg-accent")}
        onClick={role("minimize")}
      >
        <Minus className="size-4" strokeWidth={1.5} />
      </button>
      <button
        type="button"
        aria-label={maximized ? "Restore" : "Maximize"}
        className={cn(base, "hover:bg-accent")}
        onClick={role("toggleMaximize")}
      >
        {maximized ? (
          <Copy className="size-3.5 -scale-x-100" strokeWidth={1.5} />
        ) : (
          <Square className="size-3.5" strokeWidth={1.5} />
        )}
      </button>
      <button
        type="button"
        aria-label="Close"
        className={cn(base, "hover:bg-destructive hover:text-white")}
        onClick={role("close")}
      >
        <X className="size-4" strokeWidth={1.5} />
      </button>
    </div>
  );
}

export function WindowBorder() {
  const { maximized, fullscreen, focused } = useWindowState();
  if (isMac || maximized || fullscreen) return null;
  return (
    <div
      aria-hidden
      className={cn(
        "pointer-events-none fixed inset-0 z-50 border",
        focused ? "border-white/12" : "border-white/7",
      )}
    />
  );
}

export function AppBar({
  children,
  className,
  menu = true,
}: {
  children?: ReactNode;
  className?: string;
  menu?: boolean;
}) {
  return (
    <header
      className={cn(
        "app-drag bg-sidebar text-sidebar-foreground flex h-12 shrink-0 items-center border-b",
        isMac ? "pr-3 pl-21" : "pl-2",
        className,
      )}
    >
      <div className="flex h-full min-w-0 flex-1 items-center gap-2">
        <img src={logoUrl} alt="Demoflow" className="mx-1 size-5 shrink-0" draggable={false} />
        {isMac || !menu ? null : (
          <>
            <WideMenu />
            <CompactMenu />
            {children ? (
              <Separator orientation="vertical" className="mx-1 h-5 data-vertical:self-center" />
            ) : null}
          </>
        )}
        {children}
      </div>
      <ApiBadge />
      {isMac ? null : (
        <>
          <Separator orientation="vertical" className="mx-3 h-5 data-vertical:self-center" />
          <WindowControls />
        </>
      )}
    </header>
  );
}

function ApiBadge() {
  const on = useAppSettings((s) => s.settings?.api.enabled);
  const status = useAppSettings((s) => s.api);
  if (!on || !status) return null;
  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <button
            type="button"
            className="ml-2 shrink-0"
            onClick={() => void window.demoflow.openSettings("api")}
          />
        }
      >
        <Badge variant={status.listening ? "secondary" : "destructive"}>API</Badge>
      </TooltipTrigger>
      <TooltipContent>
        {status.listening
          ? "Scripts and agents on this computer can use the open project."
          : "The API is switched on but isn't running. Open its settings to see why."}
      </TooltipContent>
    </Tooltip>
  );
}

export function PageBar({ title }: { title: string }) {
  return (
    <AppBar>{title ? <span className="text-muted-foreground text-sm">{title}</span> : null}</AppBar>
  );
}
