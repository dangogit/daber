/**
 * Last-resort crash reporting for the main window.
 *
 * Handy has twice had "settings window opens blank" reports (#378, #1617) that
 * could not be reproduced or diagnosed, because a throw in the frontend is
 * currently invisible by every available route:
 *
 *   - there is no ErrorBoundary, and React 18 unmounts the whole root on an
 *     uncaught render error, leaving an empty window;
 *   - `tauri-plugin-log` only streams Rust -> webview, so nothing the webview
 *     logs reaches the log file;
 *   - WKWebView remote inspection needs `isInspectable`, which is macOS 13.3+,
 *     so release builds on older macOS cannot be inspected at all.
 *
 * This module closes that hole: it paints the error into the window (so a
 * reporter can screenshot it) and forwards it to the Rust logger (so it lands
 * in the log file the app can already open).
 *
 * Deliberately plain DOM with inline styles, and deliberately untranslated: it
 * has to render when React, Tailwind and i18n have all failed to initialize.
 */

import { invoke } from "@tauri-apps/api/core";
import { collectProbes, formatProbes } from "./diagnostics";

/**
 * How long the UI gets to mount before we suspect it never will, and how long
 * it then gets to prove us wrong. The machines this exists for are the slow ones
 * — the 2013 Mac Pro in #1617 is a decade-old Xeon parsing an 800 KB bundle — so
 * a single deadline risks painting over an app that was merely late. Nothing is
 * lost by confirming: a real crash paints immediately via the error path, and
 * only a silent non-mount ever reaches these timers.
 */
const BOOT_TIMEOUT_MS = 10_000;
const BOOT_CONFIRM_MS = 5_000;

const globals = window as unknown as Record<string, unknown>;

const captured: string[] = [];
let painted = false;
/** Headline of the visible panel, so late arrivals can refresh it in place. */
let headline = "";
let watchdog: ReturnType<typeof setTimeout> | undefined;

const describeError = (error: unknown): string => {
  if (error instanceof Error) {
    return error.stack || `${error.name}: ${error.message}`;
  }
  if (typeof error === "string") return error;
  try {
    return JSON.stringify(error);
  } catch {
    return String(error);
  }
};

/** True once React has put something in the root element. */
const appIsMounted = (): boolean => {
  const root = document.getElementById("root");
  return !!root && root.childElementCount > 0;
};

/** tauri-plugin-log serializes its levels as ints. */
const LOG_INFO = 3;
const LOG_WARN = 4;
const LOG_ERROR = 5;

/**
 * Forward to the Rust logger, which is the only route from the webview to the
 * log file. Best-effort: this runs in the failure path, so it must never throw
 * or reject onto the console it is trying to replace.
 */
const logToBackend = (message: string, level: number = LOG_ERROR): void => {
  try {
    void invoke("plugin:log|log", {
      level,
      message,
      location: "webview",
    }).catch(() => {});
  } catch {
    // Nothing else we can do.
  }
};

/**
 * Record the WebView's capabilities on every launch.
 *
 * The point is the case where nothing crashes: a report of "the UI looks wrong"
 * is otherwise unanswerable, because we have no way to ask what engine the user
 * is on. One line in the log file the app can already open settles it.
 */
export const logEnvironment = (): void => {
  logToBackend(
    `WebView environment\n${formatProbes(collectProbes())}`,
    LOG_INFO,
  );
};

const copyToClipboard = (text: string): void => {
  try {
    if (navigator.clipboard && navigator.clipboard.writeText) {
      void navigator.clipboard.writeText(text).catch(() => {});
      return;
    }
  } catch {
    // Fall through to the execCommand path.
  }
  try {
    const scratch = document.createElement("textarea");
    scratch.value = text;
    scratch.style.position = "fixed";
    scratch.style.opacity = "0";
    document.body.appendChild(scratch);
    scratch.select();
    document.execCommand("copy");
    document.body.removeChild(scratch);
  } catch {
    // Nothing else we can do; the text is on screen to be screenshotted.
  }
};

const styled = <K extends keyof HTMLElementTagNameMap>(
  tag: K,
  css: Partial<CSSStyleDeclaration>,
  text?: string,
): HTMLElementTagNameMap[K] => {
  const element = document.createElement(tag);
  Object.assign(element.style, css);
  if (text !== undefined) element.textContent = text;
  return element;
};

const buildReport = (): string =>
  [
    headline,
    "",
    captured.length ? captured.join("\n\n") : "(no error was captured)",
    "",
    formatProbes(collectProbes()),
  ].join("\n");

/**
 * Replace the window contents with the error report. Idempotent — later errors
 * append to the same panel rather than rebuilding it.
 */
