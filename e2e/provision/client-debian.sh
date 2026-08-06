#!/usr/bin/env bash
# Guest-side: turn a bare Debian VM into a Woodstock client with a btrfs /home.
#
# The btrfs volume is deliberately mounted on /home rather than on some
# /srv/data: the agent snapshots the *mount point* of the share
# (client-rs/src/storage/snapshots/btrfs.rs), so the share has to sit on its own
# btrfs volume — and "back up the home directories" is the scenario under test.
#
# Enrollment is NOT done here: fetching the agent bundle from the server is a
# user-facing step that tests/30-certs.sh performs and asserts on.
#
# Environment:
#   LAB_IP    static address on the lab network
#   HOSTNAME_ name to report to the server (must match hosts.yml)

set -euo pipefail

: "${LAB_IP:?LAB_IP is required}"
: "${HOSTNAME_:?HOSTNAME_ is required}"

DEB_DIR="/tmp/packages/deb"
DATA_DEV="/dev/vdb"

echo "=== lab network ==="
bash /tmp/lab-net.sh "${LAB_IP}" 24

echo "=== hostname ==="
hostnamectl set-hostname "${HOSTNAME_}"

echo "=== btrfs /home ==="
if ! findmnt -n -o FSTYPE /home 2>/dev/null | grep -q btrfs; then
    [[ -b "${DATA_DEV}" ]] || { echo "no data disk at ${DATA_DEV}" >&2; exit 1; }
    mkfs.btrfs -f -L woodstock-home "${DATA_DEV}" >/dev/null
    mkdir -p /home
    mount "${DATA_DEV}" /home
    uuid="$(blkid -s UUID -o value "${DATA_DEV}")"
    grep -q "${uuid}" /etc/fstab || echo "UUID=${uuid} /home btrfs defaults 0 0" >> /etc/fstab
fi
findmnt -n -o SOURCE,TARGET,FSTYPE /home

echo "=== install the agent ==="
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq "${DEB_DIR}"/woodstock-client_*.deb

echo "=== generate the data to back up ==="
bash /tmp/gen-testdata.sh /home/e2e

echo "=== ready ==="
# The daemon cannot run yet: it needs rootCA.pem and its own certificates,
# which only exist once the agent bundle has been downloaded from the server.
systemctl is-enabled woodstock-client || true
