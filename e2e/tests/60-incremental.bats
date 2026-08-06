#!/usr/bin/env bats
# §7.6 — second backup, deduplication and incremental behaviour.
#
# The data set is built so these assertions can discriminate: twin-a.bin and
# nested/twin-b.bin are byte-identical (so the pool must hold their chunks
# once), and mutable.txt is rewritten between the two runs.
#
# Changing the file and triggering the second backup is the shared precondition
# for everything below, not itself a single user-observable assertion, so it
# runs once per client in setup_file. State (new uuid, http acceptance, pool
# growth) is persisted via lib/state.sh so each @test can assert on it
# independently.

load test_helper

_second_backup() {
    local client="$1" host="$2"
    [[ "$(state_get backup_ok "${client}" 0)" == "1" ]] || return 0

    local pool_before first_uuid data_dir
    pool_before="$(server_pool_size)"
    first_uuid="$(state_get backup_uuid "${client}")"

    data_dir="$(client_data_dir "${client}")"
    if [[ "${client}" == "windows" ]]; then
        ps_script windows <<PS >/dev/null
Set-Content -LiteralPath '${data_dir}\\mutable.txt' -Value 'version 2 — modified between the two backups' -Encoding utf8
& 'C:\\e2e-tools\\gen-checksums.ps1' '${data_dir}'
PS
    else
        ssh_script "${client}" <<EOF >/dev/null
printf 'version 2 — modified between the two backups\n' > ${data_dir}/mutable.txt
cd ${data_dir} && find . -type f ! -name checksums.sha256 ! -name '*.nobackup' -print0 | sort -z | xargs -0 sha256sum > checksums.sha256
EOF
    fi

    # POST /backups is deduplicated for 30s; wait it out so this is a new job.
    sleep 35
    local code
    code="$(curl -fsS -o /dev/null -w '%{http_code}' -X POST "${API}/hosts/${host}/backups" || echo 000)"
    state_set second_backup_http "${client}" "${code}"
    [[ "${code}" == "201" || "${code}" == "202" ]] || return 0

    # Not a count: retention keeps one backup per hourly slot, so the first one
    # is purged as surplus minutes after the second completes.
    if wait_for_new_backup "${host}" "${first_uuid}"; then
        state_set second_backup_ok "${client}" 1
    else
        return 0
    fi

    local uuid
    uuid="$(latest_backup_uuid "${host}")"
    state_set backup_uuid "${client}" "${uuid}"

    local pool_after growth
    pool_after="$(server_pool_size)"
    growth=$(( pool_after - pool_before ))
    state_set pool_growth "${client}" "${growth}"

    local dump2="${RUN_DIR}/manifest-${host}-2.txt"
    ssh_run server "$(ws_cli) ws_console read-protobuf '$(manifest_path "${host}" "${uuid}" "$(client_share "${client}")")' file-manifest" \
        > "${dump2}" 2>&1 || true

    local modified
    modified="$(backups_json "${host}" | jq -r '(.[-1] // {}) | .modifiedFileCount // 0')"
    state_set modified_count "${client}" "${modified}"
}

setup_file() {
    local client
    for client in ${CLIENTS}; do
        _second_backup "${client}" "$(client_hostname "${client}")" || true
    done
}

_skip_no_first_backup() {
    [[ "$(state_get backup_ok "$1" 0)" == "1" ]] || skip "the first backup did not succeed"
}

# ── deduplication inside the first backup ────────────────────────────────────
# Two identical files must reference the very same pool chunks.

_dedup_check() {
    local client="$1" host="$2"
    local twin_chunk
    twin_chunk="$(ssh_run server \
        "$(ws_cli) ws_console read-protobuf '$(manifest_path "${host}" "$(state_get backup_uuid "${client}")" "$(client_share "${client}")")' file-manifest --filter-name twin-a.bin" \
        2>/dev/null | grep -oE '[0-9a-f]{64}' | head -1 || true)"
    [[ -n "${twin_chunk}" ]]

    # search-chunk walks the whole pool, so unlike read-protobuf it really
    # does need BACKUP_PATH to point at the platform's data directory.
    local refs
    refs="$(ssh_run server "$(ws_cli) ws_console search-chunk '${twin_chunk}'" 2>&1 || true)"
    [[ "$(grep -c 'twin-' <<<"${refs}")" -ge 2 ]]
}

@test "the two identical copies share the same chunk debian (dedup)" {
    skip_unless_client debian; _skip_no_first_backup debian
    _dedup_check debian e2e-debian
}
@test "the two identical copies share the same chunk freebsd (dedup)" {
    skip_unless_client freebsd; _skip_no_first_backup freebsd
    _dedup_check freebsd e2e-freebsd
}
@test "the two identical copies share the same chunk windows (dedup)" {
    skip_unless_client windows; _skip_no_first_backup windows
    _dedup_check windows e2e-windows
}

