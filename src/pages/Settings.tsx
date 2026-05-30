import { useEffect, useState } from "react";
import { NeoButton } from "../components/NeoButton";
import { NeoCard } from "../components/NeoCard";
import { HotkeyCapture } from "../components/HotkeyCapture";
import { useModels } from "../hooks/useModels";
import { api } from "../lib/api";
import type { AppConfig, HardwareInfo, PerfProfile } from "../types";

function formatBytes(bytes: number) {
  if (bytes >= 1_000_000) return `${(bytes / 1_000_000).toFixed(0)} MB`;
  return `${(bytes / 1000).toFixed(0)} KB`;
}

const profiles: { id: PerfProfile; label: string; hint: string }[] = [
  { id: "potato", label: "Potato", hint: "Tiny model, 2 threads, lazy load" },
  { id: "balanced", label: "Balanced", hint: "Base model, up to 4 threads" },
  { id: "quality", label: "Quality", hint: "Small model, up to 6 threads" },
];

export function SettingsPage() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [hardware, setHardware] = useState<HardwareInfo | null>(null);
  const [saved, setSaved] = useState(false);
  const { catalog, installed, progress } = useModels();

  useEffect(() => {
    Promise.all([api.getConfig(), api.detectHardware()]).then(
      ([cfg, hw]) => {
        setConfig(cfg);
        setHardware(hw);
      },
    );
  }, []);

  if (!config) {
    return <p className="text-[var(--neo-muted)]">Loading settings…</p>;
  }

  const save = async (next: AppConfig) => {
    setConfig(next);
    await api.saveConfig(next);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  const download = async (modelId: string) => {
    await api.downloadModel(modelId);
    await save({ ...config, modelId });
  };

  return (
    <div className="flex flex-col gap-6">
      {hardware ? (
        <NeoCard title="Hardware">
          <p className="text-sm text-[var(--neo-muted)]">
            {hardware.cpuCores} cores · {hardware.totalMemoryGb} GB RAM ·
            recommended{" "}
            <span className="font-medium capitalize text-[var(--neo-text)]">
              {hardware.recommendedProfile}
            </span>
          </p>
        </NeoCard>
      ) : null}

      <NeoCard title="Performance profile">
        <div className="grid gap-3 sm:grid-cols-3">
          {profiles.map((profile) => (
            <button
              key={profile.id}
              type="button"
              className={`rounded-2xl p-4 text-left transition-all ${
                config.perfProfile === profile.id
                  ? "neo-inset"
                  : "neo-surface hover:brightness-[1.02]"
              }`}
              onClick={() => save({ ...config, perfProfile: profile.id })}
            >
              <p className="font-semibold capitalize text-[var(--neo-text)]">
                {profile.label}
              </p>
              <p className="mt-1 text-xs text-[var(--neo-muted)]">
                {profile.hint}
              </p>
            </button>
          ))}
        </div>
      </NeoCard>

      <NeoCard title="Speech model">
        <div className="space-y-3">
          {catalog.map((model) => {
            const isInstalled = installed.some((m) => m.id === model.id);
            const isActive = config.modelId === model.id;
            return (
              <div
                key={model.id}
                className="neo-inset flex flex-wrap items-center justify-between gap-3 rounded-2xl p-4"
              >
                <div>
                  <p className="font-medium text-[var(--neo-text)]">
                    {model.name}
                    {isActive ? " · Active" : ""}
                  </p>
                  <p className="text-xs text-[var(--neo-muted)]">
                    ~{formatBytes(model.sizeBytes)} · {model.profile}
                  </p>
                </div>
                <div className="flex gap-2">
                  {isInstalled ? (
                    <NeoButton
                      variant={isActive ? "accent" : "default"}
                      onClick={() => save({ ...config, modelId: model.id })}
                    >
                      {isActive ? "Selected" : "Use"}
                    </NeoButton>
                  ) : (
                    <NeoButton onClick={() => download(model.id)}>
                      Download
                    </NeoButton>
                  )}
                  {isInstalled && !isActive ? (
                    <NeoButton
                      variant="ghost"
                      onClick={() => api.deleteModel(model.id).then(() => save(config))}
                    >
                      Delete
                    </NeoButton>
                  ) : null}
                </div>
              </div>
            );
          })}
          {progress ? (
            <div className="neo-inset rounded-2xl p-4">
              <p className="mb-2 text-sm text-[var(--neo-text)]">
                Downloading {progress.modelId}… {progress.percent}%
              </p>
              <div className="h-2 overflow-hidden rounded-full bg-[var(--neo-bg)]">
                <div
                  className="h-full rounded-full bg-[var(--neo-accent)] transition-all"
                  style={{ width: `${progress.percent}%` }}
                />
              </div>
            </div>
          ) : null}
        </div>
      </NeoCard>

      <NeoCard title="Dictation">
        <div className="space-y-5">
          <label className="block">
            <span className="mb-2 block text-sm text-[var(--neo-muted)]">
              Global hotkey
            </span>
            <HotkeyCapture
              value={config.hotkey}
              onChange={(hotkey) => save({ ...config, hotkey })}
            />
          </label>

          <label className="block">
            <span className="mb-2 block text-sm text-[var(--neo-muted)]">
              Language
            </span>
            <select
              className="neo-inset w-full rounded-2xl px-4 py-3 text-[var(--neo-text)]"
              value={config.language}
              onChange={(e) => save({ ...config, language: e.target.value })}
            >
              <option value="auto">Auto-detect</option>
              <option value="en">English</option>
              <option value="es">Spanish</option>
              <option value="fr">French</option>
              <option value="de">German</option>
              <option value="ja">Japanese</option>
            </select>
          </label>

          <div className="grid gap-3 sm:grid-cols-2">
            {[
              {
                key: "preloadModel" as const,
                label: "Preload model at startup",
              },
              {
                key: "unloadWhenIdle" as const,
                label: "Unload model when idle (saves RAM)",
              },
              { key: "silenceSkip" as const, label: "Skip silent recordings" },
              { key: "showLatency" as const, label: "Show latency badge" },
              { key: "pasteMode" as const, label: "Paste instead of typing" },
            ].map((item) => (
              <label
                key={item.key}
                className="neo-inset flex cursor-pointer items-center gap-3 rounded-2xl px-4 py-3"
              >
                <input
                  type="checkbox"
                  checked={config[item.key]}
                  onChange={(e) =>
                    save({ ...config, [item.key]: e.target.checked })
                  }
                  className="h-4 w-4 accent-[var(--neo-accent)]"
                />
                <span className="text-sm text-[var(--neo-text)]">
                  {item.label}
                </span>
              </label>
            ))}
          </div>
        </div>
      </NeoCard>

      {saved ? (
        <p className="text-center text-sm text-[var(--neo-success)]">
          Settings saved
        </p>
      ) : null}
    </div>
  );
}
