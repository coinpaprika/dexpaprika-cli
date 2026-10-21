#!/bin/sh
# DexPaprika CLI installer
# Usage: curl -sSL https://raw.githubusercontent.com/coinpaprika/dexpaprika-cli/main/install.sh | sh
#
# Detects your OS/arch, downloads the latest release binary from GitHub, checks
# it against the SHA256SUMS file published with the same release, and drops it
# into ~/.local/bin (or /usr/local/bin with sudo).

set -e

REPO="coinpaprika/dexpaprika-cli"
BINARY="dexpaprika-cli"
INSTALL_DIR="${DEXPAPRIKA_INSTALL_DIR:-$HOME/.local/bin}"

# ---------------------------------------------------------------------------
# helpers
# ---------------------------------------------------------------------------
info()  { printf '\033[1;34m%s\033[0m\n' "$*"; }
ok()    { printf '\033[1;32m%s\033[0m\n' "$*"; }
err()   { printf '\033[1;31mError: %s\033[0m\n' "$*" >&2; exit 1; }

need() {
    command -v "$1" >/dev/null 2>&1 || err "'$1' is required but not found. Install it and try again."
}

# ---------------------------------------------------------------------------
# detect platform
# ---------------------------------------------------------------------------
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)  OS="linux"  ;;
        Darwin) OS="darwin" ;;
        MINGW*|MSYS*|CYGWIN*) OS="windows" ;;
        *) err "Unsupported OS: $OS" ;;
    esac

    case "$ARCH" in
        x86_64|amd64)  ARCH="x86_64"  ;;
        arm64|aarch64) ARCH="aarch64" ;;
        *) err "Unsupported architecture: $ARCH" ;;
    esac

    PLATFORM="${OS}-${ARCH}"
}

# ---------------------------------------------------------------------------
# find latest release tag
# ---------------------------------------------------------------------------
get_latest_version() {
    need curl
    VERSION=$(curl -sSf "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"//;s/".*//')

    [ -z "$VERSION" ] && err "Could not determine latest version. Check https://github.com/${REPO}/releases"
    # The tag goes into download URLs and file names; accept a version tag and
    # nothing else.
    case "$VERSION" in
        v[0-9]*) ;;
        *) err "Unexpected release tag. Check https://github.com/${REPO}/releases" ;;
    esac
    case "$VERSION" in
        *[!A-Za-z0-9._+-]*) err "Unexpected release tag. Check https://github.com/${REPO}/releases" ;;
    esac
    info "Latest version: $VERSION"
}

# ---------------------------------------------------------------------------
# integrity
# ---------------------------------------------------------------------------
sha256_of() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | awk '{print $1}'
    elif command -v openssl >/dev/null 2>&1; then
        openssl dgst -sha256 "$1" | awk '{print $NF}'
    else
        err "Need sha256sum, shasum or openssl to verify the download. Install one and try again."
    fi
}

# Refuse the archive unless its SHA-256 matches the line for it in SHA256SUMS.
verify_checksum() {
    SUMS="${WORK_DIR}/SHA256SUMS"
    curl -sSfL "${BASE_URL}/SHA256SUMS" -o "$SUMS" \
        || err "Could not download SHA256SUMS for ${VERSION}, so the download cannot be verified. Nothing was installed."
    EXPECTED=$(awk -v f="$ARCHIVE" '$2 == f || $2 == "*" f { print tolower($1); exit }' "$SUMS")
    [ -n "$EXPECTED" ] || err "SHA256SUMS for ${VERSION} has no entry for ${ARCHIVE}. Nothing was installed."
    ACTUAL=$(sha256_of "${WORK_DIR}/${ARCHIVE}" | tr 'A-F' 'a-f')
    [ "$ACTUAL" = "$EXPECTED" ] || err "Checksum mismatch for ${ARCHIVE} (expected ${EXPECTED}, got ${ACTUAL}). Nothing was installed."
    info "Checksum verified."
}

