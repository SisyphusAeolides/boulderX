#!/usr/bin/env bash
# Build an RPM for Boulder Relay on Fedora.
set -euo pipefail

VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*= *"//;s/"//')
ARCHIVE="boulder-relay-${VERSION}.tar.gz"
SPECFILE="packaging/boulder-relay.spec"

echo "==> Building boulder-relay v${VERSION} RPM"
git archive --prefix="boulder-relay-${VERSION}/" HEAD | gzip > "/tmp/${ARCHIVE}"
mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
cp "/tmp/${ARCHIVE}" ~/rpmbuild/SOURCES/
cp "${SPECFILE}" ~/rpmbuild/SPECS/boulder-relay.spec
echo "==> Installing build dependencies..."
sudo dnf builddep -y ~/rpmbuild/SPECS/boulder-relay.spec || true
echo "==> Running rpmbuild..."
rpmbuild -ba ~/rpmbuild/SPECS/boulder-relay.spec
echo ""
echo "==> RPM built:"
find ~/rpmbuild/RPMS -name "boulder-relay-*.rpm" -print
