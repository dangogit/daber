import React from "react";
import { useTranslation } from "react-i18next";
import { ArrowLeft, ArrowRight } from "lucide-react";
import DaberTextLogo from "../icons/DaberTextLogo";
import { ShortcutInput } from "../settings/ShortcutInput";
import { PushToTalk } from "../settings/PushToTalk";
import { getLanguageDirection } from "@/lib/utils/rtl";

interface HotkeyStepProps {
  onContinue: () => void;
}

/**
 * The one thing someone has to remember to use this app at all.
 *
 * It reuses the settings control rather than a bespoke one, so the key someone
 * picks here behaves exactly like changing it later does — same validation,
 * same conflict handling, same reset.
 */
const HotkeyStep: React.FC<HotkeyStepProps> = ({ onContinue }) => {
  const { t, i18n } = useTranslation();
  const Arrow =
    getLanguageDirection(i18n.language) === "rtl" ? ArrowLeft : ArrowRight;

  return (
    <div className="h-full w-full flex flex-col items-center justify-center gap-6 p-6">
      <DaberTextLogo width={200} />

      <div className="max-w-md w-full flex flex-col gap-4">
        <div className="text-center">
          <h2 className="text-xl font-semibold text-text mb-2">
            {t("onboarding.hotkey.title")}
          </h2>
          <p className="text-text/70">{t("onboarding.hotkey.description")}</p>
        </div>

        <div className="glass rounded-xl divide-y divide-mid-gray/10">
          <ShortcutInput shortcutId="transcribe" grouped={true} />
          <PushToTalk descriptionMode="tooltip" grouped={true} />
        </div>

        <button
          onClick={onContinue}
          className="self-center flex items-center gap-2 px-5 py-2.5 rounded-lg bg-logo-primary hover:bg-logo-primary/90 text-white font-medium transition-colors"
        >
          {t("onboarding.continue")}
          <Arrow className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
};

export default HotkeyStep;
