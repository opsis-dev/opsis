import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import { archiveFile, archiveName, compilerArgs, privateSymbol, type SkiaPlatform } from "./skia";

const PLATFORMS: SkiaPlatform[] = ["linux", "darwin", "win32"];
const platform = fc.constantFrom(...PLATFORMS);
const name = fc.stringMatching(/^[a-z][a-z0-9_]{0,15}$/);
const symbol = fc.stringMatching(/^_{0,2}[A-Za-z][A-Za-z0-9_]{0,30}$/);

describe("Skia's archives", () => {
  it("name each library back from its file on every system", () => {
    fc.assert(
      fc.property(platform, name, (system, library) => {
        expect(archiveName(archiveFile(library, system), system)).toBe(library);
      }),
    );
  });

  it("leave out files that aren't archives on that system", () => {
    fc.assert(
      fc.property(name, (library) => {
        expect(archiveName(archiveFile(library, "win32"), "linux")).toBeNull();
        expect(archiveName(`${library}.ninja`, "win32")).toBeNull();
      }),
    );
  });
});

describe("Skia's compiler", () => {
  it("is clang-cl from the given LLVM on Windows, with no flags only Unix compilers know", () => {
    const args = compilerArgs("win32", "C:/Program Files/LLVM");
    expect(args.clang_win).toBe(JSON.stringify("C:/Program Files/LLVM"));
    expect(args.cc).toBeUndefined();
    expect(args.extra_cflags).not.toContain("-fPIC");
    expect(JSON.parse(args.extra_cflags ?? "[]")).toContain("/MD");
  });

  it("keeps Chrome's defines on every system", () => {
    fc.assert(
      fc.property(platform, (system) => {
        expect(compilerArgs(system, "C:/LLVM").extra_cflags).toContain("SK_GAMMA_EXPONENT=1.2");
      }),
    );
  });

  it("is clang on Linux and macOS", () => {
    fc.assert(
      fc.property(fc.constantFrom<SkiaPlatform>("linux", "darwin"), (system) => {
        expect(compilerArgs(system).cc).toBe(JSON.stringify("clang"));
      }),
    );
  });
});

describe("a private symbol", () => {
  it("is new and keeps the original at its end", () => {
    fc.assert(
      fc.property(platform, symbol, (system, a) => {
        const renamed = privateSymbol(a, system);
        expect(renamed).not.toBe(a);
        expect(renamed.endsWith(a.replace(/^_/, ""))).toBe(true);
      }),
    );
  });

  it("is distinct for distinct symbols", () => {
    fc.assert(
      fc.property(platform, symbol, symbol, (system, a, b) => {
        fc.pre(a !== b);
        expect(privateSymbol(b, system)).not.toBe(privateSymbol(a, system));
      }),
    );
  });

  it("keeps the leading underscore of a macOS symbol in front", () => {
    fc.assert(
      fc.property(symbol, (s) => {
        const renamed = privateSymbol(`_${s}`, "darwin");
        expect(renamed.startsWith("_opsis_skia_")).toBe(true);
      }),
    );
  });
});
