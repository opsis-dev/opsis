import type { Cookie } from "electron";

export type LocalStorageByOrigin = Map<string, Array<{ name: string; value: string }>>;

const sameSite = (s: Cookie["sameSite"]) =>
  s === "strict" ? "Strict" : s === "no_restriction" ? "None" : "Lax";

function cookieDomain(c: Cookie): string {
  if (c.hostOnly || !c.domain) return (c.domain ?? "").replace(/^\./, "");
  return c.domain.startsWith(".") ? c.domain : `.${c.domain}`;
}

export function storageState(cookies: Cookie[], origins: LocalStorageByOrigin) {
  return {
    cookies: cookies.map((c) => ({
      name: c.name,
      value: c.value,
      domain: cookieDomain(c),
      path: c.path ?? "/",
      expires: c.session || !c.expirationDate ? -1 : c.expirationDate,
      httpOnly: !!c.httpOnly,
      secure: !!c.secure,
      sameSite: sameSite(c.sameSite),
    })),
    origins: [...origins].map(([origin, localStorage]) => ({ origin, localStorage })),
  };
}
