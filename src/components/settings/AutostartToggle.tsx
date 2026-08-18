import React from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { useSettings } from "../../hooks/useSettings";

interface AutostartToggleProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const AutostartToggle: React.FC<AutostartToggleProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();

    // Mirrors default_autostart_enabled() in settings.rs. A `?? false` here
    // would render the toggle off while the app is in fact registered to start
    // at login, for the render before settings arrive.
    const autostartEnabled = getSetting("autostart_enabled") ?? true;

    return (
      <ToggleSwitch
        checked={autostartEnabled}
        onChange={(enabled) => updateSetting("autostart_enabled", enabled)}
        isUpdating={isUpdating("autostart_enabled")}
        label={t("settings.advanced.autostart.label")}
        description={t("settings.advanced.autostart.description")}
        descriptionMode={descriptionMode}
        grouped={grouped}
      />
    );
  },
);
