import { createContext, useContext, useState } from "react";
import type { DrawFunction } from "#video/lib/transitions/presentation.ts";

type CutSide = { image: OffscreenCanvas; progress: number; draw: DrawFunction };

type Cut = { entering?: CutSide; exiting?: CutSide };

type CutDirection = "entering" | "exiting";

export type CutBoard = {
  report: (cut: number, direction: CutDirection, side: CutSide) => void;
  leave: (cut: number, direction: CutDirection) => void;
};

function drawCut({ entering, exiting }: Cut): void {
  if (entering && !exiting) {
    entering.draw(entering.image, null, entering.progress);
    return;
  }
  if (exiting && !entering) {
    exiting.draw(null, exiting.image, exiting.progress);
    return;
  }
  if (entering && exiting && entering.progress === exiting.progress) {
    entering.draw(entering.image, exiting.image, entering.progress);
    exiting.draw(null, null, 0);
  }
}

export function createCutBoard(): CutBoard {
  const cuts = new Map<number, Cut>();
  return {
    report(cut, direction, side) {
      const sides = { ...cuts.get(cut), [direction]: side };
      cuts.set(cut, sides);
      drawCut(sides);
    },
    leave(cut, direction) {
      const sides = { ...cuts.get(cut) };
      delete sides[direction];
      if (sides.entering || sides.exiting) {
        cuts.set(cut, sides);
        drawCut(sides);
      } else cuts.delete(cut);
    },
  };
}

const CutBoardContext = createContext<CutBoard | null>(null);

export function CutBoardProvider({ children }: { children: React.ReactNode }) {
  const [board] = useState(createCutBoard);
  return <CutBoardContext.Provider value={board}>{children}</CutBoardContext.Provider>;
}

export function useCutBoard(): CutBoard | null {
  return useContext(CutBoardContext);
}
