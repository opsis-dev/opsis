export type StemLevels = { fps: number; peak: number[]; rms: number[] };

export type Loudness = {
  fps: number;
  momentary: number[];
  shortTerm: number[];
  integrated: number | null;
  range: number | null;
  truePeak: number | null;
};

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
