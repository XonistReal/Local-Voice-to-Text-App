import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api } from "../lib/api";
import type {
  DownloadProgress,
  InstalledModel,
  ModelCatalogEntry,
} from "../types";

export function useModels() {
  const [catalog, setCatalog] = useState<ModelCatalogEntry[]>([]);
  const [installed, setInstalled] = useState<InstalledModel[]>([]);
  const [progress, setProgress] = useState<DownloadProgress | null>(null);

  const refresh = useCallback(async () => {
    const [cat, inst] = await Promise.all([
      api.getModelCatalog(),
      api.getInstalledModels(),
    ]);
    setCatalog(cat);
    setInstalled(inst);
  }, []);

  useEffect(() => {
    refresh();
    const unlisten = listen<DownloadProgress>(
      "model-download-progress",
      (event) => {
        setProgress(event.payload);
        if (event.payload.percent >= 100) {
          refresh();
        }
      },
    );
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [refresh]);

  return { catalog, installed, progress, refresh };
}
