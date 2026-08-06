#!/usr/bin/env bash
# Helpers that know about Woodstock itself: API polling, backup identifiers,
# pool size. Used by the tests/*.sh scripts.
#
# The shape of GET /api/hosts/{h}/backups comes from `Backup` in
# server-rs/src/api/dto/backup.rs — serialised camelCase:
#
#   [{ "id": "<uuid>", "number": 1,
#      "status": {"statusType": "completed", …},
#      "fileCount": …, "newFileSize": …, … }]
#
# There is no boolean "done" flag: completion is statusType == "completed", and
# aborted/failed are terminal too — polling has to stop on those instead of
# waiting out the whole timeout.

# Host name a client VM registers under. Kept distinct from the VM role so a
# future run can host several agents of the same OS.
client_hostname() {
    case "$1" in
        debian)  echo "e2e-debian"  ;;
        freebsd) echo "e2e-freebsd" ;;
        windows) echo "e2e-windows" ;;
        *) die "unknown client: $1" ;;
    esac
}

# Raw backup list of a host, as JSON. Never fails: callers poll on it.
backups_json() {
    curl -fsS "${API}/hosts/$1/backups" 2>/dev/null || echo '[]'
}

# Compact state of every backup, for failure messages.
backup_state() {
    backups_json "$1" \
        | jq -r 'if length == 0 then "no backups"
                 else map("#\(.number):\(.status.statusType)\(if .errorMessage then " (" + .errorMessage + ")" else "" end)") | join(", ")
                 end' 2>/dev/null || echo "state unavailable"
}

# True once the host has at least $2 (default 1) completed backups.
backup_count_at_least() {
    backups_json "$1" \
        | jq -e --argjson n "${2:-1}" \
            'map(select(.status.statusType == "completed")) | length >= $n' >/dev/null 2>&1
}

backup_completed() { backup_count_at_least "$1" 1; }

# True when the newest backup reached a terminal state that is not success, so
# a poll can give up immediately instead of waiting out BACKUP_TIMEOUT.
backup_failed() {
    backups_json "$1" \
        | jq -e '(.[-1] // {}) | .status.statusType as $s
                 | ($s == "failed" or $s == "aborted")' >/dev/null 2>&1
}

# Poll until the host has $2 completed backups, giving up early on failure.
wait_for_backup() {
    local host="$1" want="${2:-1}"
    local deadline=$(( $(date +%s) + BACKUP_TIMEOUT ))
    while true; do
        backup_count_at_least "${host}" "${want}" && return 0
        backup_failed "${host}" && return 1
        (( $(date +%s) >= deadline )) && return 1
        sleep 10
    done
}

# Poll until a completed backup appears whose UUID differs from $2.
#
# Counting backups does not work for a second run: retention keeps one backup
# per hourly slot, so two backups taken minutes apart collapse into one and the
# older is purged as surplus. Identity is the only stable signal.
wait_for_new_backup() {
    local host="$1" previous_uuid="$2"
    local deadline=$(( $(date +%s) + BACKUP_TIMEOUT ))
    local current
    while true; do
        current="$(latest_backup_uuid "${host}")"
        [[ -n "${current}" && "${current}" != "${previous_uuid}" ]] && return 0
        backup_failed "${host}" && return 1
        (( $(date +%s) >= deadline )) && return 1
        sleep 10
    done
}

# Configuration and data directories differ between platforms: Debian uses
# /etc/woodstock and /var/lib/woodstock, FreeBSD /usr/local/etc/woodstock and
# /var/db/woodstock.
client_config_dir() {
    case "$1" in
        freebsd) echo "/usr/local/etc/woodstock" ;;
        windows) echo "C:/ProgramData/woodstock" ;;
        *)       echo "/etc/woodstock" ;;
    esac
}

# The primary share a client offers: the one holding the generated test data,
# the one ws_restore is given and the one whose manifest the backup assertions
# read. It names the manifest on disk (percent-encoded), so every consumer has
# to agree on the exact spelling — including the backslashes on Windows.
client_share() {
    case "$1" in
        windows) echo 'C:\e2e' ;;
        *)       echo "/home"  ;;
    esac
}

# Every share declared in <host>.yml, comma-separated.
#
# Windows gets a second one: NTUSER.DAT is held open by the logged-in user, so
# it can only be read through a shadow copy. Its presence in the C:\Users
# manifest is the strongest single proof that VSS actually engaged — and it
# lives nowhere near the test-data share, so it has to be backed up explicitly.
client_shares() {
    case "$1" in
        windows) echo 'C:\e2e,C:\Users' ;;
        *)       echo "$(client_share "$1")" ;;
    esac
}

