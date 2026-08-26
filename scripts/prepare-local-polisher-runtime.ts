import {
  access,
  chmod,
  copyFile,
  mkdir,
  readdir,
  readFile,
  realpath,
  rm,
  writeFile,
} from "node:fs/promises";
import { basename, join, resolve } from "node:path";

const LLAMA_VERSION = "b10621";
const QWEN_LICENSE_URL =
  "https://huggingface.co/Qwen/Qwen3-4B-GGUF/resolve/bc640142c66e1fdd12af0bd68f40445458f3869b/LICENSE";
const QWEN_LICENSE_SHA256 =
  "5de36594c10839788a8c589443a8ef9d8b8d17c65a1b5807206ae037fc36c6bd";
const RELEASE_BASE = `https://github.com/ggml-org/llama.cpp/releases/download/${LLAMA_VERSION}`;

type RuntimeArchive = {
  archive: string;
  sha256: string;
  server: string;
};

const archives: Record<string, RuntimeArchive> = {
  "aarch64-apple-darwin": {
    archive: `llama-${LLAMA_VERSION}-bin-macos-arm64.tar.gz`,
    sha256: "429c8270608600188035e5e92f7d78dffb7900904fe7dd7e6a84f48068cd13cf",
    server: "llama-server",
  },
  "x86_64-apple-darwin": {
    archive: `llama-${LLAMA_VERSION}-bin-macos-x64.tar.gz`,
    sha256: "33c44e036e0e223f71a29fc74a0ab3e130ca9eadeb032ecc1c7af25985b8b91b",
    server: "llama-server",
  },
  "x86_64-pc-windows-msvc": {
    archive: `llama-${LLAMA_VERSION}-bin-win-cpu-x64.zip`,
    sha256: "0e8b65e650e369f70f8307d890508886f171ef4fb00facccddd4a1b7ffdaca51",
    server: "llama-server.exe",
  },
  "aarch64-pc-windows-msvc": {
    archive: `llama-${LLAMA_VERSION}-bin-win-cpu-arm64.zip`,
    sha256: "c072e8bb057751587243c1e0ed28d82e23c7e0544a426e0d476f1e77792bf3ce",
    server: "llama-server.exe",
  },
  "x86_64-unknown-linux-gnu": {
    archive: "llama-" + LLAMA_VERSION + "-bin-ubuntu-x64.tar.gz",
    sha256: "91d7b03ddae498a39f28fdb85d84d2b4a0fd3838d10b4f897e0ef8975bb9b583",
    server: "llama-server",
  },
  "aarch64-unknown-linux-gnu": {
    archive: "llama-" + LLAMA_VERSION + "-bin-ubuntu-arm64.tar.gz",
    sha256: "95940151be63492f70f659da420b268244cc83a6ee70e310d2600ccdb7ea4deb",
    server: "llama-server",
  },
};

function hostTarget(): string {
  if (process.platform === "darwin") {
    return process.arch === "arm64"
      ? "aarch64-apple-darwin"
      : "x86_64-apple-darwin";
  }
  if (process.platform === "win32") {
    return process.arch === "arm64"
      ? "aarch64-pc-windows-msvc"
      : "x86_64-pc-windows-msvc";
  }
  if (process.platform === "linux") {
    return process.arch === "arm64"
      ? "aarch64-unknown-linux-gnu"
      : "x86_64-unknown-linux-gnu";
  }
  return "";
}

function selectedTarget(): string {
  const flagIndex = process.argv.indexOf("--target");
  return (
    (flagIndex >= 0 ? process.argv[flagIndex + 1] : undefined) ||
    process.env.DIBUR_BUILD_TARGET ||
    process.env.TAURI_ENV_TARGET_TRIPLE ||
    hostTarget()
  );
}

async function run(command: string[]): Promise<void> {
  const child = Bun.spawn(command, { stdout: "inherit", stderr: "inherit" });
  const exitCode = await child.exited;
  if (exitCode !== 0) {
    throw new Error(`${command[0]} exited with ${exitCode}`);
  }
}

async function sha256(path: string): Promise<string> {
  const bytes = await readFile(path);
  const hasher = new Bun.CryptoHasher("sha256");
  hasher.update(bytes);
  return hasher.digest("hex");
}

async function findRuntimeRoot(
  directory: string,
  server: string,
): Promise<string> {
  const entries = await readdir(directory, { withFileTypes: true });
  if (entries.some((entry) => entry.isFile() && entry.name === server)) {
    return directory;
  }
  for (const entry of entries) {
    if (entry.isDirectory()) {
      const nested = join(directory, entry.name);
      try {
        return await findRuntimeRoot(nested, server);
      } catch {
        // Keep searching sibling directories.
      }
    }
  }
  throw new Error(`Could not find ${server} in extracted archive`);
}

const target = selectedTarget();
const runtime = archives[target];
if (!runtime) {
  console.log(`Local polisher runtime is not packaged for target '${target}'.`);
  process.exit(0);
}

