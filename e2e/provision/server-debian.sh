#!/usr/bin/env bash
# Guest-side: turn a bare Debian VM into a Woodstock server, the way the
# installation guide tells a user to.
#
# Expects the packages already uploaded to /tmp/packages/deb/ and
# /tmp/lab-net.sh in place.
#
# Environment:
#   LAB_IP        static address on the lab network
#   HOSTS         space-separated "<name>|<share>" pairs to declare in hosts.yml.
#                 The share differs per platform — /home on Unix, C:\e2e on
#                 Windows — so it travels with the name rather than being
#                 assumed here.
#   HOST_PASSWORD shared secret written to every <host>.yml

set -euo pipefail

: "${LAB_IP:?LAB_IP is required}"
: "${HOSTS:?HOSTS is required}"
: "${HOST_PASSWORD:=e2e-shared-secret}"

DEB_DIR="/tmp/packages/deb"
CONFIG_DIR="/var/lib/woodstock/config"

echo "=== lab network ==="
bash /tmp/lab-net.sh "${LAB_IP}" 24

echo "=== install server + admin CLI ==="
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
# Let apt resolve the declared dependencies (valkey-server | redis-server) on
# its own — that resolution is exactly what test 00-install checks.
apt-get install -y -qq "${DEB_DIR}"/woodstock-server_*.deb "${DEB_DIR}"/woodstock-cli_*.deb

echo "=== make the API reachable from the host port forward ==="
# MANAGEMENT_API_LISTEN already defaults to 0.0.0.0, but be explicit: the suite
# drives the API through QEMU's hostfwd, which connects from 10.0.2.2.
if ! grep -q '^MANAGEMENT_API_LISTEN=' /etc/woodstock/server.env; then
    echo 'MANAGEMENT_API_LISTEN=0.0.0.0' >> /etc/woodstock/server.env
fi

echo "=== declare the hosts ==="
install -d -o woodstock -g woodstock -m 750 "${CONFIG_DIR}"

: > "${CONFIG_DIR}/hosts.yml"
for entry in ${HOSTS}; do
    echo "- ${entry%%|*}" >> "${CONFIG_DIR}/hosts.yml"
done

for entry in ${HOSTS}; do
    host="${entry%%|*}"
    shares="${entry#*|}"
    # The first share holds the generated test data; on the Debian client it is
    # also the btrfs mount point, which is what exercises the snapshot path.
    # Windows declares a second one (C:\Users) so NTUSER.DAT proves VSS.
    # Single-quoted in YAML so the Windows backslashes stay literal.
    {
        echo "password: ${HOST_PASSWORD}"
        echo "operations:"
        echo "  operation:"
        echo "    timeout: 3600"
        echo "    shares:"
        printf '%s\n' "${shares}" | tr ',' '\n' | while IFS= read -r share; do
            [ -n "${share}" ] && echo "      - name: '${share}'"
        done
        echo "    excludes:"
        echo '      - "**/*.nobackup"'
        echo "schedule:"
        echo "  activated: false"
    } > "${CONFIG_DIR}/${host}.yml"
done

chown -R woodstock:woodstock "${CONFIG_DIR}"
chmod 640 "${CONFIG_DIR}"/*.yml

echo "=== restart the stack ==="
systemctl restart woodstock-api woodstock-client-api woodstock-worker woodstock-scheduler

# Host configuration is cached in Redis for 24h; without this the API would
# keep serving the empty hosts.yml it read at boot.
for _ in $(seq 1 30); do
    if curl -fsS -X POST http://127.0.0.1:3000/api/server/cache/clear >/dev/null 2>&1; then
        break
    fi
    sleep 2
done

echo "=== ready ==="
curl -fsS http://127.0.0.1:3000/api/hosts
echo
