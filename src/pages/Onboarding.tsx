import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { NeoButton } from "../components/NeoButton";
import { NeoCard } from "../components/NeoCard";
import { HotkeyCapture } from "../components/HotkeyCapture";
import { useModels } from "../hooks/useModels";
import { api } from "../lib/api";
import type { AppConfig, HardwareInfo } from "../types";

type Step = "welcome" | "hardware" | "model" | "hotkey" | "test";

export function OnboardingPage({ onComplete }: { onComplete: () => void }) {
  const [step, setStep] = useState<Step>("welcome");
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [hardware, setHardware] = useState<HardwareInfo | null>(null);
  const [testText, setTestText] = useState("");
  const { catalog, installed, progress } = useModels();

  useEffect(() => {
    Promise.all([api.getConfig(), api.detectHardware()]).then(
      ([cfg, hw]) => {
        setConfig(cfg);
        setHardware(hw);
      },
    );
  }, []);

  if (!config || !hardware) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-[var(--neo-bg)]">
        <p className="text-[var(--neo-muted)]">Preparing VTT…</p>
      </div>
    );
  }

  const recommendedModel =
    catalog.find((m) => m.profile === hardware.recommendedProfile)?.id ??
    "tiny.en";

  const finish = async () => {
    await api.completeOnboarding();
    onComplete();
  };

  return (
    <div className="min-h-screen bg-[var(--neo-bg)] p-6">
      <div className="mx-auto flex max-w-xl flex-col gap-6">
        <header>
          <p className="text-sm uppercase tracking-wider text-[var(--neo-muted)]">
            Setup
          </p>
          <h1 className="text-3xl font-bold text-[var(--neo-text)]">
            Welcome to VTT
          </h1>
          <p className="mt-2 text-[var(--neo-muted)]">
            Private, offline voice-to-text. No subscription, no cloud.
          </p>
        </header>

        {step === "welcome" && (
          <NeoCard title="What you get">
            <ul className="space-y-2 text-sm text-[var(--neo-muted)]">
              <li>Push-to-talk dictation into any app</li>
              <li>Everything runs locally on your machine</li>
              <li>Optimized for low-end hardware with Potato mode</li>
            </ul>
            <NeoButton
              className="mt-6 w-full"
              variant="accent"
              onClick={() => setStep("hardware")}
            >
              Get started
            </NeoButton>
          </NeoCard>
        )}

        {step === "hardware" && (
          <NeoCard title="Your hardware">
            <p className="text-sm text-[var(--neo-muted)]">
              Detected {hardware.cpuCores} cores and {hardware.totalMemoryGb} GB
              RAM. We recommend the{" "}
              <span className="capitalize text-[var(--neo-text)]">
                {hardware.recommendedProfile}
              </span>{" "}
              profile.
            </p>
            <NeoButton
              className="mt-6 w-full"
              variant="accent"
              onClick={() => {
                setConfig({
                  ...config,
                  perfProfile: hardware.recommendedProfile,
                  modelId: recommendedModel,
                  preloadModel: hardware.recommendedProfile !== "potato",
                  unloadWhenIdle: hardware.recommendedProfile === "potato",
                });
                setStep("model");
              }}
            >
              Continue
            </NeoButton>
          </NeoCard>
        )}

        {step === "model" && (
          <NeoCard title="Download speech model">
            <p className="mb-4 text-sm text-[var(--neo-muted)]">
              Download{" "}
              {catalog.find((m) => m.id === recommendedModel)?.name ??
                recommendedModel}{" "}
              (~75 MB). This stays on your device and works offline.
            </p>
            {installed.some((m) => m.id === recommendedModel) ? (
              <NeoButton
                className="w-full"
                variant="accent"
                onClick={() => setStep("hotkey")}
              >
                Model ready — continue
              </NeoButton>
            ) : (
              <>
                <NeoButton
                  className="w-full"
                  variant="accent"
                  onClick={() => api.downloadModel(recommendedModel)}
                >
                  Download model
                </NeoButton>
                {progress ? (
                  <div className="mt-4 neo-inset rounded-2xl p-4">
                    <p className="mb-2 text-sm">{progress.percent}%</p>
                    <div className="h-2 overflow-hidden rounded-full bg-[var(--neo-bg)]">
                      <div
                        className="h-full bg-[var(--neo-accent)]"
                        style={{ width: `${progress.percent}%` }}
                      />
                    </div>
                  </div>
                ) : null}
              </>
            )}
          </NeoCard>
        )}

        {step === "hotkey" && (
          <NeoCard title="Choose your hotkey">
            <p className="mb-4 text-sm text-[var(--neo-muted)]">
              Hold this shortcut anywhere to dictate. Release to transcribe.
            </p>
            <HotkeyCapture
              value={config.hotkey}
              onChange={(hotkey) => setConfig({ ...config, hotkey })}
            />
            <NeoButton
              className="mt-6 w-full"
              variant="accent"
              onClick={async () => {
                await api.saveConfig({
                  ...config,
                  modelId: recommendedModel,
                });
                setStep("test");
              }}
            >
              Continue
            </NeoButton>
          </NeoCard>
        )}

        {step === "test" && (
          <NeoCard title="Try it">
            <p className="mb-4 text-sm text-[var(--neo-muted)]">
              Hold the button, speak, then release. Your words appear below.
            </p>
            <TestRecorder onTranscript={setTestText} />
            <textarea
              className="neo-inset min-h-28 w-full resize-none rounded-2xl p-4 text-[var(--neo-text)]"
              placeholder="Transcript appears here…"
              value={testText}
              onChange={(e) => setTestText(e.target.value)}
            />
            <NeoButton className="mt-6 w-full" variant="accent" onClick={finish}>
              Finish setup
            </NeoButton>
          </NeoCard>
        )}
      </div>
    </div>
  );
}

function TestRecorder({ onTranscript }: { onTranscript: (text: string) => void }) {
  useEffect(() => {
    const unlisten = listen<{ text: string }>("transcript-added", (event) => {
      onTranscript(event.payload.text);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [onTranscript]);

  return (
    <NeoButton
      className="mb-4 w-full"
      variant="accent"
      onMouseDown={() => api.startRecording()}
      onMouseUp={() => api.stopAndTranscribe()}
    >
      Hold to test
    </NeoButton>
  );
}
