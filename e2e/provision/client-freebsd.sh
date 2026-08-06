#!/usr/bin/env bash
# Guest-side: turn a bare FreeBSD VM into a Woodstock client.
#
# Differences from the Debian client that matter here:
#   * no snapshot backend exists (client-rs/src/storage/snapshots/ only has
#     btrfs.rs and vss.rs), so the agent reads the live filesystem — expected,
#     not a failure. The data therefore lives on plain UFS, no extra disk.
#   * paths are /usr/local/etc/woodstock and /var/db/woodstock
#   * the service is rc.d, and ws_client_daemon is wrapped in daemon(8)
#
# Environment:
#   LAB_IP     static address on the lab network
#   HOSTNAME_  name to report to the server (must match hosts.yml)

set -euo pipefail

: "${LAB_IP:?LAB_IP is required}"
: "${HOSTNAME_:?HOSTNAME_ is required}"

PKG_DIR="/tmp/packages/pkg"

echo "=== lab network ==="
sh /tmp/lab-net-freebsd.sh "${LAB_IP}" 24

echo "=== hostname ==="
hostname "${HOSTNAME_}"
sysrc hostname="${HOSTNAME_}" >/dev/null

echo "=== install the agent ==="
# `pkg add` with no ABI override: the packages must declare the ABI of the
# system they are installed on, which is exactly what test 00-install asserts.
env ASSUME_ALWAYS_YES=yes pkg add "${PKG_DIR}"/woodstock-client-*.pkg

echo "=== generate the data to back up ==="
mkdir -p /home
sh /tmp/gen-testdata.sh /home/e2e

echo "=== enable the service ==="
# Enabled but not started: the agent needs its certificates first, which only
# exist once the bundle has been downloaded from the server (tests/30-certs.sh).
sysrc woodstock_client_enable=YES >/dev/null

echo "=== ready ==="
pkg info woodstock-client | head -3
