#!/usr/bin/env bash
# Guest-side: turn a bare FreeBSD VM into a Woodstock server, the way the
# installation guide tells a user to.
#
# Differences from the Debian server:
#   * paths are /usr/local/etc/woodstock and /var/db/woodstock
#   * services are rc.d, enabled through sysrc rather than systemctl
#   * `pkg install` pulls in valkey through the declared dependency
#
# Expects the packages already uploaded to /tmp/packages/pkg/ and
# /tmp/lab-net-freebsd.sh in place.
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

PKG_DIR="/tmp/packages/pkg"
CONFIG_DIR="/var/db/woodstock/config"
ETC_DIR="/usr/local/etc/woodstock"

echo "=== lab network ==="
sh /tmp/lab-net-freebsd.sh "${LAB_IP}" 24

echo "=== install the server ==="
# The catalogue has to exist before anything can be resolved against it: the
# cloud image ships pkg with no repository fetched yet.
env ASSUME_ALWAYS_YES=yes pkg update

# curl is not in FreeBSD base — only fetch(1) is — and both this script and the
# test suite drive the management API with it.
command -v curl >/dev/null 2>&1 || env ASSUME_ALWAYS_YES=yes pkg install -y curl

# `pkg install` on a local file, not `pkg add`: the package depends on
# databases/valkey, and only `pkg install` pulls a missing dependency from the
# remote repository — `pkg add` stops at "Missing dependency 'valkey'"
# (observed, 2026-08-03). Resolving that dependency on its own is exactly what
# 00-install asserts, so valkey must not be installed by hand beforehand.
#
# No ABI override either: whether pkg accepts the package at all is the other
# half of that assertion.
env ASSUME_ALWAYS_YES=yes pkg install -y "${PKG_DIR}"/woodstock-server-*.pkg

echo "=== start valkey ==="
sysrc valkey_enable=YES >/dev/null
service valkey start >/dev/null 2>&1 || service valkey restart

echo "=== server configuration ==="
# post-install.sh only ships server.env.sample; the real file is the admin's job.
if [[ ! -f "${ETC_DIR}/server.env" ]]; then
    cp "${ETC_DIR}/server.env.sample" "${ETC_DIR}/server.env"
    chown root:woodstock "${ETC_DIR}/server.env"
    chmod 640 "${ETC_DIR}/server.env"
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

echo "=== enable and start the stack ==="
for svc in woodstock_worker woodstock_api woodstock_client_api woodstock_scheduler; do
    sysrc "${svc}_enable=YES" >/dev/null
done
# Start the worker first: the other three declare REQUIRE on it.
for svc in woodstock_worker woodstock_api woodstock_client_api woodstock_scheduler; do
    service "${svc}" start >/dev/null 2>&1 || service "${svc}" restart >/dev/null 2>&1 || true
done

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
