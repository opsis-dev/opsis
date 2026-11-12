const SHARES = [0, 10, 25, 50, 75, 90, 100];

const SPACES = ["oklch", "oklab", "srgb", "srgb-linear"];

export function Case() {
  return (
    <div
      style={{
        position: "absolute",
        inset: 24,
        display: "grid",
        gridTemplateColumns: `repeat(${SHARES.length}, 1fr)`,
        gap: 4,
      }}
    >
      {SPACES.flatMap((space) =>
        SHARES.map((share) => (
          <div
            key={`${space}-${share}`}
            style={{
              minHeight: 44,
              background: `color-mix(in ${space}, #f97316 ${share}%, #0ea5e9)`,
            }}
          />
        )),
      )}
      {SHARES.map((share) => (
        <div
          key={`alpha-${share}`}
          style={{
            minHeight: 44,
            background: `color-mix(in oklch, #f8fafc ${share}%, transparent)`,
          }}
        />
      ))}
      {SHARES.map((share) => (
        <div
          key={`hue-${share}`}
          style={{
            minHeight: 44,
            background: `color-mix(in oklch longer hue, oklch(0.7 0.2 20) ${share}%, oklch(0.7 0.2 200))`,
          }}
        />
      ))}
    </div>
  );
}
