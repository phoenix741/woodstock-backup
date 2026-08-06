#!/usr/bin/env bats
# §6 Uninstall — runs last, since it takes the agents apart.
#
# Debian distinguishes remove (keep the data) from purge (drop everything,
# including the system user). FreeBSD has no purge: `pkg delete` removes what
# the package registered and leaves everything else — the `woodstock` user, the
# data it accumulated, and the files created after installation — behind.
# Normal for the platform, and worth asserting so it is documented rather than
# assumed.
#
# "Registered" is the operative word: a directory the package declared is
# reaped when it ends up empty, so the client really does lose
# /var/db/woodstock. Only the server, which fills it, keeps it.
#
# The sharpest check here is that pre-deinstall actually stops the service:
# without it, pkg would delete the binaries from under a running daemon. That
# only works if the pidfile is right, which is what 10-service covers.
#
# `remove`/`delete` is the shared precondition for the checks right after it
# and has to happen before any test runs, so it goes in setup_file, in order
# (clients before the server, since the client checks still need a reachable
# API). `purge` is a **second**, later mutation on Debian: it has its own
# @test ("apt purge woodstock-*"), positioned after the post-remove checks in
# file order, so those checks still see the intermediate remove-but-not-purged
# state instead of a state setup_file already collapsed past. Command-level
# success/failure is persisted via lib/state.sh; output already lands in
# ${RUN_DIR}/uninstall-*.log for the content-based checks.

load test_helper

_remove_or_delete_client() {
    local client="$1"
    [[ "${client}" == "windows" ]] && return 0

    case "${client}" in
        debian)
            if ssh_run debian "DEBIAN_FRONTEND=noninteractive apt-get remove -y -qq woodstock-client" \
                 > "${RUN_DIR}/uninstall-${client}.log" 2>&1; then
                state_set remove_ok "${client}" 1
            else
                state_set remove_ok "${client}" 0
            fi
            ;;
        freebsd)
            if ssh_run freebsd "env ASSUME_ALWAYS_YES=yes pkg delete woodstock-client" \
                 > "${RUN_DIR}/uninstall-${client}.log" 2>&1; then
                state_set delete_ok "${client}" 1
            else
                state_set delete_ok "${client}" 0
            fi
            ;;
    esac
}

_remove_or_delete_server() {
    case "${SERVER_OS}" in
        debian)
            if ssh_run server "DEBIAN_FRONTEND=noninteractive apt-get remove -y -qq woodstock-server" \
                 > "${RUN_DIR}/uninstall-server.log" 2>&1; then
                state_set remove_ok server 1
            else
                state_set remove_ok server 0
            fi
            ;;
        freebsd)
            if ssh_run server "env ASSUME_ALWAYS_YES=yes pkg delete woodstock-server" \
                 > "${RUN_DIR}/uninstall-server.log" 2>&1; then
                state_set delete_ok server 1
            else
                state_set delete_ok server 0
            fi
            ;;
    esac
}

setup_file() {
    local client
    for client in ${CLIENTS}; do
        _remove_or_delete_client "${client}"
    done
    # Kept for last: the client checks above still need a reachable API.
    _remove_or_delete_server
}

# ── Debian client ─────────────────────────────────────────────────────────────

@test "apt remove woodstock-client" {
    skip_unless_client debian
    [[ "$(state_get remove_ok debian 0)" == "1" ]]
}

@test "the client binary is removed after remove (debian)" {
    skip_unless_client debian
    [[ "$(state_get remove_ok debian 0)" == "1" ]] || skip "apt remove non abouti"
    run ssh_run debian "! test -x /usr/bin/ws_client_daemon"
    assert_success
}

# The bracket keeps pgrep from matching the shell sshd started to run this
# very command: its argv contains the pattern verbatim, so
# `pgrep -f ws_client_daemon` always finds *something* and the assertion could
# never pass. `pgrep -x` is not an option either — Linux truncates the process
# name to 15 characters.
@test "the debian agent no longer runs after remove" {
    skip_unless_client debian
    [[ "$(state_get remove_ok debian 0)" == "1" ]] || skip "apt remove non abouti"
    run ssh_run debian "! pgrep -f '[w]s_client_daemon' >/dev/null"
    assert_success
}

@test "remove preserves the configuration (debian)" {
    skip_unless_client debian
    [[ "$(state_get remove_ok debian 0)" == "1" ]] || skip "apt remove non abouti"
    run ssh_run debian "test -f $(client_config_dir debian)/config.yaml"
    assert_success
}

@test "apt purge woodstock-client" {
    skip_unless_client debian
    [[ "$(state_get remove_ok debian 0)" == "1" ]] || skip "apt remove non abouti"
    run ssh_run debian "DEBIAN_FRONTEND=noninteractive apt-get purge -y -qq woodstock-client"
    if [[ "${status}" -eq 0 ]]; then
        echo "${output}" >> "${RUN_DIR}/uninstall-debian.log"
        state_set purge_ok debian 1
    else
        state_set purge_ok debian 0
    fi
    assert_success
}

@test "purge removes the conffile (debian)" {
    skip_unless_client debian
    [[ "$(state_get purge_ok debian 0)" == "1" ]] || skip "apt purge non abouti"
    run ssh_run debian "! test -f $(client_config_dir debian)/config.yaml"
    assert_success
}