# Where inside that share gen-testdata writes. On Unix the share is /home and
# the data goes in /home/e2e; on Windows the share *is* the data directory.
client_data_dir() {
    case "$1" in
        windows) echo 'C:\e2e' ;;
        *)       echo "/home/e2e" ;;
    esac
}

server_backup_path() {
    case "${SERVER_OS}" in
        freebsd) echo "/var/db/woodstock" ;;
        *)       echo "/var/lib/woodstock" ;;
    esac
}

server_config_dir() {
    case "${SERVER_OS}" in
        freebsd) echo "/usr/local/etc/woodstock" ;;
        *)       echo "/etc/woodstock" ;;
    esac
}

# ws_console and ws_restore read BACKUP_PATH from the environment and fall back
# to the Linux path when it is unset (`/var/lib/woodstock`,
# woodstock-rs/src/config/core.rs). The services get it from server.env, but a
# command run over SSH does not source that file — so every manual invocation
# has to carry it, or it would look for a pool that does not exist on FreeBSD.
ws_cli() {
    echo "BACKUP_PATH=$(server_backup_path)"
}

# Restart one server service, whichever init system runs it. The unit names use
# dashes on Debian and underscores on FreeBSD.
server_restart_cmd() {
    local unit="$1"
    case "${SERVER_OS}" in
        freebsd) echo "service ${unit//-/_} restart" ;;
        *)       echo "systemctl restart ${unit//_/-}" ;;
    esac
}

# Server-side logs, whichever init system runs them: journald on Debian, plain
# files under /var/log/woodstock on FreeBSD.
server_log() {
    local unit="$1"
    case "${SERVER_OS}" in
        freebsd)
            # rc.d writes one file per service, named after the rc script minus
            # the `woodstock_` prefix: woodstock_client_api -> client_api.log
            ssh_run server "cat /var/log/woodstock/${unit#woodstock_}.log 2>/dev/null" || true
            ;;
        *)
            # No --since: the VM is minutes old, the whole journal is relevant,
            # and a time window that is slightly off silently matches nothing.
            ssh_run server "journalctl -u ${unit//_/-} --no-pager" || true
            ;;
    esac
}

# `stat` format strings differ between GNU and BSD.
stat_owner_mode() {
    local role="$1" path="$2"
    case "$(vm_os "${role}")" in
        freebsd) ssh_run "${role}" "stat -f '%Su %Sg %Lp' '${path}'" ;;
        *)       ssh_run "${role}" "stat -c '%U %G %a' '${path}'" ;;
    esac
}

# Which OS a VM role runs.
vm_os() {
    case "$1" in
        server) echo "${SERVER_OS}" ;;
        *)      echo "$1" ;;
    esac
}

# Is the agent running? On FreeBSD `service … status` exits 0 only when it
# resolves a live pid through the pidfile, which doubles as the regression test
# for the daemon(8) wrapping.
client_status_cmd() {
    case "$1" in
        freebsd) echo "service woodstock_client status" ;;
        # The service registered by `ws_client_daemon install-service`
        # (client-rs/src/winserv.rs, SERVICE_NAME).
        windows) echo "if ((Get-Service woodstock_client_daemon).Status -ne 'Running') { exit 1 }" ;;
        *)       echo "systemctl is-active --quiet woodstock-client" ;;
    esac
}

# (Re)start the agent, whichever init system the guest uses.
#
# `service … restart` fails on FreeBSD when the service is not running yet —
# which is the normal state before enrollment — and the caller runs under
# `set -e`, so the whole deployment would abort. Stop-then-start is idempotent.
client_restart_cmd() {
    case "$1" in
        freebsd) echo "service woodstock_client stop >/dev/null 2>&1 || true; service woodstock_client start" ;;
        # Stop-then-start rather than Restart-Service: the agent is expected to
        # be stopped before enrollment (no certificates), and Restart-Service
        # errors on a stopped service.
        #
        # The Stop is backed up by killing the process. An agent stuck retrying
        # authentication does not answer the SCM stop control in time — observed
        # as "cannot be stopped ... CouldNotStopService" — and without the kill
        # the following Start silently keeps the old, wedged process.
        windows) echo "Stop-Service woodstock_client_daemon -Force -ErrorAction SilentlyContinue; Start-Sleep -Seconds 2; Stop-Process -Name ws_client_daemon -Force -ErrorAction SilentlyContinue; Start-Sleep -Seconds 3; Start-Service woodstock_client_daemon" ;;
        *)       echo "systemctl restart woodstock-client" ;;
    esac
}

