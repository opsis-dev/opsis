import type { ComponentProps } from "react";
import { Slider } from "@/components/ui/slider";

type SingleSliderProps = Omit<
  ComponentProps<typeof Slider>,
  "value" | "defaultValue" | "onValueChange" | "onChange"
> & { value: number; onChange: (value: number) => void };

export function SingleSlider({ value, onChange, ...props }: SingleSliderProps) {
  return (
    <Slider
      {...props}
      value={[value]}
      onValueChange={(v) => onChange(typeof v === "number" ? v : v[0])}
    />
  );
}
