#!/usr/bin/env bats
# §7.1-7.4 — run a full backup of every client and inspect the manifest.
#
# Two API quirks the suite has to live with:
#   * POST /backups is deduplicated in Redis for 30s and answers 202 with no
#     job when it fires twice, so 202 is tolerated.
#   * The backup list carries no boolean "done": completion is
#     status.statusType == "completed" (see lib/woodstock.sh).
#
# backup_ok/backup_uuid (lib/state.sh) are persisted per client so the later
# suites can tell "the assertion failed" from "there was never a backup to
# assert on" — including when they run in a separate process.
#
# Triggering each client's backup and waiting for it is not itself something
# to assert on in isolation (nothing to check yet at that point), so it runs
# once in setup_file; the manifest and its contents are what the tests below
# check.

load test_helper

_run_backup() {
    local client="$1" host uuid file_count
    host="$(client_hostname "${client}")"
    state_set backup_ok "${client}" 0

    local code
    code="$(curl -fsS -o /dev/null -w '%{http_code}' -X POST "${API}/hosts/${host}/backups" || echo 000)"
    [[ "${code}" == "201" || "${code}" == "202" ]] || return 1

    wait_for_backup "${host}" 1 || return 1

    uuid="$(latest_backup_uuid "${host}")"
    [[ -n "${uuid}" ]] || return 1
    state_set backup_uuid "${client}" "${uuid}"

    file_count="$(latest_backup_file_count "${host}")"
    (( file_count > 0 )) || return 1

    local dump="${RUN_DIR}/manifest-${host}.txt"
    ssh_run server "$(ws_cli) ws_console read-protobuf '$(manifest_path "${host}" "${uuid}" "$(client_share "${client}")")' file-manifest" \
        > "${dump}" 2>&1 || return 1

    state_set backup_ok "${client}" 1
}

setup_file() {
    local client
    for client in ${CLIENTS}; do
        _run_backup "${client}" || true
    done
}

# ─── one @test per client for the setup-level assertions ────────────────────
# (HTTP accepted, backup completes, uuid/file-count/manifest readable) — the
# manifest *contents* get their own tests further down, shared across clients
# since the assertions and the fixture file names are identical.

@test "e2e-debian backup accepted and completed" {
    skip_unless_client debian
    [[ "$(state_get backup_ok debian 0)" == "1" ]]
}

@test "e2e-freebsd backup accepted and completed" {
    skip_unless_client freebsd
    [[ "$(state_get backup_ok freebsd 0)" == "1" ]]
}

@test "e2e-windows backup accepted and completed" {
    skip_unless_client windows
    [[ "$(state_get backup_ok windows 0)" == "1" ]]
}

# ─── manifest contents, one @test per client × expectation ──────────────────

_manifest_checks() {
    local client="$1" host="$2"
    if [[ "$(state_get backup_ok "${client}" 0)" != "1" ]]; then
        skip "no backup succeeded for ${host}"
    fi
}

