import { lazy, Suspense, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  BrowserRouter,
  Navigate,
  Route,
  Routes,
  useNavigate,
} from "react-router-dom";
import { Layout } from "./components/Layout";
import { HomePage } from "./pages/Home";
import { OnboardingPage } from "./pages/Onboarding";
import { OverlayPage } from "./pages/Overlay";
import { api } from "./lib/api";

const SettingsPage = lazy(() =>
  import("./pages/Settings").then((m) => ({ default: m.SettingsPage })),
);
const HistoryPage = lazy(() =>
  import("./pages/History").then((m) => ({ default: m.HistoryPage })),
);

function MainApp() {
  const [ready, setReady] = useState(false);
  const [onboardingComplete, setOnboardingComplete] = useState(false);
  const navigate = useNavigate();

  useEffect(() => {
    api.getConfig().then((config) => {
      setOnboardingComplete(config.onboardingComplete);
      setReady(true);
    });

    const unlisten = listen<string>("navigate", (event) => {
      navigate(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [navigate]);

  if (!ready) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <p className="text-[var(--neo-muted)]">Loading VTT…</p>
      </div>
    );
  }

  if (!onboardingComplete) {
    return (
      <OnboardingPage
        onComplete={() => {
          setOnboardingComplete(true);
          navigate("/");
        }}
      />
    );
  }

  return (
    <Suspense
      fallback={
        <div className="flex min-h-screen items-center justify-center">
          <p className="text-[var(--neo-muted)]">Loading…</p>
        </div>
      }
    >
      <Routes>
        <Route element={<Layout />}>
          <Route index element={<HomePage />} />
          <Route path="settings" element={<SettingsPage />} />
          <Route path="history" element={<HistoryPage />} />
        </Route>
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </Suspense>
  );
}

export default function App() {
  const isOverlay =
    window.location.pathname === "/overlay" ||
    window.location.hash.includes("overlay");

  if (isOverlay) {
    return <OverlayPage />;
  }

  return (
    <BrowserRouter>
      <MainApp />
    </BrowserRouter>
  );
}
