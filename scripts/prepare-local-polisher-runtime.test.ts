import { afterEach, describe, expect, test } from "bun:test";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { assertRequiredFiles } from "./local-polisher-files";

const temporaryDirectories: string[] = [];

afterEach(async () => {
  await Promise.all(
    temporaryDirectories
      .splice(0)
      .map((directory) => rm(directory, { recursive: true, force: true })),
  );
});

describe("local polisher runtime validation", () => {
  test("rejects a prepared runtime that is missing its main license", async () => {
    const directory = await mkdtemp(join(tmpdir(), "dibur-polisher-test-"));
    temporaryDirectories.push(directory);
    await writeFile(join(directory, "llama-server.exe"), "runtime");

    await expect(
      assertRequiredFiles(directory, ["llama-server.exe", "LICENSE"]),
    ).rejects.toThrow("LICENSE");
  });

  test("accepts a prepared runtime with every required file", async () => {
    const directory = await mkdtemp(join(tmpdir(), "dibur-polisher-test-"));
    temporaryDirectories.push(directory);
    await writeFile(join(directory, "llama-server.exe"), "runtime");
    await writeFile(join(directory, "LICENSE"), "license");

    await expect(
      assertRequiredFiles(directory, ["llama-server.exe", "LICENSE"]),
    ).resolves.toBeUndefined();
  });
});
