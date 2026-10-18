const UNSAFE = { word: /[^\w.-]+/g, alnum: /[^a-z0-9]+/gi } as const;

export function safeName(
  text: string,
  {
    fallback,
    lower = true,
    alphabet = "word",
  }: { fallback: string; lower?: boolean; alphabet?: keyof typeof UNSAFE },
): string {
  const safe = text.replace(UNSAFE[alphabet], "-").replace(/^-+|-+$/g, "");
  return (lower ? safe.toLowerCase() : safe) || fallback;
}
