#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
VERSION=${VERSION:-0.1.0}
ARCH=${ARCH:-amd64}
BINARY=${BINARY:-$ROOT/target/release/linux-clipboard-history}
STAGE=${STAGE:-$(mktemp -d "${TMPDIR:-/tmp}/linux-clipboard-history-deb.XXXXXX")}
OUTPUT=$ROOT/dist/linux-clipboard-history_${VERSION}_${ARCH}.deb
trap 'rm -rf "$STAGE"' EXIT HUP INT TERM
chmod 0755 "$STAGE"

if [ ! -x "$BINARY" ]; then
    echo "Release binary not found; building it with Cargo..."
    cargo build --release
fi

install -d "$STAGE/DEBIAN" "$STAGE/usr/bin" \
    "$STAGE/usr/share/applications" \
    "$STAGE/usr/share/icons/hicolor/scalable/apps" \
    "$STAGE/usr/lib/systemd/user" \
    "$STAGE/usr/share/doc/linux-clipboard-history" \
    "$ROOT/dist"

sed -e "s/^Version:.*/Version: $VERSION/" \
    -e "s/^Architecture:.*/Architecture: $ARCH/" \
    "$ROOT/packaging/deb/control" > "$STAGE/DEBIAN/control"
install -m 0755 "$ROOT/packaging/deb/postinst" "$STAGE/DEBIAN/postinst"
install -m 0755 "$ROOT/packaging/deb/postrm" "$STAGE/DEBIAN/postrm"
install -m 0755 "$BINARY" "$STAGE/usr/bin/linux-clipboard-history"
install -m 0644 "$ROOT/packaging/linux-clipboard-history.desktop" \
    "$STAGE/usr/share/applications/linux-clipboard-history.desktop"
install -m 0644 "$ROOT/packaging/linux-clipboard-history.svg" \
    "$STAGE/usr/share/icons/hicolor/scalable/apps/linux-clipboard-history.svg"
install -m 0644 "$ROOT/packaging/systemd-user.service" \
    "$STAGE/usr/lib/systemd/user/linux-clipboard-history.service"
install -m 0644 "$ROOT/README.md" "$STAGE/usr/share/doc/linux-clipboard-history/README.md"

dpkg-deb --root-owner-group --build "$STAGE" "$OUTPUT"
echo "$OUTPUT"
