#!/usr/bin/env bash
# Build Daber signed with a Developer ID and notarized by Apple.
#
# Credentials come from the macOS Keychain, never from a file or the shell
# history. See "Signing and notarizing it properly" in README.md for how to put
# them there. Without them, plain `bun run tauri build` still works — it just
# produces the ad-hoc signed build that warns on first open.
set -euo pipefail

need() {
  local service="$1" var="$2"
  local value
  if ! value=$(security find-generic-password -a "$USER" -s "$service" -w 2>/dev/null); then
    echo "Missing Keychain item '$service'." >&2
    echo "See README.md → Signing and notarizing it properly." >&2
    exit 1
  fi
  export "$var=$value"
}

need daber-apple-id APPLE_ID
need daber-apple-password APPLE_PASSWORD
need daber-apple-team-id APPLE_TEAM_ID
need daber-signing-identity APPLE_SIGNING_IDENTITY

if ! security find-identity -v -p codesigning | grep -q "Developer ID Application"; then
  echo "No 'Developer ID Application' certificate is installed on this Mac." >&2
  echo "An 'Apple Development' certificate cannot notarize. See README.md." >&2
  exit 1
fi

echo "Signing as: $APPLE_SIGNING_IDENTITY"
echo "Notarizing under team: $APPLE_TEAM_ID"
echo

# transcribe-cpp's CMake build needs this on current CMake releases.
export CMAKE_POLICY_VERSION_MINIMUM=3.5

bun run tauri build "$@"

echo
echo "Verify the result with:"
echo "  spctl -a -vvv -t install \"src-tauri/target/release/bundle/macos/Daber.app\""