const repoRoot = resolve(import.meta.dir, "..");
const outputDir = join(repoRoot, "src-tauri", "resources", "local-polisher");
const markerPath = join(outputDir, "runtime-version.txt");
const marker = LLAMA_VERSION + ":" + target + ":" + QWEN_LICENSE_SHA256 + "\n";
const requiredOutputFiles = [
  runtime.server,
  "LICENSE",
  "QWEN3-APACHE-2.0.txt",
  "THIRD-PARTY-NOTICE.txt",
];

try {
  const markerMatches = (await readFile(markerPath, "utf8")) === marker;
  const filesPresent = await Promise.all(
    requiredOutputFiles.map(async (filename) => {
      try {
        await access(join(outputDir, filename));
        return true;
      } catch {
        return false;
      }
    }),
  );
  if (markerMatches && filesPresent.every(Boolean)) {
    console.log(
      `llama.cpp runtime ${LLAMA_VERSION} already prepared for ${target}.`,
    );
    process.exit(0);
  }
} catch {
  // Missing marker means the runtime needs to be prepared.
}

const cacheDir = join(repoRoot, "src-tauri", "target", "local-polisher-cache");
const archivePath = join(cacheDir, runtime.archive);
const extractDir = join(cacheDir, `${target}-${LLAMA_VERSION}`);
await mkdir(cacheDir, { recursive: true });

let archiveValid = false;
try {
  archiveValid = (await sha256(archivePath)) === runtime.sha256;
} catch {
  archiveValid = false;
}

if (!archiveValid) {
  const providedArchive = process.env.DIBUR_LOCAL_POLISHER_ARCHIVE;
  if (providedArchive) {
    await copyFile(providedArchive, archivePath);
  } else {
    const response = await fetch(`${RELEASE_BASE}/${runtime.archive}`);
    if (!response.ok) {
      throw new Error(`Runtime download failed with HTTP ${response.status}`);
    }
    await Bun.write(archivePath, response);
  }
  const actualHash = await sha256(archivePath);
  if (actualHash !== runtime.sha256) {
    throw new Error(
      `Runtime SHA-256 mismatch: expected ${runtime.sha256}, got ${actualHash}`,
    );
  }
}

await rm(extractDir, { recursive: true, force: true });
await mkdir(extractDir, { recursive: true });
await run(["tar", "-xf", archivePath, "-C", extractDir]);

const runtimeRoot = await findRuntimeRoot(extractDir, runtime.server);
const files = await readdir(runtimeRoot, { withFileTypes: true });
const shouldCopy = (name: string): boolean =>
  name === runtime.server ||
  name.endsWith(".dll") ||
  name.endsWith(".dylib") ||
  name.includes(".so") ||
  name === "LICENSE" ||
  name.startsWith("LICENSE-");

await rm(outputDir, { recursive: true, force: true });
await mkdir(outputDir, { recursive: true });
for (const entry of files) {
  if ((entry.isFile() || entry.isSymbolicLink()) && shouldCopy(entry.name)) {
    const source = await realpath(join(runtimeRoot, entry.name));
    await copyFile(source, join(outputDir, basename(entry.name)));
  }
}
if (process.platform !== "win32") {
  await chmod(join(outputDir, runtime.server), 0o755);
}

const providedQwenLicense = process.env.DIBUR_QWEN_LICENSE;
let qwenLicense: Uint8Array;
if (providedQwenLicense) {
  qwenLicense = new Uint8Array(await readFile(providedQwenLicense));
} else {
  const qwenLicenseResponse = await fetch(QWEN_LICENSE_URL);
  if (!qwenLicenseResponse.ok) {
    throw new Error(
      "Qwen license download failed with HTTP " + qwenLicenseResponse.status,
    );
  }
  qwenLicense = new Uint8Array(await qwenLicenseResponse.arrayBuffer());
}
const qwenLicenseHasher = new Bun.CryptoHasher("sha256");
qwenLicenseHasher.update(qwenLicense);
const qwenLicenseHash = qwenLicenseHasher.digest("hex");
if (qwenLicenseHash !== QWEN_LICENSE_SHA256) {
  throw new Error(
    "Qwen license SHA-256 mismatch: expected " +
      QWEN_LICENSE_SHA256 +
      ", got " +
      qwenLicenseHash,
  );
}
await writeFile(join(outputDir, "QWEN3-APACHE-2.0.txt"), qwenLicense);
await writeFile(
  join(outputDir, "THIRD-PARTY-NOTICE.txt"),
  [
    "Dibur local text polish",
    "",
    "Qwen3-4B-GGUF model weights",
    "Copyright Qwen Team",
    "License: Apache License 2.0",
    "Source: https://huggingface.co/Qwen/Qwen3-4B-GGUF",
    "",
    "llama.cpp runtime " + LLAMA_VERSION,
    "Copyright ggml authors",
    "License: MIT",
    "Source: https://github.com/ggml-org/llama.cpp",
    "",
  ].join("\n"),
);
await writeFile(markerPath, marker);
console.log(`Prepared llama.cpp ${LLAMA_VERSION} runtime for ${target}.`);
