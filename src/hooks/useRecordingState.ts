import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api } from "../lib/api";
import type { StatusPayload } from "../types";

export function useRecordingState() {
  const [status, setStatus] = useState<StatusPayload>({ status: "idle" });

  const refresh = useCallback(async () => {
    setStatus(await api.getStatus());
  }, []);

  useEffect(() => {
    refresh();
    const unlisten = listen<StatusPayload>("status-changed", (event) => {
      setStatus(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [refresh]);

  return { status, refresh };
}