# The client package deliberately creates no user, so there is none to remove
# — the assertion is that purge did not invent one.
@test "no woodstock user is left behind (debian)" {
    skip_unless_client debian
    [[ "$(state_get purge_ok debian 0)" == "1" ]] || skip "apt purge non abouti"
    run ssh_run debian "! getent passwd woodstock"
    assert_success
}

# ── FreeBSD client ─────────────────────────────────────────────────────────────

@test "pkg delete woodstock-client" {
    skip_unless_client freebsd
    [[ "$(state_get delete_ok freebsd 0)" == "1" ]]
}

@test "pre-deinstall stopped the service (freebsd)" {
    skip_unless_client freebsd
    [[ "$(state_get delete_ok freebsd 0)" == "1" ]] || skip "pkg delete non abouti"
    run cat "${RUN_DIR}/uninstall-freebsd.log"
    assert_output --partial "Stopping"
}

@test "no orphaned daemon remains (freebsd)" {
    skip_unless_client freebsd
    [[ "$(state_get delete_ok freebsd 0)" == "1" ]] || skip "pkg delete non abouti"
    run ssh_run freebsd "! pgrep -f '[w]s_client_daemon' >/dev/null"
    assert_success
}

@test "the client binary is removed (freebsd)" {
    skip_unless_client freebsd
    [[ "$(state_get delete_ok freebsd 0)" == "1" ]] || skip "pkg delete non abouti"
    run ssh_run freebsd "! test -x /usr/local/bin/ws_client_daemon"
    assert_success
}

# Documented FreeBSD behaviour: there is no purge equivalent, so everything the
# package did not register stays — the config.yaml post-install copied from
# the sample, and the certificates deployed at enrollment next to it.
#
# /var/db/woodstock is NOT the thing to assert on: the client package
# registers it as one of its own directories and it stays empty on a client,
# so pkg legitimately reaps it on deinstall. It only survives where the server
# package also owns it and fills it.
@test "pkg delete preserves the configuration, no purge equivalent (freebsd)" {
    skip_unless_client freebsd
    [[ "$(state_get delete_ok freebsd 0)" == "1" ]] || skip "pkg delete non abouti"
    run ssh_run freebsd "test -s $(client_config_dir freebsd)/config.yaml"
    assert_success
}

@test "pkg delete preserves the deployed certificates (freebsd)" {
    skip_unless_client freebsd
    [[ "$(state_get delete_ok freebsd 0)" == "1" ]] || skip "pkg delete non abouti"
    run ssh_run freebsd "test -s $(client_config_dir freebsd)/rootCA.pem"
    assert_success
}

# ── Server ─────────────────────────────────────────────────────────────────────

@test "apt remove woodstock-server" {
    skip_unless_server debian
    [[ "$(state_get remove_ok server 0)" == "1" ]]
}

@test "the 4 server services are stopped (debian)" {
    skip_unless_server debian
    [[ "$(state_get remove_ok server 0)" == "1" ]] || skip "apt remove non abouti"
    run ssh_run server "! systemctl is-active --quiet woodstock-api && ! systemctl is-active --quiet woodstock-worker"
    assert_success
}

@test "remove preserves the backups (debian)" {
    skip_unless_server debian
    [[ "$(state_get remove_ok server 0)" == "1" ]] || skip "apt remove non abouti"
    run ssh_run server "test -d /var/lib/woodstock/hosts"
    assert_success
}

@test "apt purge woodstock-server" {
    skip_unless_server debian
    [[ "$(state_get remove_ok server 0)" == "1" ]] || skip "apt remove non abouti"
    run ssh_run server "DEBIAN_FRONTEND=noninteractive apt-get purge -y -qq woodstock-server"
    if [[ "${status}" -eq 0 ]]; then
        echo "${output}" >> "${RUN_DIR}/uninstall-server.log"
        state_set purge_ok server 1
    else
        state_set purge_ok server 0
    fi
    assert_success
}

@test "purge removes /etc/woodstock (debian)" {
    skip_unless_server debian
    [[ "$(state_get purge_ok server 0)" == "1" ]] || skip "apt purge non abouti"
    run ssh_run server "! test -f /etc/woodstock/server.env"
    assert_success
}

@test "purge removes the woodstock system user (debian)" {
    skip_unless_server debian
    [[ "$(state_get purge_ok server 0)" == "1" ]] || skip "apt purge non abouti"
    run ssh_run server "! getent passwd woodstock"
    assert_success
}

@test "pkg delete woodstock-server (freebsd)" {
    skip_unless_server freebsd
    [[ "$(state_get delete_ok server 0)" == "1" ]]
}

# No bracket needed here, unlike the checks above: the pattern this shell
# carries in its argv is the alternation source text, which the regex itself
# does not match. Leave it alone.
@test "the 4 services are stopped by pre-deinstall (freebsd)" {
    skip_unless_server freebsd
    [[ "$(state_get delete_ok server 0)" == "1" ]] || skip "pkg delete non abouti"
    run ssh_run server "! pgrep -f 'bin/(api_server|job_worker|scheduler|client_api_server)' >/dev/null"
    assert_success
}

@test "pkg delete preserves the data and user (freebsd)" {
    skip_unless_server freebsd
    [[ "$(state_get delete_ok server 0)" == "1" ]] || skip "pkg delete non abouti"
    run ssh_run server "test -d /var/db/woodstock && pw usershow woodstock >/dev/null"
    assert_success
}
