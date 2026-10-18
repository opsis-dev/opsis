import { isOneOf } from "@/shared/guards";

export const whenOneOf =
  <T extends string>(values: readonly T[], apply: (value: T) => void) =>
  (value: unknown): void => {
    if (isOneOf(values, value)) apply(value);
  };

export const whenFirstOneOf =
  <T extends string>(values: readonly T[], apply: (value: T) => void) =>
  (picked: readonly unknown[]): void =>
    whenOneOf(values, apply)(picked[0]);
