import { Menu } from "lucide-react";
import { useEffect, type ReactNode } from "react";
import { acceleratorLabel, appMenu, matchesAccelerator, type MenuEntry } from "@/shared/menu";
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
import { Separator } from "@/renderer/components/ui/separator";
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

function hexOf(color: string): string | null {
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = 1;
  const ctx = canvas.getContext("2d");
  if (!ctx) return null;
  ctx.fillStyle = color;
  ctx.fillRect(0, 0, 1, 1);
  const [r, g, b] = ctx.getImageData(0, 0, 1, 1).data;
  return `#${[r, g, b].map((v) => v.toString(16).padStart(2, "0")).join("")}`;
}

export function useTitleBarColors(): void {
  useEffect(() => {
    if (isMac) return;
    const probe = document.createElement("div");
    probe.className = "bg-background text-foreground";
    document.body.appendChild(probe);
    const style = getComputedStyle(probe);
    const color = hexOf(style.backgroundColor);
    const symbolColor = hexOf(style.color);
    probe.remove();
    if (color && symbolColor) void window.demoflow.setTitleBarColors({ color, symbolColor });
  }, []);
}

export function AppBar({ children, className }: { children?: ReactNode; className?: string }) {
  return (
    <header
      className={cn(
        "app-drag bg-background flex h-12 shrink-0 items-center gap-2 border-b pr-3",
        isMac ? "pl-21" : "pl-2",
        className,
      )}
      style={
        isMac
          ? undefined
          : {
              paddingRight:
                "calc(100vw - env(titlebar-area-x, 0px) - env(titlebar-area-width, 100vw) + 0.75rem)",
            }
      }
    >
      {isMac ? null : (
        <>
          <WideMenu />
          <CompactMenu />
          {children ? <Separator orientation="vertical" className="mx-1 h-5 self-center" /> : null}
        </>
      )}
      {children}
    </header>
  );
}

export function PageBar({ title }: { title: string }) {
  return (
    <AppBar>{title ? <span className="text-muted-foreground text-sm">{title}</span> : null}</AppBar>
  );
}