const paint = (reason: string): void => {
  const host = document.getElementById("root") || document.body;
  if (!host) return;

  // The first reason to paint is the one that gets top billing; anything after
  // it is a consequence and belongs in the body of the report.
  if (!headline) headline = reason;
  const report = buildReport();

  if (painted) {
    const log = document.getElementById("handy-crash-log");
    if (log) log.textContent = report;
    return;
  }
  painted = true;

  host.textContent = "";

  const panel = styled("div", {
    position: "fixed",
    inset: "0",
    top: "0",
    left: "0",
    right: "0",
    bottom: "0",
    overflow: "auto",
    padding: "24px",
    boxSizing: "border-box",
    background: "#1c1b1a",
    color: "#f4f2f0",
    font: "13px/1.5 -apple-system, BlinkMacSystemFont, Segoe UI, sans-serif",
    zIndex: "2147483647",
  });

  panel.appendChild(
    styled(
      "h1",
      {
        margin: "0 0 4px",
        fontSize: "17px",
        fontWeight: "600",
        color: "#ff6369",
      },
      "Handy could not start its interface",
    ),
  );

  panel.appendChild(
    styled(
      "p",
      { margin: "0 0 16px", color: "#b8b4b0" },
      "This is the bug we are trying to track down. Please copy the details below (or screenshot this window) and attach them to the GitHub issue.",
    ),
  );

  const log = styled("pre", {
    margin: "0 0 16px",
    padding: "12px",
    borderRadius: "6px",
    background: "#100f0e",
    border: "1px solid #3a3835",
    color: "#e8e4e0",
    font: "11px/1.45 ui-monospace, SFMono-Regular, Menlo, monospace",
    whiteSpace: "pre-wrap",
    wordBreak: "break-word",
    userSelect: "text",
  });
  log.id = "handy-crash-log";
  log.textContent = report;
  panel.appendChild(log);

  const button = (label: string, onClick: () => void): HTMLButtonElement => {
    const element = styled(
      "button",
      {
        marginRight: "8px",
        padding: "7px 14px",
        borderRadius: "6px",
        border: "1px solid #4a4744",
        background: "#2c2b29",
        color: "#f4f2f0",
        font: "inherit",
        cursor: "pointer",
      },
      label,
    );
    element.addEventListener("click", onClick);
    return element;
  };

  panel.appendChild(
    button("Copy details", () => {
      const current = document.getElementById("handy-crash-log");
      copyToClipboard(current ? current.textContent || report : report);
    }),
  );
  panel.appendChild(button("Reload", () => window.location.reload()));

  host.appendChild(panel);
};

/** Add to the report without deciding whether the window is beyond saving. */
const record = (error: unknown, context: string, level: number): void => {
  const details = `[${context}] ${describeError(error)}`;
  captured.push(details);
  logToBackend(details, level);

  // Keep an already-visible panel current, so anything that arrives after the
  // first paint still reaches whoever screenshots it.
  const visible = document.getElementById("handy-crash-log");
  if (visible) visible.textContent = buildReport();
};

/**
 * Record a failure that leaves the window unusable.
 *
 * Painted only when the root is actually empty, unless forced — a late error in
 * a mounted app gets logged without wiping out a working UI.
 */
export const reportFatal = (
  error: unknown,
  context: string,
  options?: { force?: boolean },
): void => {
  record(error, context, LOG_ERROR);

  if (options && options.force) {
    paint("Handy hit an error while rendering.");
    return;
  }
  if (!appIsMounted()) {
    paint("Handy hit an error while starting up.");
  }
};

/**
 * Install global handlers. Call this first in `main.tsx`, before any app module
 * has a chance to throw.
 */
export const installCrashHandlers = (): void => {
  window.addEventListener("error", (event) => {
    reportFatal(event.error || event.message, "uncaught error");
  });

  // Rejections are recorded but never paint, even during startup. Unlisten
  // cleanup in @tauri-apps/api/event rejects routinely during normal operation,
  // and some of those land before React has mounted — painting on them would
  // replace a perfectly healthy app with a crash screen. If a rejection really
  // is what stopped the UI, the boot watchdog below catches it and the report
  // still contains every rejection seen along the way.
  window.addEventListener("unhandledrejection", (event) => {
    record(event.reason, "unhandled rejection", LOG_WARN);
  });

  // Drain whatever the inline bootstrap in index.html caught before this module
  // evaluated — including a failure to evaluate the bundle at all.
  const early = globals.__handyEarlyErrors;
  if (Array.isArray(early)) {
    early.forEach((entry: { kind?: string; value?: unknown }) => {
      if (entry && entry.kind === "rejection") {
        record(entry.value, "early rejection", LOG_WARN);
      } else {
        reportFatal(entry ? entry.value : entry, "startup");
      }
    });
    globals.__handyEarlyErrors = [];
  }

  // The real catch-all. Whatever went wrong — a throw, a rejection, or an app
  // that simply never mounted — the reporter gets an explanation instead of an
  // empty window. Self-checking, so a healthy app needs no cancellation.
  watchdog = setTimeout(() => {
    if (appIsMounted()) return;
    watchdog = setTimeout(() => {
      if (!appIsMounted()) {
        paint("Handy's interface never finished loading.");
      }
    }, BOOT_CONFIRM_MS);
  }, BOOT_TIMEOUT_MS);
};
