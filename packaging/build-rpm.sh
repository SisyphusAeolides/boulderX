#!/usr/bin/env bash
# Build an RPM for boulderX on Fedora.
set -euo pipefail

VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*= *"//;s/"//')
ARCHIVE="boulderX-${VERSION}.tar.gz"
SPECFILE="packaging/boulderX.spec"

echo "==> Building boulderX v${VERSION} RPM"
git archive --prefix="boulderX-${VERSION}/" HEAD | gzip > "/tmp/${ARCHIVE}"
mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
cp "/tmp/${ARCHIVE}" ~/rpmbuild/SOURCES/
cp "${SPECFILE}" ~/rpmbuild/SPECS/boulderX.spec
echo "==> Installing build dependencies..."
sudo dnf builddep -y ~/rpmbuild/SPECS/boulderX.spec || true
echo "==> Running rpmbuild..."
rpmbuild -ba ~/rpmbuild/SPECS/boulderX.spec
echo ""
echo "==> RPM built:"
find ~/rpmbuild/RPMS -name "boulderX-*.rpm" -print
