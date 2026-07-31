/**
 * Shims for WebViews older than the app's real floor.
 *
 * `tauri.conf.json` claims `minimumSystemVersion: "10.15"`, but the frontend is
 * built for a far newer engine (Vite 6 defaults to a Safari 16 target, and
 * Tailwind v4's own documented floor is Safari 16.4). macOS Monterey tops out at
 * Safari 16.6 and ships as low as 15.0, so Monterey users land on either side of
 * that line depending on whether they ever updated Safari.
 *
 * These shims cover the JS gaps that are actually reachable in our bundle. They
 * do not make the app pretty on an old engine — Tailwind still needs 16.4 — but
 * they keep it from going blank.
 *
 * Must be installed before any app module runs.
 */

import { nativeSupport } from "./diagnostics";

export const installCompatShims = (): void => {
  // Safari 15.4. Called unconditionally by react-markdown's deprecated-prop
  // check, so a missing `Object.hasOwn` throws during render of the What's New
  // modal — which unmounts the entire React root.
  if (!nativeSupport.objectHasOwn) {
    Object.defineProperty(Object, "hasOwn", {
      value: (target: object, key: PropertyKey): boolean =>
        Object.prototype.hasOwnProperty.call(target, key),
      configurable: true,
      writable: true,
    });
  }
};
