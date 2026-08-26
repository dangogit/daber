import { describe, expect, test } from "bun:test";
import { canCompleteOnboarding, canStartDictation } from "./onboarding.ts";

test("dictation instructions stay hidden until the model and controls are ready", () => {
  expect(canStartDictation({ modelReady: true, controlsReady: false })).toBe(
    false,
  );
  expect(canStartDictation({ modelReady: false, controlsReady: true })).toBe(
    false,
  );
  expect(canStartDictation({ modelReady: true, controlsReady: true })).toBe(
    true,
  );
});

describe("canCompleteOnboarding", () => {
  test("requires a ready model and a successful real dictation", () => {
    expect(
      canCompleteOnboarding({
        modelReady: false,
        dictationSucceeded: false,
        pasteSucceeded: false,
      }),
    ).toBe(false);
    expect(
      canCompleteOnboarding({
        modelReady: true,
        dictationSucceeded: false,
        pasteSucceeded: false,
      }),
    ).toBe(false);
    expect(
      canCompleteOnboarding({
        modelReady: false,
        dictationSucceeded: true,
        pasteSucceeded: true,
      }),
    ).toBe(false);
    expect(
      canCompleteOnboarding({
        modelReady: true,
        dictationSucceeded: true,
        pasteSucceeded: false,
      }),
    ).toBe(false);
    expect(
      canCompleteOnboarding({
        modelReady: true,
        dictationSucceeded: true,
        pasteSucceeded: true,
      }),
    ).toBe(true);
  });
});
