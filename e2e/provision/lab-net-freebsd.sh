#!/bin/sh
# Guest-side, FreeBSD: bring up the lab NIC with a static address.
#
# Same contract as lab-net.sh, rewritten for FreeBSD: no /sys, no iproute2, and
# /bin/sh is not bash. Interfaces are matched by MAC because names differ
# between platforms, but vm_start assigns 52:54:00:e2:e1:XX to every lab NIC.
#
# Usage: lab-net-freebsd.sh <ip> <prefix-length>

set -eu

IP="${1:?usage: lab-net-freebsd.sh <ip> <prefix>}"
PREFIX="${2:-24}"
LAB_MAC_PREFIX="52:54:00:e2:e1"

iface=""
for candidate in $(ifconfig -l); do
    case "${candidate}" in
        lo*) continue ;;
    esac
    if ifconfig "${candidate}" | grep -q "ether ${LAB_MAC_PREFIX}:"; then
        iface="${candidate}"
        break
    fi
done

if [ -z "${iface}" ]; then
    echo "no interface with MAC ${LAB_MAC_PREFIX}:* found" >&2
    exit 1
fi

# Idempotent: re-running the provisioning must not fail on an existing address.
ifconfig "${iface}" inet "${IP}/${PREFIX}" alias 2>/dev/null || \
    ifconfig "${iface}" inet "${IP}/${PREFIX}"
ifconfig "${iface}" up

echo "lab interface ${iface} = ${IP}/${PREFIX}"
