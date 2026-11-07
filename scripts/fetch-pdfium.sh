#!/usr/bin/env bash
# Downloads the pdfium library Tinta reads PDFs with into vendor/pdfium/.
#
# The version is pinned to the pdfium API that pdfium-render is compiled
# against (feature `pdfium_8125` in crates/tinta-pdf/Cargo.toml): change both
# together. Binaries come from https://github.com/bblanchon/pdfium-binaries.
#
# Usage: scripts/fetch-pdfium.sh [platform]
#   platform defaults to the current one: linux-x64, linux-arm64, mac-univ,
#   win-x64 or win-arm64.

set -euo pipefail

readonly PDFIUM_VERSION="8125"
readonly RELEASES="https://github.com/bblanchon/pdfium-binaries/releases/download"

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
destination="$root/vendor/pdfium"

detect_platform() {
  local machine
  machine="$(uname -m)"
  case "$(uname -s)" in
    Linux)
      [[ "$machine" == "aarch64" ]] && echo "linux-arm64" || echo "linux-x64" ;;
    Darwin)
      echo "mac-univ" ;;
    MINGW* | MSYS* | CYGWIN*)
      [[ "$machine" == "aarch64" || "$machine" == "arm64" ]] && echo "win-arm64" || echo "win-x64" ;;
    *)
      echo "Sistema no soportado: $(uname -s)" >&2
      exit 1 ;;
  esac
}

library_path() {
  case "$1" in
    linux-*) echo "lib/libpdfium.so" ;;
    mac-*) echo "lib/libpdfium.dylib" ;;
    win-*) echo "bin/pdfium.dll" ;;
  esac
}

platform="${1:-$(detect_platform)}"
library="$(library_path "${platform}")"
stamp="${destination}/.version-${PDFIUM_VERSION}-${platform}"

if [[ -f "${stamp}" ]]; then
  echo "pdfium ${PDFIUM_VERSION} (${platform}) ya está en ${destination}"
  exit 0
fi

workdir="$(mktemp -d)"
trap 'rm -rf "${workdir}"' EXIT

echo "Descargando pdfium ${PDFIUM_VERSION} para ${platform}…"
curl --fail --silent --show-error --location \
  "${RELEASES}/chromium%2F${PDFIUM_VERSION}/pdfium-${platform}.tgz" \
  --output "${workdir}/pdfium.tgz"
tar -xzf "${workdir}/pdfium.tgz" -C "${workdir}"

rm -rf "${destination}"
mkdir -p "${destination}"
cp "${workdir}/${library}" "${destination}/"
cp "${workdir}/LICENSE" "${destination}/LICENSE.pdfium" 2>/dev/null || true
touch "${stamp}"
echo "Listo: ${destination}/$(basename "${library}")"
