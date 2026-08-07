import React, { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Check, Loader2, Mic, RotateCcw } from "lucide-react";
import DiburTextLogo from "../icons/DiburTextLogo";
import { useSettings } from "@/hooks/useSettings";
import { useOsType } from "@/hooks/useOsType";
import { formatKeyCombination } from "@/lib/utils/keyboard";
import type { HebrewModelState } from "@/hooks/useHebrewModel";

interface TryItStepProps {
  model: HebrewModelState;
  onDone: () => void;
}

/**
 * Onboarding ends with the user having dictated something, not with a progress
 * bar reaching 100%.
 *
 * The box below is a plain focused textarea, which means the transcription
 * arrives in it by exactly the same route it will arrive in any other app: the
 * shortcut fires, the text is pasted into whatever has focus. So this is a real
 * end-to-end run of microphone, permissions, model and paste, not a simulation
 * of one, and a failure here is a failure worth catching before someone tries
 * it in the middle of a conversation.
 */
const TryItStep: React.FC<TryItStepProps> = ({ model, onDone }) => {
  const { t } = useTranslation();
  const { getSetting } = useSettings();
  const osType = useOsType();
  const [text, setText] = useState("");
  const boxRef = useRef<HTMLTextAreaElement>(null);

  const binding = getSetting("bindings")?.transcribe?.current_binding ?? "";
  const shortcut = formatKeyCombination(binding, osType);
  const ready = model.status === "ready";
  const spoke = text.trim().length > 0;

  // Focus has to be in the box for the paste to land there, and it is easy to
  // lose to a stray click while someone is reading the instructions.
  useEffect(() => {
    if (ready) boxRef.current?.focus();
  }, [ready]);

  return (
    <div className="h-full w-full flex flex-col items-center justify-center gap-6 p-6">
      <DiburTextLogo width={200} />

      <div className="max-w-md w-full flex flex-col gap-4">
        <div className="text-center">
          <h2 className="text-xl font-semibold text-text mb-2">
            {spoke ? t("onboarding.try.worked") : t("onboarding.try.title")}
          </h2>
          <p className="text-text/70">
            {spoke ? (
              t("onboarding.try.workedDescription")
            ) : ready ? (
              <>
                {t("onboarding.try.press")}{" "}
                <kbd className="px-2 py-0.5 mx-0.5 rounded-md bg-text/10 font-mono text-sm text-text">
                  {shortcut}
                </kbd>{" "}
                {t("onboarding.try.andSpeak")}
              </>
            ) : (
              t("onboarding.try.stillPreparing")
            )}
          </p>
        </div>

        <textarea
          ref={boxRef}
          value={text}
          onChange={(e) => setText(e.target.value)}
          disabled={!ready}
          dir="auto"
          rows={4}
          placeholder={ready ? t("onboarding.try.placeholder") : ""}
          className="glass rounded-xl p-4 w-full resize-none text-text placeholder:text-text/30 outline-none focus:ring-2 focus:ring-logo-primary/50 disabled:opacity-50"
        />

        {!ready && model.status !== "failed" && (
          <div className="flex flex-col gap-2">
            <div className="flex items-center justify-between text-sm text-text/60">
              <span className="flex items-center gap-2">
                <Loader2 className="w-4 h-4 animate-spin" />
                {model.status === "downloading"
                  ? t("onboarding.try.downloading")
                  : model.status === "verifying"
                    ? t("onboarding.try.verifying")
                    : // Downloaded, now being loaded onto the GPU. Several
                      // seconds for a model this size, and saying "downloading
                      // 100%" through all of it looks stuck.
                      t("onboarding.try.preparing")}
              </span>
              <span className="font-mono">
                {model.speed
                  ? t("onboarding.try.speed", {
                      percent: Math.round(model.percentage),
                      speed: model.speed.toFixed(1),
                    })
                  : `${Math.round(model.percentage)}%`}
              </span>
            </div>
            <div className="h-1.5 w-full rounded-full bg-text/10 overflow-hidden">
              <div
                className="h-full bg-logo-primary transition-[width] duration-300"
                style={{ width: `${model.percentage}%` }}
              />
            </div>
          </div>
        )}

        {model.status === "failed" && (
          <div className="flex items-center justify-between gap-3 text-sm">
            <span className="text-error">{t("onboarding.try.failed")}</span>
            <button
              onClick={model.retry}
              className="flex items-center gap-1.5 font-medium text-logo-primary hover:underline"
            >
              <RotateCcw className="w-4 h-4" />
              {t("onboarding.try.retryDownload")}
            </button>
          </div>
        )}

        <div className="flex items-center justify-center gap-4">
          <button
            onClick={onDone}
            className={`flex items-center gap-2 px-5 py-2.5 rounded-lg font-medium transition-colors ${
              spoke
                ? "bg-logo-primary hover:bg-logo-primary/90 text-white"
                : "text-text/60 hover:text-text"
            }`}
          >
            {spoke ? (
              <>
                <Check className="w-4 h-4" />
                {t("onboarding.try.finish")}
              </>
            ) : (
              <>
                <Mic className="w-4 h-4" />
                {t("onboarding.try.skip")}
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};

export default TryItStep;
