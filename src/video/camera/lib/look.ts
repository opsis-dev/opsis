export const LOOKS = ["framed", "fullbleed"] as const;
export type ReelLook = (typeof LOOKS)[number];
export const DEFAULT_LOOK: ReelLook = "framed";
