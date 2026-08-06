#!/usr/bin/env bash
# Guest-side: bring up the lab NIC with a static address.
#
# Interfaces are matched by MAC rather than by name: predictable names differ
# between Debian and FreeBSD and even between QEMU machine types, but vm_start
# assigns 52:54:00:e2:e1:XX to the lab NIC of every VM.
#
# Usage: lab-net.sh <ip> <prefix-length>

set -euo pipefail

IP="${1:?usage: lab-net.sh <ip> <prefix>}"
PREFIX="${2:-24}"
LAB_MAC_PREFIX="52:54:00:e2:e1"

iface=""
for candidate in /sys/class/net/*; do
    name="$(basename "${candidate}")"
    [[ "${name}" == "lo" ]] && continue
    mac="$(cat "${candidate}/address" 2>/dev/null || true)"
    if [[ "${mac}" == "${LAB_MAC_PREFIX}:"* ]]; then
        iface="${name}"
        break
    fi
done

[[ -n "${iface}" ]] || { echo "no interface with MAC ${LAB_MAC_PREFIX}:* found" >&2; exit 1; }

ip link set "${iface}" up
# Idempotent: re-running the provisioning must not fail on an existing address.
ip addr replace "${IP}/${PREFIX}" dev "${iface}"

echo "lab interface ${iface} = ${IP}/${PREFIX}"