# ── second backup, per client ────────────────────────────────────────────────

@test "second backup of e2e-debian accepted" {
    skip_unless_client debian; _skip_no_first_backup debian
    [[ "$(state_get second_backup_http debian)" =~ ^(201|202)$ ]]
}
@test "the second backup of e2e-debian completes" {
    skip_unless_client debian; _skip_no_first_backup debian
    [[ "$(state_get second_backup_ok debian 0)" == "1" ]]
}
@test "the second backup of e2e-debian adds little to the pool" {
    skip_unless_client debian; _skip_no_first_backup debian
    [[ "$(state_get second_backup_ok debian 0)" == "1" ]] || skip "seconde sauvegarde non aboutie"
    (( $(state_get pool_growth debian 999999999) < 40 * 1024 * 1024 ))
}
@test "the modified file appears in the second backup of e2e-debian" {
    skip_unless_client debian; _skip_no_first_backup debian
    run grep -q "mutable.txt" "${RUN_DIR}/manifest-e2e-debian-2.txt"
    assert_success
}
@test "deduplication statistics are written (e2e-debian)" {
    skip_unless_client debian; _skip_no_first_backup debian
    run ssh_run server "test -s $(server_backup_path)/hosts/e2e-debian/statistics.yml"
    assert_success
}
@test "the backup of e2e-debian is indeed incremental" {
    skip_unless_client debian; _skip_no_first_backup debian
    (( $(state_get modified_count debian 0) > 0 ))
}

@test "second backup of e2e-freebsd accepted" {
    skip_unless_client freebsd; _skip_no_first_backup freebsd
    [[ "$(state_get second_backup_http freebsd)" =~ ^(201|202)$ ]]
}
@test "the second backup of e2e-freebsd completes" {
    skip_unless_client freebsd; _skip_no_first_backup freebsd
    [[ "$(state_get second_backup_ok freebsd 0)" == "1" ]]
}
@test "the second backup of e2e-freebsd adds little to the pool" {
    skip_unless_client freebsd; _skip_no_first_backup freebsd
    [[ "$(state_get second_backup_ok freebsd 0)" == "1" ]] || skip "seconde sauvegarde non aboutie"
    (( $(state_get pool_growth freebsd 999999999) < 40 * 1024 * 1024 ))
}
@test "the modified file appears in the second backup of e2e-freebsd" {
    skip_unless_client freebsd; _skip_no_first_backup freebsd
    run grep -q "mutable.txt" "${RUN_DIR}/manifest-e2e-freebsd-2.txt"
    assert_success
}
@test "deduplication statistics are written (e2e-freebsd)" {
    skip_unless_client freebsd; _skip_no_first_backup freebsd
    run ssh_run server "test -s $(server_backup_path)/hosts/e2e-freebsd/statistics.yml"
    assert_success
}
@test "the backup of e2e-freebsd is indeed incremental" {
    skip_unless_client freebsd; _skip_no_first_backup freebsd
    (( $(state_get modified_count freebsd 0) > 0 ))
}

@test "second backup of e2e-windows accepted" {
    skip_unless_client windows; _skip_no_first_backup windows
    [[ "$(state_get second_backup_http windows)" =~ ^(201|202)$ ]]
}
@test "the second backup of e2e-windows completes" {
    skip_unless_client windows; _skip_no_first_backup windows
    [[ "$(state_get second_backup_ok windows 0)" == "1" ]]
}
@test "the second backup of e2e-windows adds little to the pool" {
    skip_unless_client windows; _skip_no_first_backup windows
    [[ "$(state_get second_backup_ok windows 0)" == "1" ]] || skip "seconde sauvegarde non aboutie"
    (( $(state_get pool_growth windows 999999999) < 40 * 1024 * 1024 ))
}
@test "the modified file appears in the second backup of e2e-windows" {
    skip_unless_client windows; _skip_no_first_backup windows
    run grep -q "mutable.txt" "${RUN_DIR}/manifest-e2e-windows-2.txt"
    assert_success
}
@test "deduplication statistics are written (e2e-windows)" {
    skip_unless_client windows; _skip_no_first_backup windows
    run ssh_run server "test -s $(server_backup_path)/hosts/e2e-windows/statistics.yml"
    assert_success
}
@test "the backup of e2e-windows is indeed incremental" {
    skip_unless_client windows; _skip_no_first_backup windows
    (( $(state_get modified_count windows 0) > 0 ))
}
