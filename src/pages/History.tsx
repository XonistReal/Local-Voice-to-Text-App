import { useEffect, useState } from "react";
import { NeoButton } from "../components/NeoButton";
import { NeoCard } from "../components/NeoCard";
import { api } from "../lib/api";
import type { TranscriptEntry } from "../types";

function formatTime(ts: number) {
  return new Date(ts * 1000).toLocaleString();
}

export function HistoryPage() {
  const [history, setHistory] = useState<TranscriptEntry[]>([]);

  useEffect(() => {
    api.getHistory().then(setHistory);
  }, []);

  const clear = async () => {
    await api.clearHistory();
    setHistory([]);
  };

  return (
    <NeoCard title="Transcript history">
      <div className="mb-4 flex items-center justify-between gap-3">
        <p className="text-sm text-[var(--neo-muted)]">
          Last {history.length} dictations stored locally on your machine.
        </p>
        <NeoButton variant="ghost" onClick={clear} disabled={!history.length}>
          Clear
        </NeoButton>
      </div>

      {history.length === 0 ? (
        <p className="text-sm text-[var(--neo-muted)]">No transcripts yet.</p>
      ) : (
        <ul className="space-y-3">
          {history.map((entry) => (
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
  );
}
