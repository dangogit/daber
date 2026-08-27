import { access } from "node:fs/promises";
import { join } from "node:path";

export async function assertRequiredFiles(
  directory: string,
  filenames: string[],
): Promise<void> {
  const missing: string[] = [];

  for (const filename of filenames) {
    try {
      await access(join(directory, filename));
    } catch {
      missing.push(filename);
    }
  }

  if (missing.length > 0) {
    throw new Error(
      `Prepared local polisher runtime is missing required files: ${missing.join(", ")}`,
    );
  }
}
