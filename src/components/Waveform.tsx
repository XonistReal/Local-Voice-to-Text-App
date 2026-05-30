interface WaveformProps {
  active?: boolean;
}

export function Waveform({ active = false }: WaveformProps) {
  return (
    <div
      className="flex h-8 items-end justify-center gap-1"
      aria-hidden={!active}
    >
      {[0, 1, 2, 3, 4].map((i) => (
        <span
          key={i}
          className={`w-1.5 rounded-full bg-[var(--neo-accent)] ${
            active ? "animate-pulse" : "opacity-40"
          }`}
          style={{
            height: active ? `${12 + (i % 3) * 8}px` : "8px",
            animationDelay: `${i * 120}ms`,
          }}
        />
      ))}
    </div>
  );
}
