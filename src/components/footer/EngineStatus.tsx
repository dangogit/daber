import React from "react";
import { useTranslation } from "react-i18next";
import { Loader2 } from "lucide-react";
import { HEBREW_MODEL_ID } from "@/lib/constants/models";
import { useModelStore } from "@/stores/modelStore";

/**
 * What the model picker used to occupy in the footer.
 *
 * With one engine there is nothing to pick, but there is still something worth
 * saying: whether it is on disk yet. Read-only on purpose — the download is
 * owned by `useHebrewModel`, and a second component starting one would race it.
 */
export const EngineStatus: React.FC = () => {
  const { t } = useTranslation();
  const models = useModelStore((s) => s.models);
  const downloadProgress = useModelStore((s) => s.downloadProgress);
  const downloading = useModelStore(
    (s) => HEBREW_MODEL_ID in s.downloadingModels,
  );

  const ready = models.find((m) => m.id === HEBREW_MODEL_ID)?.is_downloaded;

  if (downloading) {
    return (
      <span className="flex items-center gap-1.5">
        <Loader2 className="w-3 h-3 animate-spin" />
        {t("footer.engine.downloading", {
          percent: Math.round(
            downloadProgress[HEBREW_MODEL_ID]?.percentage ?? 0,
          ),
        })}
      </span>
    );
  }

  return (
    <span className="flex items-center gap-1.5">
      <span
        aria-hidden
        className={`w-1.5 h-1.5 rounded-full ${
          ready ? "bg-emerald-500" : "bg-mid-gray"
        }`}
      />
      {ready ? t("footer.engine.ready") : t("footer.engine.missing")}
    </span>
  );
};

export default EngineStatus;