# Refuse an archive with a member that could land outside the directory it is
# extracted into: an absolute path, a '..' component, a backslash, or a link.
check_members() {
    if [ "$EXT" = "zip" ]; then
        MEMBERS=$(unzip -Z1 "$1") || err "Could not list the archive."
    else
        MEMBERS=$(tar tzf "$1") || err "Could not list the archive."
        if tar tvzf "$1" | grep -q '^[lh]'; then
            err "The archive contains a link. Nothing was installed."
        fi
    fi
    printf '%s\n' "$MEMBERS" | while IFS= read -r m; do
        case "$m" in
            /* | .. | ../* | */.. | */../* | *\\*)
                echo "$m"
                ;;
        esac
    done | grep -q . && err "The archive contains a path outside its own directory. Nothing was installed."
    return 0
}

# ---------------------------------------------------------------------------
# download & install
# ---------------------------------------------------------------------------
download_and_install() {
    EXT="tar.gz"
    [ "$OS" = "windows" ] && EXT="zip"

    ARCHIVE="${BINARY}-${VERSION}-${PLATFORM}.${EXT}"
    BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"
    URL="${BASE_URL}/${ARCHIVE}"

    WORK_DIR=$(mktemp -d)
    trap 'rm -rf "$WORK_DIR"' EXIT

    info "Downloading ${URL}..."
    curl -sSfL "$URL" -o "${WORK_DIR}/${ARCHIVE}" || err "Download failed. Check that a release exists for your platform (${PLATFORM}).\nURL: ${URL}"

    verify_checksum

    [ "$EXT" = "zip" ] && need unzip
    check_members "${WORK_DIR}/${ARCHIVE}"

    # Extract into a fresh directory of its own, never next to anything else.
    info "Extracting..."
    EXTRACT_DIR=$(mktemp -d "${WORK_DIR}/extract.XXXXXX")
    if [ "$EXT" = "zip" ]; then
        unzip -q "${WORK_DIR}/${ARCHIVE}" -d "$EXTRACT_DIR"
    else
        tar xzf "${WORK_DIR}/${ARCHIVE}" -C "$EXTRACT_DIR"
    fi

    # The release packs the binary at the top level of the archive.
    BIN_PATH="${EXTRACT_DIR}/${BINARY}"
    [ "$EXT" = "zip" ] && BIN_PATH="${EXTRACT_DIR}/${BINARY}.exe"
    if [ ! -f "$BIN_PATH" ] || [ -L "$BIN_PATH" ]; then
        err "Could not find '${BINARY}' at the top level of the archive"
    fi

    chmod +x "$BIN_PATH"

    # Install
    mkdir -p "$INSTALL_DIR"
    if [ -w "$INSTALL_DIR" ]; then
        mv "$BIN_PATH" "${INSTALL_DIR}/${BINARY}"
    else
        info "Need sudo to write to ${INSTALL_DIR}"
        sudo mv "$BIN_PATH" "${INSTALL_DIR}/${BINARY}"
    fi

    ok "Installed ${BINARY} to ${INSTALL_DIR}/${BINARY}"
}

# ---------------------------------------------------------------------------
# check PATH
# ---------------------------------------------------------------------------
check_path() {
    case ":$PATH:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            printf '\n'
            info "Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
            printf '\n  export PATH="%s:$PATH"\n\n' "$INSTALL_DIR"
            ;;
    esac
}

# ---------------------------------------------------------------------------
# main
# ---------------------------------------------------------------------------
main() {
    info "Installing DexPaprika CLI..."
    printf '\n'

    detect_platform
    info "Platform: ${PLATFORM}"

    get_latest_version
    download_and_install
    check_path

    printf '\n'
    ok "Done! Run 'dexpaprika-cli --help' to get started."
    printf '\n'
    printf '  No API key needed to start. Just data.\n'
    printf '  Run: dexpaprika-cli onboard\n'
    printf '\n'
}

main
