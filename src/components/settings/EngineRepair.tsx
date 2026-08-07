import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Loader2, RotateCcw } from "lucide-react";
import { SettingContainer } from "../ui/SettingContainer";
import { HEBREW_MODEL_ID } from "@/lib/constants/models";
import { useModelStore } from "@/stores/modelStore";

interface EngineRepairProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

/**
 * The only model control left in the app.
 *
 * There is nothing to choose, but a truncated or corrupted download is real and
 * otherwise leaves someone with an app that fails on every attempt and no way
 * to fix it short of finding the data directory by hand. Deleting first is what
 * makes this a repair rather than a no-op: the downloader treats a file that is
 * already present as done.
 */
export const EngineRepair: React.FC<EngineRepairProps> = ({
  descriptionMode = "tooltip",
  grouped = false,
}) => {
  const { t } = useTranslation();
  const { deleteModel, downloadModel } = useModelStore();
  const [busy, setBusy] = useState(false);

  const redownload = async () => {
    setBusy(true);
    try {
      await deleteModel(HEBREW_MODEL_ID);
      const ok = await downloadModel(HEBREW_MODEL_ID);
      if (ok) toast.success(t("settings.engine.repair.started"));
    } finally {
      setBusy(false);
    }
  };

  return (
    <SettingContainer
      title={t("settings.engine.repair.title")}
      description={t("settings.engine.repair.description")}
      descriptionMode={descriptionMode}
      grouped={grouped}
    >
      <button
        onClick={redownload}
        disabled={busy}
        className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-mid-gray/20 hover:border-logo-primary text-sm font-medium transition-colors disabled:opacity-50"
      >
        {busy ? (
          <Loader2 className="w-4 h-4 animate-spin" />
        ) : (
          <RotateCcw className="w-4 h-4" />
        )}
        {t("settings.engine.repair.action")}
      </button>
    </SettingContainer>
  );
};

export default EngineRepair;
