import { Braces, Pause } from "lucide-react";
import { useLayoutEffect, useRef } from "react";
import { PAUSE_MARK, tokenize, type TextToken } from "@/shared/vars";
import { Button } from "@/renderer/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from "@/renderer/components/ui/dropdown-menu";
import { cn } from "@/renderer/lib/utils";

export type VarInfo = { name: string; value: string };

const CHIP =
  "mx-0.5 inline-flex items-center gap-1 rounded-md bg-primary/15 px-1.5 align-baseline text-xs font-medium text-primary select-none";

function chip(token: Exclude<TextToken, { kind: "text" }>, vars: VarInfo[]): HTMLElement {
  const el = document.createElement("span");
  el.contentEditable = "false";
  el.className = CHIP;
  if (token.kind === "var") {
    el.dataset.var = token.name;
    el.textContent = token.name;
    const value = vars.find((v) => v.name === token.name)?.value;
    el.title = value === undefined ? "This variable doesn't exist" : value || "(empty)";
    if (value === undefined) el.classList.add("text-destructive", "bg-destructive/15");
  } else {
    el.dataset.pause = "";
    el.textContent = "pause";
    el.title = "Holds still for a moment while typing";
  }
  return el;
}

function nodesFor(tokens: TextToken[], vars: VarInfo[]): Node[] {
  return tokens.map((t) => (t.kind === "text" ? document.createTextNode(t.text) : chip(t, vars)));
}

const blockBreak = (index: number) => (index > 0 ? "\n" : "");

function serializeNode(n: Node, index: number): string {
  if (n.nodeType === Node.TEXT_NODE) return n.nodeValue ?? "";
  if (!(n instanceof HTMLElement)) return "";
  if (n.dataset.var !== undefined) return `{{${n.dataset.var}}}`;
  if (n.dataset.pause !== undefined) return PAUSE_MARK;
  switch (n.tagName) {
    case "BR":
      return "\n";
    case "DIV":
    case "P":
      return blockBreak(index) + serialize(n);
    default:
      return serialize(n);
  }
}

function serialize(root: Node): string {
  return [...root.childNodes].map(serializeNode).join("");
}

