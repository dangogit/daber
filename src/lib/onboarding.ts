export interface OnboardingReadiness {
  modelReady: boolean;
  dictationSucceeded: boolean;
  pasteSucceeded: boolean;
}

export function canCompleteOnboarding({
  modelReady,
  dictationSucceeded,
  pasteSucceeded,
}: OnboardingReadiness): boolean {
  return modelReady && dictationSucceeded && pasteSucceeded;
}

export function canStartDictation({
  modelReady,
  controlsReady,
}: {
  modelReady: boolean;
  controlsReady: boolean;
}): boolean {
  return modelReady && controlsReady;
}
