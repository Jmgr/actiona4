#!/usr/bin/env bash
set -euo pipefail

# ============================================================
# Constants
# ============================================================
readonly APP_NAME="actiona-run"
readonly GITHUB_REPO="Jmgr/actiona4"
readonly GITHUB_RELEASES_BASE_URL="https://github.com/${GITHUB_REPO}/releases/latest/download"
readonly SUPPORTED_OS="Linux"
readonly SUPPORTED_ARCH="x86_64"
readonly TARGET_TRIPLE="${SUPPORTED_ARCH}-unknown-linux-gnu"
readonly ARCHIVE_FILENAME="${TARGET_TRIPLE}.tar.gz"
readonly DOWNLOAD_URL="${GITHUB_RELEASES_BASE_URL}/${ARCHIVE_FILENAME}"
readonly DEFAULT_INSTALL_DIR="${HOME}/.local/share/${APP_NAME}"
readonly SYMLINK_DIR="${HOME}/.local/bin"

# ============================================================
# Output helpers
# ============================================================
# Use printf instead of echo -e for POSIX portability
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color / reset

info()  { printf "${GREEN}%s${NC}\n" "$*"; }
warn()  { printf "${YELLOW}Warning: %s${NC}\n" "$*" >&2; }
error() { printf "${RED}Error: %s${NC}\n" "$*" >&2; exit 1; }

# ============================================================
# Platform checks
# ============================================================
OS="$(uname -s)"
ARCH="$(uname -m)"

if [[ "${OS}" == "Darwin" ]]; then
    error "macOS is not supported."
fi

if [[ "${OS}" != "${SUPPORTED_OS}" ]]; then
    error "Unsupported operating system: ${OS}. Only Linux is supported."
fi

if [[ "${ARCH}" != "${SUPPORTED_ARCH}" ]]; then
    error "Unsupported architecture: ${ARCH}. Only ${SUPPORTED_ARCH} is currently supported."
fi

# ============================================================
# Check for a download tool (curl preferred, wget as fallback)
# ============================================================
if command -v curl &>/dev/null; then
    DOWNLOADER="curl"
elif command -v wget &>/dev/null; then
    DOWNLOADER="wget"
else
    error "Neither curl nor wget is available. Please install one and try again."
fi

# ============================================================
# Determine install directory
# Set ACTIONA_INSTALL_DIR in the environment to override the default.
# ============================================================
INSTALL_DIR="${ACTIONA_INSTALL_DIR:-${DEFAULT_INSTALL_DIR}}"
info "Installing ${APP_NAME} to: ${INSTALL_DIR}"

mkdir -p "${INSTALL_DIR}"

# ============================================================
# Download the release archive into a temporary directory
# The temp dir is cleaned up automatically on exit (success or failure).
# ============================================================
TEMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TEMP_DIR}"' EXIT

TEMP_ARCHIVE="${TEMP_DIR}/${ARCHIVE_FILENAME}"

info "Downloading ${ARCHIVE_FILENAME} ..."
if [[ "${DOWNLOADER}" == "curl" ]]; then
    curl --fail --location --progress-bar --output "${TEMP_ARCHIVE}" "${DOWNLOAD_URL}"
else
    wget --output-document="${TEMP_ARCHIVE}" "${DOWNLOAD_URL}"
fi

# ============================================================
# Extract the archive into the install directory
# ============================================================
info "Extracting to ${INSTALL_DIR} ..."
tar --extract --gzip --file="${TEMP_ARCHIVE}" --directory="${INSTALL_DIR}"

# ============================================================
# Run the first-time setup wizard (configures update checks, telemetry, etc.)
# ============================================================
info "Running initial setup ..."
"${INSTALL_DIR}/${APP_NAME}" setup

info "Installation complete."