function useEditable({
  value,
  onChange,
  vars,
  pauses,
}: {
  value: string;
  onChange: (value: string) => void;
  vars: VarInfo[];
  pauses: boolean;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const rendered = useRef<string | null>(null);
  const range = useRef<Range | null>(null);
  const varsKey = vars.map((v) => `${v.name}=${v.value}`).join("\n");

  useLayoutEffect(() => {
    const el = ref.current;
    const key = `${varsKey}\u0000${value}`;
    if (!el || rendered.current === key) return;
    el.replaceChildren(...nodesFor(tokenize(value, pauses), vars));
    rendered.current = key;
  }, [value, pauses, vars, varsKey]);

  const commit = () => {
    const el = ref.current;
    if (!el) return;
    const next = serialize(el);
    rendered.current = `${varsKey}\u0000${next}`;
    if (next !== value) onChange(next);
  };

  const saveRange = () => {
    const sel = window.getSelection();
    if (sel?.rangeCount && ref.current?.contains(sel.anchorNode))
      range.current = sel.getRangeAt(0).cloneRange();
  };

  const insertionRange = (el: HTMLElement) => {
    if (range.current && el.contains(range.current.startContainer)) return range.current;
    const r = document.createRange();
    r.selectNodeContents(el);
    r.collapse(false);
    return r;
  };

  const insert = (tokens: TextToken[]) => {
    const el = ref.current;
    if (!el) return;
    el.focus();
    const sel = window.getSelection();
    const r = insertionRange(el);
    r.deleteContents();
    const nodes = nodesFor(tokens, vars);
    const frag = document.createDocumentFragment();
    for (const n of nodes) frag.appendChild(n);
    const last = nodes.at(-1);
    r.insertNode(frag);
    if (last) {
      r.setStartAfter(last);
      r.collapse(true);
    }
    sel?.removeAllRanges();
    sel?.addRange(r);
    range.current = r.cloneRange();
    commit();
  };

  return { ref, commit, saveRange, insert };
}

function VariablesMenu({
  vars,
  compact,
  onOpen,
  onPick,
}: {
  vars: VarInfo[];
  compact: boolean;
  onOpen: () => void;
  onPick: (name: string) => void;
}) {
  const trigger = compact ? (
    <Button
      variant="ghost"
      size="icon-sm"
      aria-label="Insert variable"
      title="Insert variable"
      className="text-muted-foreground shrink-0"
      onMouseDown={onOpen}
    >
      <Braces />
    </Button>
  ) : (
    <Button variant="ghost" size="xs" className="text-muted-foreground" onMouseDown={onOpen}>
      <Braces data-icon="inline-start" />
      Insert variable
    </Button>
  );
  return (
    <DropdownMenu>
      <DropdownMenuTrigger render={trigger} />
      <DropdownMenuContent align="start" className="w-56">
        <DropdownMenuGroup>
          <DropdownMenuLabel>Variables</DropdownMenuLabel>
          {vars.map((v) => (
            <DropdownMenuItem key={v.name} onClick={() => onPick(v.name)}>
              <span className="truncate">{v.name}</span>
              <span className="text-muted-foreground ml-auto max-w-24 truncate text-xs">
                {v.value}
              </span>
            </DropdownMenuItem>
          ))}
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

const SURFACE = [
  "border-input min-w-0 rounded-lg border bg-transparent px-2.5 py-1 text-sm break-words whitespace-pre-wrap transition-colors outline-none",
  "focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-3 dark:bg-input/30",
  "data-[empty=true]:before:text-muted-foreground data-[empty=true]:before:content-[attr(data-placeholder)]",
];

export function VarText({
  id,
  value,
  onChange,
  vars,
  multiline = false,
  pauses = false,
  placeholder,
}: {
  id: string;
  value: string;
  onChange: (value: string) => void;
  vars: VarInfo[];
  multiline?: boolean;
  pauses?: boolean;
  placeholder?: string;
}) {
  const { ref, commit, saveRange, insert } = useEditable({ value, onChange, vars, pauses });
  const surface = (
    <div
      id={id}
      ref={ref}
      role="textbox"
      aria-multiline={multiline}
      contentEditable
      suppressContentEditableWarning
      spellCheck
      data-placeholder={placeholder}
      data-empty={value ? "false" : "true"}
      className={cn(SURFACE, multiline ? "min-h-16 w-full py-2" : "min-h-8 flex-1 leading-6")}
      onInput={commit}
      onKeyUp={saveRange}
      onMouseUp={saveRange}
      onBlur={saveRange}
      onKeyDown={(e) => {
        if (e.key !== "Enter") return;
        e.preventDefault();
        if (multiline) document.execCommand("insertLineBreak");
      }}
      onPaste={(e) => {
        e.preventDefault();
        const text = e.clipboardData.getData("text/plain");
        saveRange();
        insert(tokenize(multiline ? text : text.replace(/\s*\n\s*/g, " "), pauses));
      }}
      onDrop={(e) => e.preventDefault()}
    />
  );
  const menu = (
    <VariablesMenu
      vars={vars}
      compact={!multiline}
      onOpen={saveRange}
      onPick={(name) => insert([{ kind: "var", name }])}
    />
  );
  if (!multiline)
    return (
      <div className="flex items-start gap-1">
        {surface}
        {menu}
      </div>
    );
  return (
    <div className="flex flex-col gap-1.5">
      {surface}
      <div className="flex gap-1">
        {menu}
        {pauses ? (
          <Button
            variant="ghost"
            size="xs"
            className="text-muted-foreground"
            onMouseDown={saveRange}
            onClick={() => insert([{ kind: "pause" }])}
          >
            <Pause data-icon="inline-start" />
            Add pause
          </Button>
        ) : null}
      </div>
    </div>
  );
}
