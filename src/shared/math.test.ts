import { describe, expect, it } from "vitest";
import {
  clamp,
  clamp01,
  evenPx,
  finiteOr,
  lerp,
  numberOr,
  round2,
  round3,
  sampleKeyed,
} from "./math";

describe("math", () => {
  it("clamps into a range", () => {
    expect(clamp(5, 0, 10)).toBe(5);
    expect(clamp(-1, 0, 10)).toBe(0);
    expect(clamp(11, 0, 10)).toBe(10);
    expect(clamp01(1.5)).toBe(1);
  });

  it("blends between two values", () => {
    expect(lerp(2, 6, 0.25)).toBe(3);
    expect(lerp(2, 6, 1)).toBe(6);
  });

  it("rounds to three decimals", () => {
    expect(round3(1.23456)).toBe(1.235);
    expect(round3(29.97002997)).toBe(29.97);
    expect(round2(1.235)).toBe(1.24);
  });

  it("reads numbers with a fallback", () => {
    expect(numberOr(0.5, 1)).toBe(0.5);
    expect(numberOr("0.5", 1)).toBe(1);
    expect(numberOr(undefined, 1)).toBe(1);
  });
});

describe("sampleKeyed", () => {
  const keys = [
    { at: 0, v: 0 },
    { at: 10, v: 10 },
    { at: 10, v: 20 },
    { at: 20, v: 40 },
  ];
  const sample = (x: number) =>
    sampleKeyed(
      keys,
      x,
      (k) => k.at,
      (k) => k.v,
      (a, b, t) => a.v + (b.v - a.v) * t,
    );

  it("holds the ends and interpolates between keys", () => {
    expect(sample(-5)).toBe(0);
    expect(sample(5)).toBe(5);
    expect(sample(25)).toBe(40);
  });

  it("takes the later key where two share a time", () => {
    expect(sample(10)).toBe(20);
    expect(sample(15)).toBe(30);
  });

  it("gives nothing without keys", () => {
    expect(sampleKeyed([], 1, Number, Number, () => 0)).toBeUndefined();
  });
});

describe("number coercion", () => {
  it("only accepts finite numbers", () => {
    expect(numberOr(Number.NaN, 1)).toBe(1);
    expect(numberOr(Infinity, 1)).toBe(1);
    expect(numberOr("5", 1)).toBe(1);
    expect(finiteOr("5", 1)).toBe(5);
    expect(finiteOr("x", 1)).toBe(1);
  });

  it("rounds to even pixels", () => {
    expect(evenPx(1079)).toBe(1080);
    expect(evenPx(607.5)).toBe(608);
  });
});
