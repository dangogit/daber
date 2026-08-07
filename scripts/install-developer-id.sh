#!/usr/bin/env bash
# Install a downloaded Developer ID Application certificate next to the private
# key that requested it, so codesign can find the pair.
#
#   ./scripts/install-developer-id.sh ~/Downloads/developerID_application.cer
#
# The private key was generated when the signing request was made and lives in
# the Keychain under `dibur-developerid-key`. A certificate on its own cannot
# sign anything — codesign needs both halves, which is what this pairs up.
set -euo pipefail

CER="${1:-}"
if [ ! -f "$CER" ]; then
  echo "Usage: $0 <path to downloaded .cer>" >&2
  exit 1
fi

ACCT="$USER"

# `security` hex-encodes any password containing a newline, which the PEM key does.
keychain() {
  local raw
  raw=$(security find-generic-password -a "$ACCT" -s "$1" -w)
  if [[ "$raw" =~ ^[0-9a-fA-F]+$ && $(( ${#raw} % 2 )) -eq 0 ]]; then
    printf '%s' "$raw" | xxd -r -p
  else
    printf '%s' "$raw"
  fi
}

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
chmod 700 "$WORK"

keychain dibur-developerid-key > "$WORK/key.pem"
openssl x509 -inform DER -in "$CER" -out "$WORK/cert.pem"

echo "Certificate: $(openssl x509 -in "$WORK/cert.pem" -noout -subject)"
echo "Expires:     $(openssl x509 -in "$WORK/cert.pem" -noout -enddate | cut -d= -f2)"

# Refuse early rather than importing a mismatched pair that fails at sign time.
if [ "$(openssl x509 -in "$WORK/cert.pem" -noout -pubkey)" != "$(openssl rsa -in "$WORK/key.pem" -pubout 2>/dev/null)" ]; then
  echo "This certificate was not issued for the stored private key." >&2
  echo "It has to come from the request in 'dibur-developerid-csr'." >&2
  exit 1
fi

openssl pkcs12 -export -inkey "$WORK/key.pem" -in "$WORK/cert.pem" \
  -out "$WORK/bundle.p12" -passout pass:

security import "$WORK/bundle.p12" \
  -k "$HOME/Library/Keychains/login.keychain-db" \
  -P '' -T /usr/bin/codesign

echo
security find-identity -v -p codesigning

echo
echo "If a 'Developer ID Application' line appears above, run:"
echo "  ./scripts/build-signed.sh"
