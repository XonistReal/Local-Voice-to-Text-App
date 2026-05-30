import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { NeoButton } from "../components/NeoButton";
import { NeoCard } from "../components/NeoCard";
import { Waveform } from "../components/Waveform";
import { useRecordingState } from "../hooks/useRecordingState";
import { api } from "../lib/api";
import type { AppConfig, TranscriptEntry } from "../types";

function formatTime(ts: number) {
  return new Date(ts * 1000).toLocaleString();
}

export function HomePage() {
  const { status } = useRecordingState();
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [history, setHistory] = useState<TranscriptEntry[]>([]);
  const [holding, setHolding] = useState(false);

  useEffect(() => {
    api.getConfig().then(setConfig);
    api.getHistory().then(setHistory);
    const unlisten = listen<TranscriptEntry>("transcript-added", (event) => {
      setHistory((prev) => [event.payload, ...prev].slice(0, 10));
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const isRecording = status.status === "recording" || holding;
  const isBusy =
    status.status === "transcribing" || status.status === "loadingmodel";

  const handlePressStart = async () => {
    setHolding(true);
    await api.startRecording();
  };

  const handlePressEnd = async () => {
    setHolding(false);
    await api.stopAndTranscribe(false);
    setHistory(await api.getHistory());
  };

  return (
    <div className="flex flex-col gap-6">
      <NeoCard>
        <div className="flex flex-col items-center gap-5 py-4 text-center">
          <div
            className={`neo-surface flex h-40 w-40 flex-col items-center justify-center gap-3 rounded-full transition-all ${
              isRecording ? "neo-pressed scale-95" : ""
            }`}
          >
            <Waveform active={isRecording} />
            <p className="text-sm font-medium text-[var(--neo-text)]">
              {isRecording
                ? "Listening…"
                : isBusy
                  ? "Processing…"
                  : "Ready"}
            </p>
          </div>

          <p className="max-w-md text-sm text-[var(--neo-muted)]">
            Hold{" "}
            <span className="font-mono text-[var(--neo-text)]">
              {config?.hotkey ?? "Ctrl+Shift+Space"}
            </span>{" "}
            anywhere, or use the button below to dictate.
          </p>

          <NeoButton
            variant="accent"
            disabled={isBusy}
            onMouseDown={handlePressStart}
            onMouseUp={handlePressEnd}
            onMouseLeave={() => holding && handlePressEnd()}
            onTouchStart={handlePressStart}
            onTouchEnd={handlePressEnd}
          >
            {isRecording ? "Release to transcribe" : "Hold to record"}
          </NeoButton>

          {config?.showLatency && status.lastLatencyMs != null ? (
            <p className="text-xs text-[var(--neo-muted)]">
              Last transcription: {status.lastLatencyMs} ms ·{" "}
              {status.inferenceDevice ?? "CPU"}
            </p>
          ) : status.inferenceDevice ? (
            <p className="text-xs text-[var(--neo-muted)]">
              Inference: {status.inferenceDevice}
              {status.gpuActive ? " (active)" : ""}
            </p>
          ) : null}

          {status.message ? (
            <p className="text-sm text-[var(--neo-muted)]">{status.message}</p>
          ) : null}
        </div>
      </NeoCard>

      <NeoCard title="Recent transcripts">
        {history.length === 0 ? (
          <p className="text-sm text-[var(--neo-muted)]">
            Your dictations will appear here after your first recording.
          </p>
        ) : (
          <ul className="space-y-3">
            {history.slice(0, 5).map((entry) => (
              <li key={entry.id} className="neo-inset rounded-2xl p-4">
                <p className="text-sm text-[var(--neo-text)]">{entry.text}</p>
                <p className="mt-2 text-xs text-[var(--neo-muted)]">
                  {formatTime(entry.timestamp)} · {entry.latencyMs} ms
                </p>
              </li>
            ))}
          </ul>
        )}
      </NeoCard>
    </div>
  );
}
