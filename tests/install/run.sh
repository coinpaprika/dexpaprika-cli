#!/bin/sh
# Drives install.sh against local fixtures. A fake `curl` first in PATH serves
# files from a fixture directory, so nothing touches the network.
#
# Usage: tests/install/run.sh [path/to/install.sh]
# Exit status is the number of failed cases.

set -u

ROOT=$(cd "$(dirname "$0")/../.." && pwd)
INSTALL_SH=${1:-"$ROOT/install.sh"}
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

VERSION="v9.9.9"
BINARY="dexpaprika-cli"

case "$(uname -s)" in
    Linux) OS=linux ;;
    Darwin) OS=darwin ;;
    *) echo "skip: install.sh tests run on Linux and macOS only"; exit 0 ;;
esac
case "$(uname -m)" in
    x86_64 | amd64) ARCH=x86_64 ;;
    arm64 | aarch64) ARCH=aarch64 ;;
    *) echo "skip: unsupported architecture"; exit 0 ;;
esac
ARCHIVE="${BINARY}-${VERSION}-${OS}-${ARCH}.tar.gz"

sha256() {
    if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | awk '{print $1}'
    else shasum -a 256 "$1" | awk '{print $1}'; fi
}

# Fake curl: the URL's last path segment names a file in $FIXTURE; the GitHub
# API call gets latest.json. A missing file fails like `curl -f` (exit 22).
mkdir -p "$WORK/bin"
cat > "$WORK/bin/curl" <<'CURL'
#!/bin/sh
out=""; url=""
while [ $# -gt 0 ]; do
    case "$1" in
        -o) out=$2; shift ;;
        http*) url=$1 ;;
    esac
    shift
done
case "$url" in
    */releases/latest) src="$FIXTURE/latest.json" ;;
    *) src="$FIXTURE/${url##*/}" ;;
esac
[ -f "$src" ] || { echo "curl: (22) 404 $url" >&2; exit 22; }
if [ -n "$out" ]; then cp "$src" "$out"; else cat "$src"; fi
CURL
chmod +x "$WORK/bin/curl"

# A fresh fixture directory with a good archive and a matching SHA256SUMS.
new_fixture() {
    F="$WORK/fixture-$1"
    rm -rf "$F"; mkdir -p "$F/pack"
    printf '{"tag_name": "%s"}\n' "$VERSION" > "$F/latest.json"
    printf '#!/bin/sh\necho genuine\n' > "$F/pack/$BINARY"
    chmod +x "$F/pack/$BINARY"
    (cd "$F/pack" && tar czf "$F/$ARCHIVE" "$BINARY")
    printf '%s  %s\n' "$(sha256 "$F/$ARCHIVE")" "$ARCHIVE" > "$F/SHA256SUMS"
}

FAILED=0
run_case() { # name, expect (ok|refuse)
    name=$1; expect=$2
    home="$WORK/home-$name"; rm -rf "$home"; mkdir -p "$home"
    FIXTURE="$F" HOME="$home" DEXPAPRIKA_INSTALL_DIR="$home/bin" PATH="$WORK/bin:$PATH" \
        sh "$INSTALL_SH" > "$WORK/out-$name" 2>&1
    status=$?
    installed=no; [ -f "$home/bin/$BINARY" ] && installed=yes
    if [ "$expect" = ok ] && [ $status -eq 0 ] && [ $installed = yes ]; then
        echo "ok   $name"
    elif [ "$expect" = refuse ] && [ $status -ne 0 ] && [ $installed = no ]; then
        echo "ok   $name"
    else
        echo "FAIL $name: expected $expect, exit $status, installed=$installed"
        sed 's/^/     | /' "$WORK/out-$name"
        FAILED=$((FAILED + 1))
    fi
}

# 1. A genuine release installs.
new_fixture good
run_case good ok

# 2. An archive that does not match SHA256SUMS is refused.
new_fixture tampered
printf '#!/bin/sh\necho tampered\n' > "$F/pack/$BINARY"
(cd "$F/pack" && tar czf "$F/$ARCHIVE" "$BINARY")
run_case tampered refuse

# 3. A release without SHA256SUMS is refused.
new_fixture no-sums
rm "$F/SHA256SUMS"
run_case no-sums refuse

# 4. SHA256SUMS without a line for this archive is refused.
new_fixture no-entry
printf '%s  %s\n' "$(sha256 "$F/$ARCHIVE")" "some-other-file.tar.gz" > "$F/SHA256SUMS"
run_case no-entry refuse

# 5. A correctly summed archive with a '..' member is refused before extraction.
# -P keeps the '..': GNU tar strips it from member names otherwise, which
# would hand install.sh a harmless archive and prove nothing.
new_fixture dotdot
printf 'x\n' > "$F/outside"
(cd "$F/pack" && tar czPf "$F/$ARCHIVE" "$BINARY" "../outside")
tar tzf "$F/$ARCHIVE" | grep -qx '\.\./outside' || { echo "FAIL dotdot: fixture lost its '..' member"; exit 1; }
printf '%s  %s\n' "$(sha256 "$F/$ARCHIVE")" "$ARCHIVE" > "$F/SHA256SUMS"
run_case dotdot refuse

# 6. A correctly summed archive with an absolute member is refused.
new_fixture absolute
printf 'x\n' > "$F/outside"
(cd "$F/pack" && tar czPf "$F/$ARCHIVE" "$BINARY" "$F/outside")
printf '%s  %s\n' "$(sha256 "$F/$ARCHIVE")" "$ARCHIVE" > "$F/SHA256SUMS"
run_case absolute refuse

# 7. A correctly summed archive with a symlink is refused.
new_fixture symlink
(cd "$F/pack" && rm "$BINARY" && ln -s /bin/sh "$BINARY" && tar czf "$F/$ARCHIVE" "$BINARY")
printf '%s  %s\n' "$(sha256 "$F/$ARCHIVE")" "$ARCHIVE" > "$F/SHA256SUMS"
run_case symlink refuse

# 8. A release tag that is not a version is refused.
new_fixture bad-tag
printf '{"tag_name": "%s"}\n' 'v1/../../x' > "$F/latest.json"
run_case bad-tag refuse

exit $FAILED
