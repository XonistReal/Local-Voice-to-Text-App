import { useEffect, useState } from "react";
import { NeoButton } from "./NeoButton";

interface HotkeyCaptureProps {
  value: string;
  onChange: (hotkey: string) => void;
}

export function HotkeyCapture({ value, onChange }: HotkeyCaptureProps) {
  const [listening, setListening] = useState(false);

  useEffect(() => {
    if (!listening) return;

    const onKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();

      if (e.key === "Escape") {
        setListening(false);
        return;
      }

      const parts: string[] = [];
      if (e.ctrlKey || e.metaKey) parts.push("Ctrl");
      if (e.shiftKey) parts.push("Shift");
      if (e.altKey) parts.push("Alt");

      const key = e.key.length === 1 ? e.key.toUpperCase() : e.key;
      if (!["Control", "Shift", "Alt", "Meta"].includes(e.key)) {
        parts.push(key === " " ? "Space" : key);
      }

      if (parts.length >= 2) {
        onChange(parts.join("+"));
        setListening(false);
      }
    };

    window.addEventListener("keydown", onKeyDown, true);
    return () => window.removeEventListener("keydown", onKeyDown, true);
  }, [listening, onChange]);

  return (
    <div className="flex items-center gap-3">
      <div className="neo-inset min-w-[220px] rounded-2xl px-4 py-3 font-mono text-sm text-[var(--neo-text)]">
        {value}
      </div>
      <NeoButton type="button" onClick={() => setListening(true)}>
        {listening ? "Press keys…" : "Change"}
      </NeoButton>
    </div>
  );
}
