import React from "react";
import ReactDOM from "react-dom/client";
import { platform } from "@tauri-apps/plugin-os";
import App from "./App";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { logEnvironment, reportFatal } from "./lib/crashScreen";
import {
  applyTheme,
  getStoredTheme,
  syncThemeFromSettings,
} from "./lib/utils/theme";

// Initialize i18n
import "./i18n";

// Initialize model store (loads models and sets up event listeners)
import { useModelStore } from "./stores/modelStore";

logEnvironment();

try {
  // Set platform before render so CSS can scope per-platform (e.g. scrollbar styles)
  document.documentElement.dataset.platform = platform();

  // Apply the last-known theme synchronously before render to avoid a flash of
  // the wrong palette, then reconcile with the persisted setting once it loads.
  applyTheme(getStoredTheme());
  syncThemeFromSettings();

  useModelStore.getState().initialize();

  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <ErrorBoundary context="app root">
        <App />
      </ErrorBoundary>
    </React.StrictMode>,
  );
} catch (error) {
  reportFatal(error, "startup");
}
