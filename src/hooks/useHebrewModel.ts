import { useCallback, useEffect, useRef, useState } from "react";
import type { ModelInfo } from "@/bindings";
import {
  HEBREW_MODEL_ID,
  LOCAL_POLISH_MODEL_ID,
  LOCAL_POLISH_MODEL_SIZE_BYTES,
} from "@/lib/constants/models";
import { useModelStore } from "@/stores/modelStore";
import { commands } from "@/bindings";

export type HebrewModelStatus =
  | "checking"
  | "downloading"
  | "verifying"
  | "ready"
  | "failed";

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
  const [polisherReady, setPolisherReady] = useState(false);
  const [polisherFailed, setPolisherFailed] = useState(false);
  const [polisherDownloading, setPolisherDownloading] = useState(false);
  const [polisherAttempt, setPolisherAttempt] = useState(0);
  // Guards against the effect below firing a second download (or a second
  // select) while the first is still in flight.
  const startedRef = useRef(false);
  const selectingRef = useRef(false);

  const model = models.find((m: ModelInfo) => m.id === HEBREW_MODEL_ID);

  useEffect(() => {
    let cancelled = false;
    const prepare = async () => {
      setPolisherFailed(false);
      try {
        const initial = await commands.getLocalPolisherStatus();
        if (cancelled) return;
        if (
          initial.status === "ok" &&
          initial.data.model_downloaded &&
          initial.data.runtime_available
        ) {
          setPolisherReady(true);
          return;
        }
        if (initial.status !== "ok" || !initial.data.runtime_available) {
          setPolisherFailed(true);
          return;
        }

        setPolisherDownloading(true);
        const result = await commands.downloadLocalPolisherModel();
        if (cancelled) return;
        setPolisherDownloading(false);
        if (result.status !== "ok") {
          setPolisherFailed(true);
          return;
        }
        const finalStatus = await commands.getLocalPolisherStatus();
        if (
          finalStatus.status === "ok" &&
          finalStatus.data.model_downloaded &&
          finalStatus.data.runtime_available
        ) {
          setPolisherReady(true);
        } else {
          setPolisherFailed(true);
        }
      } catch (error) {
        if (cancelled) return;
        console.warn("Failed to prepare local text polish:", error);
        setPolisherDownloading(false);
        setPolisherFailed(true);
      }
    };
    void prepare();
    return () => {
      cancelled = true;
    };
  }, [polisherAttempt]);

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
    setPolisherReady(false);
    setPolisherFailed(false);
    setPolisherAttempt((attempt) => attempt + 1);
  }, []);

  let status: HebrewModelStatus = "checking";
  if (ready && polisherReady) status = "ready";
  else if (failed || polisherFailed) status = "failed";
  else if (
    HEBREW_MODEL_ID in verifyingModels ||
    LOCAL_POLISH_MODEL_ID in verifyingModels
  )
    status = "verifying";
  else if (
    HEBREW_MODEL_ID in downloadingModels ||
    polisherDownloading ||
    LOCAL_POLISH_MODEL_ID in downloadProgress
  )
    status = "downloading";

  const asrProgress = downloadProgress[HEBREW_MODEL_ID];
  const polishProgress = downloadProgress[LOCAL_POLISH_MODEL_ID];
  const asrTotal =
    asrProgress?.total ?? Math.round((model?.size_mb ?? 0) * 1024 * 1024);
  const polishTotal = polishProgress?.total ?? LOCAL_POLISH_MODEL_SIZE_BYTES;
  const downloaded =
    (asrProgress?.downloaded ??
      (ready || model?.is_downloaded ? asrTotal : (model?.partial_size ?? 0))) +
    (polishProgress?.downloaded ?? (polisherReady ? polishTotal : 0));
  const total = asrTotal + polishTotal;

  return {
    status,
    percentage:
      total > 0 ? Math.min(100, Math.round((downloaded / total) * 100)) : 0,
    speed:
      (downloadStats[HEBREW_MODEL_ID]?.speed ?? 0) +
        (downloadStats[LOCAL_POLISH_MODEL_ID]?.speed ?? 0) || null,
    retry,
  };
}
