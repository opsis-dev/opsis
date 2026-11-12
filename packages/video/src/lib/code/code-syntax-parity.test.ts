import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { expect, it } from "vitest";
import { tokenizeCode } from "#video/lib/code/code-syntax.tsx";

const FIXTURE = path.resolve(import.meta.dirname, "../../../../../fixtures/scene/code-tokens.json");

const SAMPLES = [
  "export async function publish(draft: Draft) {\n  const page = await render(draft);\n  await upload(page, { public: true });\n  return page.url;\n}",
  "/* a block\n   still inside */ let x = 1; /* open\nclosed */ y",
  "/*/ odd */ z /*/",
  "const s = 'it\\'s' + \"a \\\"b\\\"\" + `t ${x}` + 'unterminated",
  "const bad = 'ends with \\",
  "n = 0x1F + 12 + 3.14 + 1e10 + 2.5e-3 + 1.5x + 12e + 7.e2 + 0x + 1_000",
  "<Button onClick={go} />\n</Panel>\nif (a == b) { c = d; e(f) }",
  "# shell comment\nfn main() -> Result<(), Error> { let mut v: Vec<u8> = vec![]; }",
  "def run(self):\n    return self.value  # done",
  "   tabs\tand spaces  ",
  "naïve café = $value_1 + _priv",
];

function recorded() {
  return {
    samples: SAMPLES,
    tokens: SAMPLES.map((sample) => tokenizeCode(sample.split("\n"))),
  };
}

it("records the film's code tokens, for the native renderer's port to match", () => {
  const made = recorded();
  if (process.env.OPSIS_WRITE_GOLDEN === "1") {
    mkdirSync(path.dirname(FIXTURE), { recursive: true });
    writeFileSync(FIXTURE, `${JSON.stringify(made, null, 1)}\n`);
  }
  expect(existsSync(FIXTURE), `${FIXTURE} is missing: record it with OPSIS_WRITE_GOLDEN=1`).toBe(
    true,
  );
  expect(JSON.parse(readFileSync(FIXTURE, "utf8"))).toEqual(JSON.parse(JSON.stringify(made)));
});
