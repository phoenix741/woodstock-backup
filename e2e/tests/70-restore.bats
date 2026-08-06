#!/usr/bin/env bats
# §7.5 — restore with ws_restore and compare against the source.
#
# ws_restore takes four positional arguments and the address *must* carry the
# port (cli-rs/src/wsrestore.rs parses it as a SocketAddr). The backup id can
# be a UUID or the sequential number (cli-rs/src/backup_resolver.rs); the UUID
# is used here since it is what names the backup directory on disk.
#
# It runs on the server and pushes the files back to the agent, so the
# destination directory lives on the client. Running ws_restore and locating
# the restored tree is the shared precondition for every assertion below, so
# it happens once per client in setup_file; state (restored path) persists via
# lib/state.sh.

load test_helper

_restore() {
    local client="$1" host="$2"
    [[ "$(state_get backup_ok "${client}" 0)" == "1" ]] || return 0

    local uuid ip dest
    uuid="$(state_get backup_uuid "${client}")"
    ip="$(vm_ip "${client}")"

    if [[ "${client}" == "windows" ]]; then
        dest='C:\restore'
        ssh_run windows "Remove-Item -Recurse -Force '${dest}' -ErrorAction SilentlyContinue; New-Item -ItemType Directory -Force -Path '${dest}' | Out-Null" >/dev/null
    else
        dest="/tmp/restore-${host}"
        ssh_run "${client}" "rm -rf ${dest} && mkdir -p ${dest}" >/dev/null
    fi

    ssh_run server "$(ws_cli) ws_restore '${host}' '${ip}:3657' '${uuid}' '$(client_share "${client}")' --destination-directory '${dest}'" \
        > "${RUN_DIR}/restore-${host}.log" 2>&1 || return 0
    state_set restore_ok "${client}" 1

    local restored
    if [[ "${client}" == "windows" ]]; then
        restored="$(ssh_run windows \
            "(Get-ChildItem -Path '${dest}' -Recurse -Filter checksums.sha256 -File | Select-Object -First 1).DirectoryName" 2>/dev/null | tr -d '\r' || true)"
    else
        restored="$(ssh_run "${client}" \
            "dirname \"\$(find '${dest}' -name checksums.sha256 -print -quit)\"" 2>/dev/null || true)"
    fi
    [[ -n "${restored}" && "${restored}" != "." ]] || return 0
    state_set restored_path "${client}" "${restored}"
}

setup_file() {
    local client
    for client in ${CLIENTS}; do
        _restore "${client}" "$(client_hostname "${client}")" || true
    done
}

_skip_no_backup() {
    [[ "$(state_get backup_ok "$1" 0)" == "1" ]] || skip "no backup to restore"
}

# ── Debian ────────────────────────────────────────────────────────────────────

@test "ws_restore completes for e2e-debian" {
    skip_unless_client debian; _skip_no_backup debian
    [[ "$(state_get restore_ok debian 0)" == "1" ]]
}

@test "restored tree in e2e-debian" {
    skip_unless_client debian; _skip_no_backup debian
    [[ -n "$(state_get restored_path debian)" ]]
}

@test "restored files are identical bit-for-bit (debian)" {
    skip_unless_client debian; _skip_no_backup debian
    local restored; restored="$(state_get restored_path debian)"
    [[ -n "${restored}" ]] || skip "restored tree not found"
    run ssh_run debian "cd '${restored}' && sha256sum --quiet -c checksums.sha256"
    assert_success
}

@test "the extended attribute is restored (debian)" {
    skip_unless_client debian; _skip_no_backup debian
    local restored; restored="$(state_get restored_path debian)"
    [[ -n "${restored}" ]] || skip "restored tree not found"
    run ssh_run debian "getfattr -d '${restored}/xattr.txt' 2>/dev/null || true"
    assert_output --partial "user.woodstock"
}

@test "the POSIX ACL is restored (debian)" {
    skip_unless_client debian; _skip_no_backup debian
    local restored; restored="$(state_get restored_path debian)"
    [[ -n "${restored}" ]] || skip "restored tree not found"
    run ssh_run debian "getfacl -p '${restored}/acl.txt' 2>/dev/null || true"
    assert_output --partial "user:nobody"
}

@test "the symbolic link is restored as a link (debian)" {
    skip_unless_client debian; _skip_no_backup debian
    local restored; restored="$(state_get restored_path debian)"
    [[ -n "${restored}" ]] || skip "restored tree not found"
    run ssh_run debian "test -L '${restored}/link-to-big-1'"
    assert_success
}

