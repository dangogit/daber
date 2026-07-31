/**
 * WebView capability probes for the crash screen.
 *
 * Everything here has to survive the situation it exists to describe: it runs on
 * the oldest WebView we ship to, after the rest of the bundle has already failed.
 * So: no imports from the app, no i18n, no React, and no syntax or API newer than
 * the floor we claim to support in `tauri.conf.json`.
 *
 * The probes are chosen to bracket the WebKit version without asking the reporter
 * to dig through "About Safari" — each one names the Safari release that shipped
 * it, so a filled-in table pins the engine to a narrow range.
 */

export interface Probe {
  name: string;
  value: string;
}

const globals = window as unknown as Record<string, unknown>;

// `Object.hasOwn` and `Array.prototype.at` are ES2022; the project's `lib` is
// ES2020, so they are probed through casts rather than direct references.
const objectCtor = Object as unknown as Record<string, unknown>;
const arrayProto = Array.prototype as unknown as Record<string, unknown>;

/** Runs a feature test that is allowed to throw; a throw counts as unsupported. */
const probe = (test: () => boolean): string => {
  try {
    return test() ? "yes" : "no";
  } catch {
    return "no";
  }
};

const cssSupports = (condition: string): string =>
  probe(() => typeof CSS !== "undefined" && CSS.supports(condition));

const matchUserAgent = (pattern: string): string => {
  try {
    const match = navigator.userAgent.match(new RegExp(pattern));
    return match ? match[1] : "unknown";
  } catch {
    return "unknown";
  }
};

/**
 * Feature-detect regular expression lookbehind (Safari 16.4).
 *
 * Built from a string on purpose: a lookbehind *literal* is a parse error on
 * older WebKit, which would take down the whole bundle instead of just this
 * probe. Never write one as a literal anywhere in this codebase.
 */
export const supportsRegExpLookbehind = (): boolean => {
  try {
    new RegExp("(?<=a)b");
    return true;
  } catch {
    return false;
  }
};

/**
 * Snapshot of native support, taken when this module is evaluated — i.e. before
 * `installCompatShims()` runs. Without the snapshot the probe table would report
 * our own polyfill as native support and tell us nothing about the engine.
 *
 * `compat.ts` reads this, which is also what guarantees the ordering: the shim
 * module depends on this one, so this one evaluates first.
 */
export const nativeSupport = Object.freeze({
  objectHasOwn: typeof objectCtor.hasOwn === "function",
  structuredClone: typeof globals.structuredClone === "function",
  arrayAt: typeof arrayProto.at === "function",
  regExpLookbehind: supportsRegExpLookbehind(),
});

export const collectProbes = (): Probe[] => [
  { name: "Handy", value: __APP_VERSION__ },
  { name: "User agent", value: navigator.userAgent },
  { name: "WebKit build", value: matchUserAgent("AppleWebKit/([\\d.]+)") },
  { name: "Safari version", value: matchUserAgent("Version/([\\d.]+)") },

  // JS APIs the bundle depends on, with the Safari release that shipped each.
  // Read from the pre-shim snapshot so we see the engine, not our polyfills.
  {
    name: "RegExp lookbehind (16.4)",
    value: nativeSupport.regExpLookbehind ? "yes" : "no",
  },
  {
    name: "Object.hasOwn (15.4)",
    value: nativeSupport.objectHasOwn ? "yes" : "no",
  },
  {
    name: "structuredClone (15.4)",
    value: nativeSupport.structuredClone ? "yes" : "no",
  },
  {
    name: "Array.prototype.at (15.4)",
    value: nativeSupport.arrayAt ? "yes" : "no",
  },

  // CSS features Tailwind v4 assumes. Tailwind's own documented floor is Safari
  // 16.4, so a "no" on @property means the UI will be badly styled even once the
  // JS is healthy.
  {
    name: "CSS @property (16.4)",
    value: probe(
      () =>
        typeof CSS !== "undefined" &&
        typeof (CSS as unknown as Record<string, unknown>).registerProperty ===
          "function",
    ),
  },
  {
    name: "CSS color-mix() (16.2)",
    value: cssSupports("color: color-mix(in srgb, red, blue)"),
  },
  {
    name: "CSS @layer (15.4)",
    value: probe(() => typeof globals.CSSLayerBlockRule !== "undefined"),
  },
  { name: "CSS :has() (15.4)", value: cssSupports("selector(:has(*))") },
];

export const formatProbes = (probes: Probe[]): string =>
  probes.map((entry) => `${entry.name}: ${entry.value}`).join("\n");
