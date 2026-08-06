import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { Slider } from "../ui/Slider";
import { useSettings } from "../../hooks/useSettings";
import { commands } from "@/bindings";

/**
 * Wake phrase trigger. Hidden entirely when no classifier is installed —
 * a toggle that cannot do anything is worse than no toggle.
 */
export const WakeWord: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const [available, setAvailable] = useState<boolean | null>(null);

  useEffect(() => {
    let cancelled = false;
    commands
      .isWakewordAvailable()
      .then((ok) => {
        if (!cancelled) setAvailable(ok);
      })
      .catch(() => {
        if (!cancelled) setAvailable(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const enabled = getSetting("wakeword_enabled") ?? false;
  const threshold = getSetting("wakeword_threshold") ?? 0.5;

  if (available !== true) return null;

  return (
    <>
      <ToggleSwitch
        checked={enabled}
        onChange={(next) => updateSetting("wakeword_enabled", next)}
        isUpdating={isUpdating("wakeword_enabled")}
        label={t("settings.wakeword.enabled.label")}
        description={t("settings.wakeword.enabled.description")}
        descriptionMode="tooltip"
        grouped
      />
      <Slider
        value={threshold}
        onChange={(value: number) => updateSetting("wakeword_threshold", value)}
        min={0.1}
        max={0.95}
        step={0.05}
        label={t("settings.wakeword.sensitivity.title")}
        description={t("settings.wakeword.sensitivity.description")}
        descriptionMode="tooltip"
        grouped
        // Shown as sensitivity, stored as a threshold: a lower threshold means
        // the trigger is more eager, which is the opposite direction.
        formatValue={(value) => `${Math.round((1 - value) * 100)}%`}
        disabled={!enabled}
      />
    </>
  );
};
