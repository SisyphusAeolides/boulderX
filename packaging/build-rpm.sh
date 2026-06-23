#!/usr/bin/env bash
# Build boulder-relay RPM on Rocky Linux / RHEL 9.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="$(grep '^version' "$ROOT/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')"
TARBALL="boulder-relay-${VERSION}.tar.gz"
RPMBUILD="${RPMBUILD:-$HOME/rpmbuild}"

echo "==> Building release binary (offline)..."
cd "$ROOT"
cargo build --release --offline

echo "==> Preparing source tarball..."
STAGING="$(mktemp -d)"
trap 'rm -rf "$STAGING"' EXIT
SRC_DIR="$STAGING/boulder-relay-$VERSION"
mkdir -p "$SRC_DIR"
tar -C "$ROOT" \
    --exclude='./target' \
    --exclude='./.git' \
    --exclude='./*.tar.gz' \
    -cf - . | tar -C "$SRC_DIR" -xf -

mkdir -p "$RPMBUILD/SOURCES"
tar -C "$STAGING" -czf "$RPMBUILD/SOURCES/$TARBALL" "boulder-relay-$VERSION"

echo "==> Building RPM..."
rpmbuild -ba "$ROOT/packaging/boulder-relay.spec"

echo "==> Done. RPMs are in $RPMBUILD/RPMS/ and $RPMBUILD/SRPMS/"
