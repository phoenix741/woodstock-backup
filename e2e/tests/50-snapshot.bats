#!/usr/bin/env bats
# Snapshot backends — absent from the original test plan.
#
# Expectations differ per platform, and "no snapshot" is a legitimate result:
#   Debian   btrfs, because /home is a btrfs mount point
#   Windows  VSS
#   FreeBSD  none — client-rs/src/storage/snapshots/ only has btrfs.rs and
#            vss.rs, so the agent falls back to the live filesystem. That is
#            the expected behaviour, not a failure.
#
# The "no leftover" and "no error logged" assertions are strongest when
# nothing happened at all, so they only run once a snapshot has actually been
# observed — hence the setup_file probe shared by the Debian tests below.

load test_helper

setup_file() {
    if has_client debian && [[ "$(state_get backup_ok debian 0)" == "1" ]]; then
        if ssh_run debian "journalctl -u woodstock-client --since '-60 min' --no-pager | grep -qi snapshot"; then
            echo "1" > "${BATS_FILE_TMPDIR}/debian-snapshot-seen"
        fi
    fi
}

@test "/home is indeed a btrfs volume (debian)" {
    skip_unless_client debian
    run ssh_run debian "findmnt -n -o FSTYPE /home"
    assert_output --partial "btrfs"
}

@test "the agent created a btrfs snapshot during backup (debian)" {
    skip_unless_client debian
    if [[ "$(state_get backup_ok debian 0)" != "1" ]]; then
        skip "no backup succeeded — nothing to check"
    fi
    [[ -f "${BATS_FILE_TMPDIR}/debian-snapshot-seen" ]]
}

# The agent snapshots into <mount>/.woodstock-snapshot-<timestamp> and deletes
# it when the backup ends; a leftover means cleanup broke.
@test "no .woodstock-snapshot- subvolume is left behind (debian)" {
    skip_unless_client debian
    [[ -f "${BATS_FILE_TMPDIR}/debian-snapshot-seen" ]] || skip "aucun snapshot observé — rien à vérifier"
    run ssh_run debian "test -z \"\$(btrfs subvolume list /home | grep woodstock-snapshot || true)\""
    assert_success
}

@test "no snapshot errors are logged (debian)" {
    skip_unless_client debian
    [[ -f "${BATS_FILE_TMPDIR}/debian-snapshot-seen" ]] || skip "aucun snapshot observé — rien à vérifier"
    run ssh_run debian "! journalctl -u woodstock-client --since '-60 min' --no-pager | grep -qi 'Snapshot failed'"
    assert_success
}

@test "FreeBSD snapshot" {
    skip_unless_client freebsd
    skip "aucun backend implémenté (btrfs.rs et vss.rs uniquement) — repli sur le FS vivant attendu"
}

@test "a VSS snapshot was created during backup (windows)" {
    skip_unless_client windows
    if [[ "$(state_get backup_ok windows 0)" != "1" ]]; then
        skip "no backup succeeded — nothing to check"
    fi
    run ps_script windows <<<'if (Get-EventLog -LogName Application -Source VSS -Newest 20 -ErrorAction SilentlyContinue) { exit 0 } else { exit 1 }'
    assert_success
}

# NTUSER.DAT is held open by the logged-in user, so an agent reading the live
# filesystem cannot open it at all: finding it in the manifest is the
# strongest single proof that VSS actually engaged.
#
# It lives in C:\Users, not in the test-data share — hence the second share
# declared by client_shares, whose manifest has to be dumped separately here.
# 40-backup only ever reads the primary one.
@test "NTUSER.DAT is present in the C:\\Users manifest (windows)" {
    skip_unless_client windows
    if [[ "$(state_get backup_ok windows 0)" != "1" ]]; then
        skip "no backup succeeded — nothing to check"
    fi
    local win_host users_dump
    win_host="$(client_hostname windows)"
    users_dump="${RUN_DIR}/manifest-${win_host}-users.txt"
    ssh_run server "$(ws_cli) ws_console read-protobuf '$(manifest_path "${win_host}" "$(state_get backup_uuid windows)" 'C:\Users')' file-manifest" \
        > "${users_dump}" 2>&1
    run grep -qi "NTUSER.DAT" "${users_dump}"
    assert_success
}
