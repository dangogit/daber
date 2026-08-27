import { useCallback, useEffect, useRef, useState } from "react";
import type { ModelInfo } from "@/bindings";
import { HEBREW_MODEL_ID } from "@/lib/constants/models";
import { useModelStore } from "@/stores/modelStore";
import { commands } from "@/bindings";

export type HebrewModelStatus =
  "checking" | "downloading" | "verifying" | "ready" | "failed";

export interface HebrewModelState {
  status: HebrewModelStatus;
  /** 0-100 while downloading, otherwise 0. */
  percentage: number;
  /** MB/s, or null before the first progress event. */
  speed: number | null;
  retry: () => void;
}

/**
 * Owns the one model this app has.
 *
 * Dibur ships a single Hebrew engine, so there is nothing to choose and no
 * reason to make anyone wait at a picker. This starts the download the moment
 * the app opens and reports on it, which lets the rest of onboarding — granting
 * permissions, choosing a shortcut — happen over the top of it. By the time
 * someone reaches the point of actually speaking, the download has usually
 * finished without ever having been a screen of its own.
 */
export function useHebrewModel(): HebrewModelState {
  const {
    models,
    currentModel,
    downloadModel,
    selectModel,
    downloadingModels,
    verifyingModels,
    downloadProgress,
    downloadStats,
  } = useModelStore();

  const [failed, setFailed] = useState(false);
  const [ready, setReady] = useState(false);
  // Guards against the effect below firing a second download (or a second
  // select) while the first is still in flight.
  const startedRef = useRef(false);
  const selectingRef = useRef(false);

  const model = models.find((m: ModelInfo) => m.id === HEBREW_MODEL_ID);

  const start = useCallback(async () => {
    startedRef.current = true;
    setFailed(false);
    const ok = await downloadModel(HEBREW_MODEL_ID);
    if (!ok) {
      startedRef.current = false;
      setFailed(true);
    }
  }, [downloadModel]);

  useEffect(() => {
    if (!model || ready || failed) return;

    if (model.is_downloaded) {
      if (currentModel === HEBREW_MODEL_ID) {
        if (selectingRef.current) return;
        selectingRef.current = true;
        commands.getTranscriptionModelStatus().then(async (result) => {
          if (result.status === "ok" && result.data === HEBREW_MODEL_ID) {
            setReady(true);
          } else {
            const ok = await selectModel(HEBREW_MODEL_ID);
            if (ok) setReady(true);
            else setFailed(true);
          }
          selectingRef.current = false;
        });
        return;
      }
      if (selectingRef.current) return;
      selectingRef.current = true;
      selectModel(HEBREW_MODEL_ID).then((ok) => {
        selectingRef.current = false;
        if (ok) setReady(true);
        else setFailed(true);
      });
      return;
    }

    if (!startedRef.current && !model.is_downloading) {
      void start();
    }
  }, [model, currentModel, ready, failed, selectModel, start]);

  const retry = useCallback(() => {
    startedRef.current = false;
    setFailed(false);
  }, []);

  let status: HebrewModelStatus = "checking";
  if (ready) status = "ready";
  else if (failed) status = "failed";
  else if (HEBREW_MODEL_ID in verifyingModels) status = "verifying";
  else if (HEBREW_MODEL_ID in downloadingModels) status = "downloading";

  return {
    status,
    percentage: downloadProgress[HEBREW_MODEL_ID]?.percentage ?? 0,
    speed: downloadStats[HEBREW_MODEL_ID]?.speed ?? null,
    retry,
  };
}