@test "the e2e-debian manifest contains big-1.bin" {
    skip_unless_client debian; _manifest_checks debian e2e-debian
    run grep -q "big-1.bin" "${RUN_DIR}/manifest-e2e-debian.txt"
    assert_success
}
@test "the e2e-debian manifest contains twin-a.bin" {
    skip_unless_client debian; _manifest_checks debian e2e-debian
    run grep -q "twin-a.bin" "${RUN_DIR}/manifest-e2e-debian.txt"
    assert_success
}
@test "the e2e-debian manifest contains twin-b.bin" {
    skip_unless_client debian; _manifest_checks debian e2e-debian
    run grep -q "twin-b.bin" "${RUN_DIR}/manifest-e2e-debian.txt"
    assert_success
}
@test "the e2e-debian manifest contains mutable.txt" {
    skip_unless_client debian; _manifest_checks debian e2e-debian
    run grep -q "mutable.txt" "${RUN_DIR}/manifest-e2e-debian.txt"
    assert_success
}
@test "the e2e-debian manifest contains xattr.txt" {
    skip_unless_client debian; _manifest_checks debian e2e-debian
    run grep -q "xattr.txt" "${RUN_DIR}/manifest-e2e-debian.txt"
    assert_success
}
@test "the e2e-debian manifest contains acl.txt" {
    skip_unless_client debian; _manifest_checks debian e2e-debian
    run grep -q "acl.txt" "${RUN_DIR}/manifest-e2e-debian.txt"
    assert_success
}
@test "the e2e-debian manifest contains sparse.bin" {
    skip_unless_client debian; _manifest_checks debian e2e-debian
    run grep -q "sparse.bin" "${RUN_DIR}/manifest-e2e-debian.txt"
    assert_success
}
@test "the e2e-debian manifest contains leaf.txt" {
    skip_unless_client debian; _manifest_checks debian e2e-debian
    run grep -q "leaf.txt" "${RUN_DIR}/manifest-e2e-debian.txt"
    assert_success
}
@test "non-ASCII names are preserved (debian)" {
    skip_unless_client debian; _manifest_checks debian e2e-debian
    run cat "${RUN_DIR}/manifest-e2e-debian.txt"
    assert_output --partial "éàü"
}
@test "the symbolic link is recorded (debian)" {
    skip_unless_client debian; _manifest_checks debian e2e-debian
    run grep -q "link-to-big-1" "${RUN_DIR}/manifest-e2e-debian.txt"
    assert_success
}
@test "the excluded file is not backed up (debian)" {
    skip_unless_client debian; _manifest_checks debian e2e-debian
    grep -q "big-1.bin" "${RUN_DIR}/manifest-e2e-debian.txt"
    run bash -c "! grep -q 'skip.nobackup' '${RUN_DIR}/manifest-e2e-debian.txt'"
    assert_success
}
# ws_console renders xattr keys and values as decimal byte arrays, so the name
# has to be decoded before it can be matched.
@test "the extended attribute user.woodstock is captured (debian)" {
    skip_unless_client debian; _manifest_checks debian e2e-debian
    run manifest_xattr_keys "${RUN_DIR}/manifest-e2e-debian.txt" "e2e/xattr.txt"
    assert_output --partial "user.woodstock"
}
# POSIX ACLs travel in the same xattr list, under system.posix_acl_access.
@test "the POSIX ACL is captured (debian)" {
    skip_unless_client debian; _manifest_checks debian e2e-debian
    run manifest_xattr_keys "${RUN_DIR}/manifest-e2e-debian.txt" "e2e/acl.txt"
    assert_output --partial "posix_acl"
}

@test "the e2e-freebsd manifest contains big-1.bin" {
    skip_unless_client freebsd; _manifest_checks freebsd e2e-freebsd
    run grep -q "big-1.bin" "${RUN_DIR}/manifest-e2e-freebsd.txt"
    assert_success
}
@test "the e2e-freebsd manifest contains twin-a.bin" {
    skip_unless_client freebsd; _manifest_checks freebsd e2e-freebsd
    run grep -q "twin-a.bin" "${RUN_DIR}/manifest-e2e-freebsd.txt"
    assert_success
}
@test "the e2e-freebsd manifest contains twin-b.bin" {
    skip_unless_client freebsd; _manifest_checks freebsd e2e-freebsd
    run grep -q "twin-b.bin" "${RUN_DIR}/manifest-e2e-freebsd.txt"
    assert_success
}
@test "the e2e-freebsd manifest contains mutable.txt" {
    skip_unless_client freebsd; _manifest_checks freebsd e2e-freebsd
    run grep -q "mutable.txt" "${RUN_DIR}/manifest-e2e-freebsd.txt"
    assert_success
}
@test "the e2e-freebsd manifest contains xattr.txt" {
    skip_unless_client freebsd; _manifest_checks freebsd e2e-freebsd
    run grep -q "xattr.txt" "${RUN_DIR}/manifest-e2e-freebsd.txt"
    assert_success
}
@test "the e2e-freebsd manifest contains acl.txt" {
    skip_unless_client freebsd; _manifest_checks freebsd e2e-freebsd
    run grep -q "acl.txt" "${RUN_DIR}/manifest-e2e-freebsd.txt"
    assert_success
}
@test "the e2e-freebsd manifest contains sparse.bin" {
    skip_unless_client freebsd; _manifest_checks freebsd e2e-freebsd
    run grep -q "sparse.bin" "${RUN_DIR}/manifest-e2e-freebsd.txt"
    assert_success
}
@test "the e2e-freebsd manifest contains leaf.txt" {
    skip_unless_client freebsd; _manifest_checks freebsd e2e-freebsd
    run grep -q "leaf.txt" "${RUN_DIR}/manifest-e2e-freebsd.txt"
    assert_success
}
@test "non-ASCII names are preserved (freebsd)" {
    skip_unless_client freebsd; _manifest_checks freebsd e2e-freebsd
    run cat "${RUN_DIR}/manifest-e2e-freebsd.txt"
    assert_output --partial "éàü"
}
@test "the symbolic link is recorded (freebsd)" {
    skip_unless_client freebsd; _manifest_checks freebsd e2e-freebsd
    run grep -q "link-to-big-1" "${RUN_DIR}/manifest-e2e-freebsd.txt"
    assert_success
}
@test "the excluded file is not backed up (freebsd)" {
    skip_unless_client freebsd; _manifest_checks freebsd e2e-freebsd
    grep -q "big-1.bin" "${RUN_DIR}/manifest-e2e-freebsd.txt"
    run bash -c "! grep -q 'skip.nobackup' '${RUN_DIR}/manifest-e2e-freebsd.txt'"
    assert_success
}

