import React, { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { platform } from "@tauri-apps/plugin-os";
import { Loader2 } from "lucide-react";
import AccessibilityOnboarding from "./AccessibilityOnboarding";
import HotkeyStep from "./HotkeyStep";
import TryItStep from "./TryItStep";
import type { HebrewModelState } from "@/hooks/useHebrewModel";

interface OnboardingProps {
  onComplete: () => Promise<boolean>;
  model: HebrewModelState;
  initialStep?: Step;
}

type Step = "permissions" | "hotkey" | "try";

/**
 * First run, as a sequence rather than a pair of hardcoded screens.
 *
 * Which steps exist depends on the platform: Linux has neither of the
 * permissions the other two ask for, so it starts at the shortcut instead of
 * flashing an empty screen. The model is not a step at all — it downloads in
 * the background from the moment the app opens, so the setup someone has to do
 * anyway covers the wait. That download is owned by App rather than by this
 * component, because someone can reach the end of onboarding before it
 * finishes and the model still has to be selected when it lands.
 */
const Onboarding: React.FC<OnboardingProps> = ({
  onComplete,
  model,
  initialStep,
}) => {
  const { t } = useTranslation();

  const steps = useMemo<Step[]>(() => {
    const os = platform();
    const hasPermissionsToGrant = os === "macos" || os === "windows";
    return hasPermissionsToGrant
      ? ["permissions", "hotkey", "try"]
      : ["hotkey", "try"];
  }, []);

  const [index, setIndex] = useState(() =>
    initialStep ? Math.max(0, steps.indexOf(initialStep)) : 0,
  );
  const step = steps[index];

  const next = useCallback(async () => {
    if (index + 1 >= steps.length) {
      return onComplete();
    }
    setIndex(index + 1);
    return true;
  }, [index, steps.length, onComplete]);

  return (
    <div className="titlebar-inset h-screen w-screen flex flex-col">
      <div className="titlebar-drag absolute inset-x-0 top-0 h-7" />

      <div className="flex-1 min-h-0">
        {step === "permissions" && (
          <AccessibilityOnboarding onComplete={() => void next()} />
        )}
        {step === "hotkey" && <HotkeyStep onContinue={() => void next()} />}
        {step === "try" && <TryItStep model={model} onDone={next} />}
      </div>

      <div className="shrink-0 flex flex-col items-center gap-3 pb-6">
        {/* The last step reports on the download in full, so saying it twice
            there would just be noise. Before that, this is the only sign that
            anything is happening — without it, arriving at a half-finished
            download reads as the app having stalled. */}
        {step !== "try" && model.status === "downloading" && (
          <span className="flex items-center gap-2 text-xs text-text/50">
            <Loader2 className="w-3 h-3 animate-spin" />
            {t("onboarding.try.downloading")}
            <span className="font-mono">
              {t("onboarding.percent", {
                percent: Math.round(model.percentage),
              })}
            </span>
          </span>
        )}

        <div className="flex items-center justify-center gap-2">
          {steps.map((s, i) => (
            <span
              key={s}
              aria-hidden
              className={`h-1.5 rounded-full transition-all duration-300 ${
                i === index ? "w-6 bg-logo-primary" : "w-1.5 bg-text/20"
              }`}
            />
          ))}
          <span className="sr-only">
            {t("onboarding.stepCount", {
              current: index + 1,
              total: steps.length,
            })}
          </span>
        </div>
      </div>
    </div>
  );
};

export default Onboarding;
