import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  HardwareInfo,
  InstalledModel,
  ModelCatalogEntry,
  StatusPayload,
  TranscriptEntry,
} from "../types";

export const api = {
  getConfig: () => invoke<AppConfig>("get_config"),
  saveConfig: (config: AppConfig) => invoke<void>("save_app_config", { config }),
  getStatus: () => invoke<StatusPayload>("get_status"),
  detectHardware: () => invoke<HardwareInfo>("detect_hardware_info"),
  getModelCatalog: () => invoke<ModelCatalogEntry[]>("get_model_catalog"),
  getInstalledModels: () => invoke<InstalledModel[]>("get_installed_models"),
  downloadModel: (modelId: string) =>
    invoke<string>("download_model_cmd", { modelId }),
  deleteModel: (modelId: string) =>
    invoke<void>("delete_model_cmd", { modelId }),
  getHistory: () => invoke<TranscriptEntry[]>("get_history"),
  clearHistory: () => invoke<void>("clear_history"),
  completeOnboarding: () => invoke<void>("complete_onboarding"),
  startRecording: () => invoke<void>("start_recording"),
  stopAndTranscribe: (skipInjection = false) =>
    invoke<string>("stop_and_transcribe", { skipInjection }),
  preloadModel: () => invoke<void>("preload_model"),
  showMainWindow: () => invoke<void>("show_main_window"),
  quitApp: () => invoke<void>("quit_app"),
};
