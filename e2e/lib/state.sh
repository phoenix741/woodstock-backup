#!/usr/bin/env bash
# State shared between test suites, persisted to disk instead of held in bash
# associative arrays.
#
# BACKUP_OK/BACKUP_UUID used to live only in 40-backup.sh's process memory
# (run.sh `source`s every tests/*.sh into itself, so that worked as long as
# everything ran in one process). Two things break that:
#   * `--only 50-snapshot` (or 60-incremental, 70-restore) never runs
#     40-backup.sh in the same invocation, so the arrays are empty and every
#     later suite falls back to `:-0` and silently SKIPs — indistinguishable
#     from "the backup genuinely failed".
#   * bats runs each @test as its own process, so nothing declared with
#     `declare -gA` in one test is visible to the next.
#
# One JSON file per run, read/written with jq. Sequential test execution (the
# default, and what bats uses unless --jobs>1 is requested) makes a plain
# read-modify-write safe; this is not a concurrent-safe store.

# ${RUN_DIR} is set by run.sh after this file is sourced, so the path is built
# inside each function rather than once at source time.

# state_set <namespace> <client> <value>
state_set() {
    local ns="$1" client="$2" value="$3"
    local state_file="${RUN_DIR}/state.json"
    [[ -f "${state_file}" ]] || echo '{}' > "${state_file}"
    local tmp
    tmp="$(mktemp "${state_file}.XXXXXX")"
    jq --arg ns "${ns}" --arg client "${client}" --arg v "${value}" \
        '.[$ns][$client] = $v' "${state_file}" > "${tmp}" && mv "${tmp}" "${state_file}"
}

# state_get <namespace> <client> [default] — prints the value, or default
# (empty if omitted) when unset.
state_get() {
    local ns="$1" client="$2" default="${3:-}"
    local state_file="${RUN_DIR}/state.json"
    [[ -f "${state_file}" ]] || { printf '%s' "${default}"; return; }
    local value
    value="$(jq -r --arg ns "${ns}" --arg client "${client}" '.[$ns][$client] // empty' \
        "${state_file}" 2>/dev/null)"
    printf '%s' "${value:-${default}}"
}