# ── FreeBSD ───────────────────────────────────────────────────────────────────
# Metadata tooling differs: FreeBSD uses getextattr/lsextattr, and its UFS root
# is not mounted with `acls`, so ACLs were never captured in the first place
# and must not be expected back.

@test "ws_restore completes for e2e-freebsd" {
    skip_unless_client freebsd; _skip_no_backup freebsd
    [[ "$(state_get restore_ok freebsd 0)" == "1" ]]
}

@test "restored tree in e2e-freebsd" {
    skip_unless_client freebsd; _skip_no_backup freebsd
    [[ -n "$(state_get restored_path freebsd)" ]]
}

@test "restored files are identical bit-for-bit (freebsd)" {
    skip_unless_client freebsd; _skip_no_backup freebsd
    local restored; restored="$(state_get restored_path freebsd)"
    [[ -n "${restored}" ]] || skip "restored tree not found"
    run ssh_run freebsd "cd '${restored}' && sha256sum --quiet -c checksums.sha256"
    assert_success
}

@test "the extended attribute is restored (freebsd)" {
    skip_unless_client freebsd; _skip_no_backup freebsd
    local restored; restored="$(state_get restored_path freebsd)"
    [[ -n "${restored}" ]] || skip "restored tree not found"
    run ssh_run freebsd "lsextattr user '${restored}/xattr.txt' 2>/dev/null || true"
    assert_output --partial "woodstock"
}

@test "the POSIX ACL is restored (freebsd)" {
    skip_unless_client freebsd; _skip_no_backup freebsd
    skip "UFS is not mounted with acls option on this image"
}

@test "the symbolic link is restored as a link (freebsd)" {
    skip_unless_client freebsd; _skip_no_backup freebsd
    local restored; restored="$(state_get restored_path freebsd)"
    [[ -n "${restored}" ]] || skip "restored tree not found"
    run ssh_run freebsd "test -L '${restored}/link-to-big-1'"
    assert_success
}

# ── Windows ───────────────────────────────────────────────────────────────────
# NTFS has no POSIX xattr/ACL. The Windows equivalents — alternate data
# streams and NTFS ACEs — are not part of the manifest format, so nothing here
# is expected to survive the round trip.

@test "ws_restore completes for e2e-windows" {
    skip_unless_client windows; _skip_no_backup windows
    [[ "$(state_get restore_ok windows 0)" == "1" ]]
}

@test "restored tree in e2e-windows" {
    skip_unless_client windows; _skip_no_backup windows
    [[ -n "$(state_get restored_path windows)" ]]
}

# -Encoding UTF8 on the read is essential: gen-checksums.ps1 writes the file
# as BOM-less UTF-8, but PowerShell 5.1 reads with the ANSI code page by
# default, so `space and éàü.txt` comes back mangled and the check reports it
# missing when the restore was in fact perfect.
@test "restored files are identical bit-for-bit (windows)" {
    skip_unless_client windows; _skip_no_backup windows
    local restored; restored="$(state_get restored_path windows)"
    [[ -n "${restored}" ]] || skip "restored tree not found"
    run ssh_run windows "\$ErrorActionPreference='Stop'
\$root='${restored}'
\$bad=@()
foreach (\$line in Get-Content -Encoding UTF8 (Join-Path \$root 'checksums.sha256')) {
    if (\$line -notmatch '^([0-9a-f]{64})  \./(.*)\$') { continue }
    \$want=\$Matches[1]; \$rel=\$Matches[2] -replace '/','\\'
    \$path=Join-Path \$root \$rel
    if (-not (Test-Path -LiteralPath \$path)) { \$bad += \"missing \$rel\"; continue }
    \$got=(Get-FileHash -Algorithm SHA256 -LiteralPath \$path).Hash.ToLower()
    if (\$got -ne \$want) { \$bad += \"mismatch \$rel\" }
}
if (\$bad.Count) { \$bad -join '; '; exit 1 }"
    assert_success
}

@test "the extended attribute is restored (windows)" {
    skip_unless_client windows; _skip_no_backup windows
    skip "NTFS has no POSIX xattr; ADS streams are not carried by the manifest"
}

@test "the POSIX ACL is restored (windows)" {
    skip_unless_client windows; _skip_no_backup windows
    skip "NTFS has no POSIX ACL"
}

@test "the symbolic link is restored as a link (windows)" {
    skip_unless_client windows; _skip_no_backup windows
    local restored; restored="$(state_get restored_path windows)"
    [[ -n "${restored}" ]] || skip "restored tree not found"
    run ssh_run windows "if (-not ((Get-Item -LiteralPath '${restored}\\link-to-big-1' -Force).LinkType)) { exit 1 }"
    assert_success
}
