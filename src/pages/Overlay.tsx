import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Waveform } from "../components/Waveform";
import type { StatusPayload } from "../types";

export function OverlayPage() {
  const [status, setStatus] = useState<StatusPayload>({
    status: "idle",
    gpuActive: false,
    inferenceDevice: "CPU",
  });

  useEffect(() => {
    document.body.style.overflow = "hidden";

    const unlistenStatus = listen<StatusPayload>("status-changed", (event) => {
      setStatus(event.payload);
    });
    const unlistenOverlay = listen<StatusPayload>("overlay-state", (event) => {
      setStatus(event.payload);
    });
    return () => {
      document.body.style.overflow = "";
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
    <div className="flex h-full w-full items-center justify-center bg-transparent p-6">
      <div className="neo-surface flex items-center gap-3 rounded-full px-4 py-2.5">
        <Waveform active={status.status === "recording"} />
        <span className="whitespace-nowrap text-sm font-medium text-[var(--neo-text)]">
          {label}
        </span>
      </div>
    </div>
  );
}
