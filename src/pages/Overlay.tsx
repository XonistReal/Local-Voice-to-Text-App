import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Waveform } from "../components/Waveform";
import type { StatusPayload } from "../types";

export function OverlayPage() {
  const [status, setStatus] = useState<StatusPayload>({ status: "idle" });

  useEffect(() => {
    document.documentElement.classList.add("overlay-root");
    const unlistenStatus = listen<StatusPayload>("status-changed", (event) => {
      setStatus(event.payload);
    });
    const unlistenOverlay = listen<StatusPayload>("overlay-state", (event) => {
      setStatus(event.payload);
    });
    return () => {
      document.documentElement.classList.remove("overlay-root");
      unlistenStatus.then((fn) => fn());
      unlistenOverlay.then((fn) => fn());
    };
  }, []);

  const label =
    status.status === "recording"
      ? "Recording"
      : status.status === "transcribing"
        ? "Transcribing"
        : "VTT";

  return (
    <div className="flex min-h-screen items-center justify-center bg-transparent p-2">
      <div className="neo-surface flex items-center gap-3 rounded-full px-5 py-3 shadow-lg">
        <Waveform active={status.status === "recording"} />
        <span className="text-sm font-medium text-[var(--neo-text)]">
          {label}
        </span>
      </div>
    </div>
  );
}
