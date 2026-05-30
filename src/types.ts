export type PerfProfile = "potato" | "balanced" | "quality";

export type AppStatus =
  | "idle"
  | "recording"
  | "transcribing"
  | "loadingmodel"
  | "error";

export interface AppConfig {
  perfProfile: PerfProfile;
  autoDetectProfile: boolean;
  modelId: string;
  language: string;
  hotkey: string;
  preloadModel: boolean;
  maxRecordingSecs: number;
  silenceSkip: boolean;
  showLatency: boolean;
  injectDelayMs: number;
  pasteMode: boolean;
  unloadWhenIdle: boolean;
  onboardingComplete: boolean;
}

export interface StatusPayload {
  status: AppStatus;
  message?: string | null;
  lastLatencyMs?: number | null;
}

export interface HardwareInfo {
  totalMemoryGb: number;
  cpuCores: number;
  recommendedProfile: PerfProfile;
}

export interface ModelCatalogEntry {
  id: string;
  name: string;
  filename: string;
  url: string;
  sizeBytes: number;
  profile: PerfProfile;
}

export interface InstalledModel {
  id: string;
  name: string;
  filename: string;
  sizeBytes: number;
  path: string;
}

export interface TranscriptEntry {
  id: string;
  text: string;
  timestamp: number;
  latencyMs: number;
}

export interface DownloadProgress {
  modelId: string;
  downloaded: number;
  total: number;
  percent: number;
}
