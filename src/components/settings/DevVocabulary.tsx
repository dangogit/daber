import React from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { useSettings } from "../../hooks/useSettings";

interface DevVocabularyProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const DevVocabulary: React.FC<DevVocabularyProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();

    const enabled = getSetting("dev_vocabulary") ?? true;

    return (
      <ToggleSwitch
        checked={enabled}
        onChange={(enabled) => updateSetting("dev_vocabulary", enabled)}
        isUpdating={isUpdating("dev_vocabulary")}
        label={t("settings.advanced.devVocabulary.label")}
        description={t("settings.advanced.devVocabulary.description")}
        descriptionMode={descriptionMode}
        grouped={grouped}
      />
    );
  },
);
