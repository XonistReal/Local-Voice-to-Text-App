import { getCurrentWindow } from "@tauri-apps/api/window";
import { api } from "../lib/api";

interface TitleBarProps {
  title?: string;
}

export function TitleBar({ title = "VTT" }: TitleBarProps) {
  const win = getCurrentWindow();

  return (
    <header
      data-tauri-drag-region
      className="titlebar flex h-10 shrink-0 items-center justify-between border-b border-[oklch(85%_0.01_260)] bg-[var(--neo-bg)] px-3 select-none"
    >
      <div
        data-tauri-drag-region
        className="flex min-w-0 flex-1 items-center gap-2 text-sm font-semibold text-[var(--neo-text)]"
      >
        <span
          aria-hidden
          className="titlebar-icon inline-block h-5 w-5 shrink-0 rounded-md bg-contain bg-center bg-no-repeat"
          style={{ backgroundImage: "url(/app-icon.png)" }}
        />
        <span data-tauri-drag-region className="truncate">
          {title}
        </span>
      </div>
      <div className="flex items-center gap-1" data-tauri-drag-region="false">        <button
          type="button"
          aria-label="Minimize"
          className="neo-surface flex h-7 w-8 items-center justify-center rounded-lg text-[var(--neo-muted)] hover:text-[var(--neo-text)]"
          onClick={() => win.minimize()}
        >
          −
        </button>
        <button
          type="button"
          aria-label="Close"
          className="neo-surface flex h-7 w-8 items-center justify-center rounded-lg text-[var(--neo-muted)] hover:text-[var(--neo-danger)]"
          onClick={() => api.quitApp()}
        >
          ×
        </button>
      </div>
    </header>
  );
}
