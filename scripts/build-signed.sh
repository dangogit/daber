#!/usr/bin/env bash
# Build Daber signed with a Developer ID, notarized by Apple, and stapled.
#
# Credentials come from the macOS Keychain, never from a file or the shell
# history. See "Signing and notarizing it properly" in README.md for how to put
# them there. Without them, plain `bun run tauri build` still works — it just
# produces the ad-hoc signed build that warns on first open.
#
# Notarization authenticates with an App Store Connect API key rather than an
# Apple ID and app-specific password: the key is scoped to one team, it is not
# tied to a person's account, and revoking it does not touch anything else.
set -euo pipefail

cd "$(dirname "$0")/.."

APP="src-tauri/target/release/bundle/macos/Daber.app"
ACCT="$USER"

# `security` hex-encodes any password containing a newline, which the PEM key does.
keychain() {
  local raw
  if ! raw=$(security find-generic-password -a "$ACCT" -s "$1" -w 2>/dev/null); then
    echo "Missing Keychain item '$1'." >&2
    echo "See README.md → Signing and notarizing it properly." >&2
    exit 1
  fi
  if [[ "$raw" =~ ^[0-9a-fA-F]+$ && $(( ${#raw} % 2 )) -eq 0 ]]; then
    printf '%s' "$raw" | xxd -r -p
  else
    printf '%s' "$raw"
  fi
}

# `|| true` because grep exits 1 when there is no such identity, and under
# `set -e` that would kill the script before the explanation below is printed.
IDENTITY=$(security find-identity -v -p codesigning \
  | grep "Developer ID Application" \
  | head -1 \
  | sed -E 's/.*"(.*)"$/\1/' || true)

if [ -z "$IDENTITY" ]; then
  echo "No 'Developer ID Application' certificate is installed on this Mac." >&2
  echo "Apple Development and Apple Distribution certificates cannot notarize;" >&2
  echo "only the Account Holder of the team can create a Developer ID one." >&2
  echo "See README.md → Signing and notarizing it properly." >&2
  exit 1
fi

export APPLE_SIGNING_IDENTITY="$IDENTITY"
echo "Signing as: $IDENTITY"

# The key only touches disk here, at 0600, and is shredded on the way out.
KEYFILE=$(mktemp -t asc-key)
trap 'rm -P "$KEYFILE" 2>/dev/null || rm -f "$KEYFILE"' EXIT
chmod 600 "$KEYFILE"
keychain daber-asc-api-key > "$KEYFILE"
KEY_ID=$(keychain daber-asc-key-id)
ISSUER_ID=$(keychain daber-asc-issuer-id)

notarize() {
  xcrun notarytool submit "$1" \
    --key "$KEYFILE" --key-id "$KEY_ID" --issuer "$ISSUER_ID" \
    --wait
}

# transcribe-cpp's CMake build needs this on current CMake releases.
export CMAKE_POLICY_VERSION_MINIMUM=3.5

bun run tauri build "$@"

# The .app is notarized inside a zip because notarytool does not accept a bare
# bundle; the ticket it issues is stapled to the bundle itself afterwards.
echo
echo "==> Notarizing the app"
ZIP=$(mktemp -t daber-app -u).zip
ditto -c -k --keepParent "$APP" "$ZIP"
notarize "$ZIP"
rm -f "$ZIP"
xcrun stapler staple "$APP"

DMG=$(ls -t src-tauri/target/release/bundle/dmg/*.dmg 2>/dev/null | head -1)
if [ -n "$DMG" ]; then
  echo
  echo "==> Notarizing the disk image"
  notarize "$DMG"
  xcrun stapler staple "$DMG"
fi

echo
echo "==> Gatekeeper verdict"
spctl -a -vvv -t install "$APP"
[ -n "$DMG" ] && spctl -a -vvv -t open --context context:primary-signature "$DMG"

echo
echo "Done. Ship: $DMG"
