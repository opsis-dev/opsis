import { z } from "zod";

const levelList = z.array(z.number());

export const stemLevelsSchema = z.object({ fps: z.number(), peak: levelList, rms: levelList });
export type StemLevels = z.infer<typeof stemLevelsSchema>;

export const loudnessSchema = z.object({
  fps: z.number(),
  momentary: levelList,
  shortTerm: levelList,
  integrated: z.number().nullable(),
  range: z.number().nullable(),
  truePeak: z.number().nullable(),
});
export type Loudness = z.infer<typeof loudnessSchema>;

export type MixResult = {
  key: string;
  master: { src: string; hash: string } | null;
  levels: {
    channels: Record<string, StemLevels>;
    buses: Record<string, StemLevels>;
    master: StemLevels | null;
  };
  reduction: { channels: Record<string, number[]>; master: number[] | null };
  loudness: Loudness | null;
};