@test "the e2e-windows manifest contains big-1.bin" {
    skip_unless_client windows; _manifest_checks windows e2e-windows
    run grep -q "big-1.bin" "${RUN_DIR}/manifest-e2e-windows.txt"
    assert_success
}
@test "the e2e-windows manifest contains twin-a.bin" {
    skip_unless_client windows; _manifest_checks windows e2e-windows
    run grep -q "twin-a.bin" "${RUN_DIR}/manifest-e2e-windows.txt"
    assert_success
}
@test "the e2e-windows manifest contains twin-b.bin" {
    skip_unless_client windows; _manifest_checks windows e2e-windows
    run grep -q "twin-b.bin" "${RUN_DIR}/manifest-e2e-windows.txt"
    assert_success
}
@test "the e2e-windows manifest contains mutable.txt" {
    skip_unless_client windows; _manifest_checks windows e2e-windows
    run grep -q "mutable.txt" "${RUN_DIR}/manifest-e2e-windows.txt"
    assert_success
}
@test "the e2e-windows manifest contains xattr.txt" {
    skip_unless_client windows; _manifest_checks windows e2e-windows
    run grep -q "xattr.txt" "${RUN_DIR}/manifest-e2e-windows.txt"
    assert_success
}
@test "the e2e-windows manifest contains acl.txt" {
    skip_unless_client windows; _manifest_checks windows e2e-windows
    run grep -q "acl.txt" "${RUN_DIR}/manifest-e2e-windows.txt"
    assert_success
}
@test "the e2e-windows manifest contains sparse.bin" {
    skip_unless_client windows; _manifest_checks windows e2e-windows
    run grep -q "sparse.bin" "${RUN_DIR}/manifest-e2e-windows.txt"
    assert_success
}
@test "the e2e-windows manifest contains leaf.txt" {
    skip_unless_client windows; _manifest_checks windows e2e-windows
    run grep -q "leaf.txt" "${RUN_DIR}/manifest-e2e-windows.txt"
    assert_success
}
@test "non-ASCII names are preserved (windows)" {
    skip_unless_client windows; _manifest_checks windows e2e-windows
    run cat "${RUN_DIR}/manifest-e2e-windows.txt"
    assert_output --partial "éàü"
}
@test "the symbolic link is recorded (windows)" {
    skip_unless_client windows; _manifest_checks windows e2e-windows
    run grep -q "link-to-big-1" "${RUN_DIR}/manifest-e2e-windows.txt"
    assert_success
}
@test "the excluded file is not backed up (windows)" {
    skip_unless_client windows; _manifest_checks windows e2e-windows
    grep -q "big-1.bin" "${RUN_DIR}/manifest-e2e-windows.txt"
    run bash -c "! grep -q 'skip.nobackup' '${RUN_DIR}/manifest-e2e-windows.txt'"
    assert_success
}
