#!/usr/bin/env bats
# PKI and agent enrollment — the part packaging-test-plan.md used to gloss over
# as "certificates generated/deployed".
#
# The real sequence is lazy and split across three triggers:
#   api_server boot         → rootCA.pem + the JWT RSA key pair
#   client_api_server boot  → https.pem (CN=localhost)
#   GET /api/hosts/{n}/client → the four per-host key pairs, and the agent bundle
#
# That last call IS the enrollment mechanism: there is no password-based
# protocol and no CLI to generate certificates. A backup started before it has
# been made fails at gRPC connect.
#
# Enrollment itself (downloading the bundle and deploying it on the client) has
# to happen once per client before any of the per-client assertions below can
# be checked, so it runs in setup_file rather than as its own test — there is
# nothing user-observable about "the bundle got copied" on its own; what
# matters is everything that follows from it.

load test_helper

_enroll() {
    local client="$1" host bundle
    host="$(client_hostname "${client}")"
    bundle="${RUN_DIR}/bundle-${host}.zip"

    curl -fsS -H "Host: ${IP_SERVER}:3000" \
        "${API}/hosts/${host}/client?client=none" -o "${bundle}" || return 1

    local cfg_dir
    cfg_dir="$(client_config_dir "${client}")"
    if [[ "${client}" == "windows" ]]; then
        scp_to windows "${bundle}" 'C:/agent-bundle.zip'
        ps_script windows <<PS >/dev/null
Expand-Archive -Path 'C:\agent-bundle.zip' -DestinationPath '${cfg_dir}' -Force
$(client_restart_cmd "${client}")
PS
    else
        scp_to "${client}" "${bundle}" /tmp/agent-bundle.zip
        local acl_line="acl: true"
        [[ "${client}" == "freebsd" ]] && acl_line="# acl unsupported on this filesystem"
        ssh_script "${client}" <<EOF >/dev/null
unzip -qo /tmp/agent-bundle.zip -d ${cfg_dir}/
chmod 600 ${cfg_dir}/*.key ${cfg_dir}/config.yaml
printf 'xattr: true\n${acl_line}\n' >> ${cfg_dir}/config.yaml
$(client_restart_cmd "${client}")
EOF
    fi
}

setup_file() {
    local client
    for client in ${CLIENTS}; do
        _enroll "${client}" || true
    done
}

# ─── Server-side PKI ─────────────────────────────────────────────────────────

@test "rootCA.pem is generated on api_server startup" {
    run ssh_run server "test -s $(server_backup_path)/certs/rootCA.pem"
    assert_success
}

# Regression: generate_rsa_key() used to be dead code, so these never existed
# and no backup could ever authenticate. No manual openssl step must be needed.
@test "the JWT key pair is generated automatically (no manual openssl)" {
    run ssh_run server "test -s $(server_backup_path)/certs/private_key.pem && test -s $(server_backup_path)/certs/public_key.pem"
    assert_success
}

@test "private_key.pem is readable only by its owner" {
    run stat_owner_mode server "$(server_backup_path)/certs/private_key.pem"
    assert_output --partial "600"
}

@test "https.pem is generated on client_api_server startup" {
    run ssh_run server "test -s $(server_backup_path)/certs/https.pem"
    assert_success
}

# ─── Enrollment: Debian ──────────────────────────────────────────────────────

@test "GET /api/hosts/e2e-debian/client returns a bundle (debian)" {
    skip_unless_client debian
    [[ -s "${RUN_DIR}/bundle-e2e-debian.zip" ]]
}

@test "the e2e-debian_ca certificate is created server-side" {
    skip_unless_client debian
    run ssh_run server "test -s $(server_backup_path)/certs/e2e-debian_ca.pem"
    assert_success
}

@test "the e2e-debian_client certificate is created server-side" {
    skip_unless_client debian
    run ssh_run server "test -s $(server_backup_path)/certs/e2e-debian_client.pem"
    assert_success
}

@test "the e2e-debian_server certificate is created server-side" {
    skip_unless_client debian
    run ssh_run server "test -s $(server_backup_path)/certs/e2e-debian_server.pem"
    assert_success
}

@test "the e2e-debian_https certificate is created server-side" {
    skip_unless_client debian
    run ssh_run server "test -s $(server_backup_path)/certs/e2e-debian_https.pem"
    assert_success
}

@test "the e2e-debian bundle contains certificates and config.yaml" {
    skip_unless_client debian
    local bundle="${RUN_DIR}/bundle-e2e-debian.zip"
    for entry in rootCA.pem public_key.pem e2e-debian_server.pem e2e-debian_server.key \
                 e2e-debian_https.pem e2e-debian_https.key config.yaml; do
        run unzip -l "${bundle}" -- "${entry}"
        assert_success
    done
}

@test "the generated config.yaml for e2e-debian points to the server, not localhost" {
    skip_unless_client debian
    run bash -c "unzip -p '${RUN_DIR}/bundle-e2e-debian.zip' config.yaml | grep '^server:'"
    assert_output --partial "${IP_SERVER}"
}

@test "the generated config.yaml for e2e-debian does not enable xattr or acl by default" {
    skip_unless_client debian
    run bash -c "! unzip -p '${RUN_DIR}/bundle-e2e-debian.zip' config.yaml | grep -qE '^(xattr|acl):'"
    assert_success
}

@test "the debian agent starts once enrolled" {
    skip_unless_client debian
    run retry 60 3 ssh_run debian "$(client_status_cmd debian)"
    assert_not_infra_failure
    assert_success
}

@test "the debian agent listens on port 3657" {
    skip_unless_client debian
    run retry 60 3 ssh_run debian "ss -lntp | grep -q ':3657'"
    assert_not_infra_failure
    assert_success
}

@test "the e2e-debian agent registers with the mTLS gateway" {
    skip_unless_client debian
    _registered() { server_log woodstock_client_api | grep -q "Client registration request from e2e-debian"; }
    run retry 180 10 _registered
    assert_not_infra_failure
    assert_success
}

@test "the address of e2e-debian is published in the resolver cache" {
    skip_unless_client debian
    run retry 180 10 ssh_run server \
        "(redis-cli HEXISTS woodstock_dns e2e-debian 2>/dev/null || valkey-cli HEXISTS woodstock_dns e2e-debian 2>/dev/null) | grep -qx 1"
    assert_not_infra_failure
    assert_success
}

# ─── Enrollment: FreeBSD ─────────────────────────────────────────────────────

@test "GET /api/hosts/e2e-freebsd/client returns a bundle (freebsd)" {
    skip_unless_client freebsd
    [[ -s "${RUN_DIR}/bundle-e2e-freebsd.zip" ]]
}

@test "the e2e-freebsd_ca certificate is created server-side" {
    skip_unless_client freebsd
    run ssh_run server "test -s $(server_backup_path)/certs/e2e-freebsd_ca.pem"
    assert_success
}

@test "the e2e-freebsd_client certificate is created server-side" {
    skip_unless_client freebsd
    run ssh_run server "test -s $(server_backup_path)/certs/e2e-freebsd_client.pem"
    assert_success
}

@test "the e2e-freebsd_server certificate is created server-side" {
    skip_unless_client freebsd
    run ssh_run server "test -s $(server_backup_path)/certs/e2e-freebsd_server.pem"
    assert_success
}

@test "the e2e-freebsd_https certificate is created server-side" {
    skip_unless_client freebsd
    run ssh_run server "test -s $(server_backup_path)/certs/e2e-freebsd_https.pem"
    assert_success
}

@test "the e2e-freebsd bundle contains certificates and config.yaml" {
    skip_unless_client freebsd
    local bundle="${RUN_DIR}/bundle-e2e-freebsd.zip"
    for entry in rootCA.pem public_key.pem e2e-freebsd_server.pem e2e-freebsd_server.key \
                 e2e-freebsd_https.pem e2e-freebsd_https.key config.yaml; do
        run unzip -l "${bundle}" -- "${entry}"
        assert_success
    done
}

@test "the generated config.yaml for e2e-freebsd points to the server, not localhost" {
    skip_unless_client freebsd
    run bash -c "unzip -p '${RUN_DIR}/bundle-e2e-freebsd.zip' config.yaml | grep '^server:'"
    assert_output --partial "${IP_SERVER}"
}

@test "the generated config.yaml for e2e-freebsd does not enable xattr or acl by default" {
    skip_unless_client freebsd
    run bash -c "! unzip -p '${RUN_DIR}/bundle-e2e-freebsd.zip' config.yaml | grep -qE '^(xattr|acl):'"
    assert_success
}

@test "the freebsd agent starts once enrolled (pid findable)" {
    skip_unless_client freebsd
    run retry 60 3 ssh_run freebsd "service woodstock_client status"
    assert_not_infra_failure
    assert_success
}

@test "the freebsd agent listens on port 3657" {
    skip_unless_client freebsd
    run retry 60 3 ssh_run freebsd "sockstat -4l | grep -q ':3657'"
    assert_not_infra_failure
    assert_success
}

@test "the e2e-freebsd agent registers with the mTLS gateway" {
    skip_unless_client freebsd
    _registered() { server_log woodstock_client_api | grep -q "Client registration request from e2e-freebsd"; }
    run retry 180 10 _registered
    assert_not_infra_failure
    assert_success
}

@test "the address of e2e-freebsd is published in the resolver cache" {
    skip_unless_client freebsd
    run retry 180 10 ssh_run server \
        "(redis-cli HEXISTS woodstock_dns e2e-freebsd 2>/dev/null || valkey-cli HEXISTS woodstock_dns e2e-freebsd 2>/dev/null) | grep -qx 1"
    assert_not_infra_failure
    assert_success
}

# ─── Enrollment: Windows ─────────────────────────────────────────────────────

@test "GET /api/hosts/e2e-windows/client returns a bundle (windows)" {
    skip_unless_client windows
    [[ -s "${RUN_DIR}/bundle-e2e-windows.zip" ]]
}

@test "the e2e-windows_ca certificate is created server-side" {
    skip_unless_client windows
    run ssh_run server "test -s $(server_backup_path)/certs/e2e-windows_ca.pem"
    assert_success
}

@test "the e2e-windows_client certificate is created server-side" {
    skip_unless_client windows
    run ssh_run server "test -s $(server_backup_path)/certs/e2e-windows_client.pem"
    assert_success
}

@test "the e2e-windows_server certificate is created server-side" {
    skip_unless_client windows
    run ssh_run server "test -s $(server_backup_path)/certs/e2e-windows_server.pem"
    assert_success
}

@test "the e2e-windows_https certificate is created server-side" {
    skip_unless_client windows
    run ssh_run server "test -s $(server_backup_path)/certs/e2e-windows_https.pem"
    assert_success
}

@test "the e2e-windows bundle contains certificates and config.yaml" {
    skip_unless_client windows
    local bundle="${RUN_DIR}/bundle-e2e-windows.zip"
    for entry in rootCA.pem public_key.pem e2e-windows_server.pem e2e-windows_server.key \
                 e2e-windows_https.pem e2e-windows_https.key config.yaml; do
        run unzip -l "${bundle}" -- "${entry}"
        assert_success
    done
}

@test "the generated config.yaml for e2e-windows points to the server, not localhost" {
    skip_unless_client windows
    run bash -c "unzip -p '${RUN_DIR}/bundle-e2e-windows.zip' config.yaml | grep '^server:'"
    assert_output --partial "${IP_SERVER}"
}

@test "the generated config.yaml for e2e-windows does not enable xattr or acl by default" {
    skip_unless_client windows
    run bash -c "! unzip -p '${RUN_DIR}/bundle-e2e-windows.zip' config.yaml | grep -qE '^(xattr|acl):'"
    assert_success
}

@test "the windows agent starts once enrolled" {
    skip_unless_client windows
    run retry 60 3 ssh_run windows "$(client_status_cmd windows)"
    assert_not_infra_failure
    assert_success
}

@test "the windows agent listens on port 3657" {
    skip_unless_client windows
    run retry 60 3 ssh_run windows "if (-not (Get-NetTCPConnection -State Listen -LocalPort 3657 -ErrorAction SilentlyContinue)) { exit 1 }"
    assert_not_infra_failure
    assert_success
}

@test "the e2e-windows agent registers with the mTLS gateway" {
    skip_unless_client windows
    # 300s not 180s (debian/freebsd's own window, line ~152/235 above): the
    # first registration attempt after a fresh boot is measurably slower and
    # more variable on Windows than on either Unix guest — observed failing
    # here on roughly half of all attempts at 180s, both on this image and
    # on the pre-Packer build-windows.sh one, always clearing well inside
    # 300s on retry. Debian/FreeBSD have never shown this at 180s, so their
    # windows stay as-is rather than widening everything to match the
    # slowest platform.
    _registered() { server_log woodstock_client_api | grep -q "Client registration request from e2e-windows"; }
    run retry 300 10 _registered
    assert_not_infra_failure
    assert_success
}

@test "the address of e2e-windows is published in the resolver cache" {
    skip_unless_client windows
    run retry 300 10 ssh_run server \
        "(redis-cli HEXISTS woodstock_dns e2e-windows 2>/dev/null || valkey-cli HEXISTS woodstock_dns e2e-windows 2>/dev/null) | grep -qx 1"
    assert_not_infra_failure
    assert_success
}

# ─── Windows bundle ──────────────────────────────────────────────────────────
# `?client=windows` fetches the agent binary from the Gitea release assets and
# silently ships a bundle without it on failure (hosts.rs catches and skips),
# so check explicitly rather than assume.

@test "the bundle ?client=windows includes ws_client_daemon.exe" {
    skip_unless_client windows
    local win_bundle="${RUN_DIR}/bundle-windows-check.zip"
    run curl -fsS -H "Host: ${IP_SERVER}:3000" \
        "${API}/hosts/$(client_hostname windows)/client?client=windows" -o "${win_bundle}"
    assert_success
    run unzip -l "${win_bundle}" -- ws_client_daemon.exe
    assert_success
}
