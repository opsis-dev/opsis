import type { EdgeEnds } from "@/generated/bindings";

const ID_ALPHABET = "0123456789abcdefghijklmnopqrstuvwxyz";
const ID_LENGTH = 8;

export function randomId(prefix?: string): string {
  const bytes = crypto.getRandomValues(new Uint8Array(ID_LENGTH));
  const id = Array.from(bytes, (b) => ID_ALPHABET[b % ID_ALPHABET.length]).join("");
  return prefix ? `${prefix}_${id}` : id;
}

export function edgeId(e: EdgeEnds): string {
  return `${e.source}.${e.sourceHandle}->${e.target}.${e.targetHandle}`;
}