# Number of files recorded by the newest completed backup.
#
# A backup the worker skipped ("host not reachable") is still reported as
# `completed`, with no manifest on disk and a file count of zero — so
# statusType alone does not tell a real backup from an empty one.
latest_backup_file_count() {
    backups_json "$1" \
        | jq -r 'map(select(.status.statusType == "completed")) | (.[-1] // {}) | .fileCount // 0' \
          2>/dev/null || echo 0
}

# Sequential number of the newest completed backup — a display label only; the
# on-disk directory is named after the UUID.
latest_backup_number() {
    backups_json "$1" \
        | jq -r 'map(select(.status.statusType == "completed")) | (.[-1] // {}) | .number // empty' \
          2>/dev/null || true
}

# UUID of the newest completed backup — required by the /backups/{id}/… routes,
# which parse that segment as a UUID even though they declare {number}.
latest_backup_uuid() {
    backups_json "$1" \
        | jq -r 'map(select(.status.statusType == "completed")) | (.[-1] // {}) | .id // empty' \
          2>/dev/null || true
}

# Total size of the chunk pool, in bytes.
server_pool_size() {
    # GNU du has -b (bytes) but no -A; BSD du has -A but no -b. Neither accepts
    # the other's flag, and a silently failing du would report 0 and make the
    # deduplication assertion pass for the wrong reason.
    local kib
    case "${SERVER_OS}" in
        freebsd) kib="$(ssh_run server "du -Aks $(server_backup_path)/pool | cut -f1" 2>/dev/null || echo "")" ;;
        *)       kib="$(ssh_run server "du -sk $(server_backup_path)/pool | cut -f1" 2>/dev/null || echo "")" ;;
    esac
    [[ "${kib}" =~ ^[0-9]+$ ]] || { warn "could not read the pool size"; echo 0; return; }
    echo $(( kib * 1024 ))
}

# Absolute path of a share manifest on the server, for ws_console.
#
# Three things that are easy to get wrong, all verified on a real backup:
#   * ws_console resolves its argument against the current directory, not
#     against BACKUP_PATH, so the path has to be absolute.
#   * the backup directory is named after the *UUID*, not the sequential number
#     (`get_backup_destination_directory`, woodstock-rs/src/config/backups.rs).
#   * the share name is percent-encoded by `mangle()`, which escapes every
#     non-alphanumeric byte — /home becomes %2Fhome, and a share containing a
#     dash or a dot escapes those too.
manifest_path() {
    local host="$1" uuid="$2" share="${3:-/home}"
    local backup_path="${SERVER_BACKUP_PATH:-$(server_backup_path)}"
    local mangled=""
    local i char
    for (( i = 0; i < ${#share}; i++ )); do
        char="${share:i:1}"
        if [[ "${char}" == [A-Za-z0-9] ]]; then
            mangled+="${char}"
        else
            mangled+="$(printf '%%%02X' "'${char}")"
        fi
    done
    printf '%s/hosts/%s/%s/%s.manifest' "${backup_path}" "${host}" "${uuid}" "${mangled}"
}

human() {
    numfmt --to=iec-i --suffix=B "$1" 2>/dev/null || echo "$1 B"
}

# Extended-attribute names of one file in a `ws_console read-protobuf` dump.
#
# ws_console renders xattr keys and values as YAML arrays of decimal bytes:
#
#     xattr:
#     - key:
#       - 117      # 'u'
#       - 115      # 's'
#       ...
#
# so the names have to be decoded before they can be matched. POSIX ACLs show up
# here too, as `system.posix_acl_access`.
#
# Usage: manifest_xattr_keys <dump file> <path within the share>
manifest_xattr_keys() {
    python3 - "$1" "$2" <<'PY'
import re, sys

dump, target = sys.argv[1], sys.argv[2]
text = open(dump, encoding="utf-8", errors="replace").read()

entry = re.search(
    r"- path: %s\n(.*?)(?=\n- path: |\Z)" % re.escape(target), text, re.S
)
if not entry:
    sys.exit(f"no manifest entry for {target}")

for key in re.findall(r"key:\n((?:\s+- \d+\n)+)", entry.group(1)):
    print(bytes(int(b) for b in re.findall(r"- (\d+)", key)).decode("utf-8", "replace"))
PY
}
