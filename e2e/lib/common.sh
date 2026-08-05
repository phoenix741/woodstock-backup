#!/usr/bin/env bash
# Shared helpers: configuration loading and logging.
#
# Sourced by run.sh, images/build-all.sh, and every tests/*.bats file (via
# tests/test_helper.bash).

set -euo pipefail

E2E_DIR="${E2E_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
REPO_ROOT="$(cd "${E2E_DIR}/.." && pwd)"

IMAGES_DIR="${E2E_DIR}/images"
CACHE_DIR="${IMAGES_DIR}/cache"
RUNS_DIR="${E2E_DIR}/run"
SSH_KEY="${CACHE_DIR}/id_e2e"

# ─── Configuration ───────────────────────────────────────────────────────────

# Load e2e.conf, then apply defaults for anything it did not set. Values already
# present in the environment always win, so `VM_MEM=4096 ./run.sh` works.
load_config() {
    local conf="${E2E_DIR}/e2e.conf"
    if [[ -f "${conf}" ]]; then
        # The config file assigns unconditionally, so anything already set in
        # the environment — or by a command-line flag such as --keep — has to be
        # restored afterwards, otherwise the file silently wins.
        local -A preset=()
        local var
        for var in KEEP_VMS ARTIFACTS_SOURCE ARTIFACTS_DIR GITEA_RUN_ID GITEA_TOKEN \
                   DEBIAN_ISO FREEBSD_IMAGE_URL WINDOWS_ISO VM_MEM VM_CPUS \
                   BOOT_TIMEOUT BACKUP_TIMEOUT; do
            [[ -n "${!var+set}" ]] && preset["${var}"]="${!var}"
        done

        # shellcheck disable=SC1090
        source "${conf}"

        for var in "${!preset[@]}"; do
            printf -v "${var}" '%s' "${preset[${var}]}"
        done
    fi

    : "${DEBIAN_ISO:=/server/ISO/debian-13.6.0-amd64-netinst.iso}"
    : "${FREEBSD_IMAGE_URL:=https://download.freebsd.org/releases/VM-IMAGES/15.1-RELEASE/amd64/Latest/FreeBSD-15.1-RELEASE-amd64-BASIC-CLOUDINIT-ufs.qcow2.xz}"
    : "${WINDOWS_ISO:=/server/ISO/windows/Win11_25H2_English_x64_v2.iso}"

    : "${ARTIFACTS_SOURCE:=local}"
    : "${ARTIFACTS_DIR:=${E2E_DIR}/artefacts}"
    : "${GITEA_URL:=https://gogs.shadoware.org}"
    : "${GITEA_REPO:=ShadowareOrg/woodstock-backup}"
    : "${GITEA_RUN_ID:=}"
    : "${GITEA_TOKEN:=}"

    : "${VM_MEM:=2048}"
    : "${VM_CPUS:=2}"
    : "${VM_DISK:=12G}"
    # Windows needs its own size. 12G holds the installed system with barely a
    # gigabyte to spare, and VSS cannot create a shadow copy without free space
    # — which is the one thing the Windows client exists to prove. qcow2 is
    # sparse, so the larger figure costs nothing until it is used.
    : "${WINDOWS_DISK:=48G}"
    : "${DATA_DISK:=6G}"

    : "${LAB_NET:=10.66.0}"
    : "${LAB_MCAST:=230.0.66.1:14566}"

    : "${PORT_SSH_SERVER:=22610}"
    : "${PORT_API_SERVER:=13000}"
    : "${PORT_SSH_DEBIAN:=22620}"
    : "${PORT_SSH_FREEBSD:=22621}"
    : "${PORT_SSH_WINDOWS:=22622}"

    # Tight on purpose: this data set backs up in seconds, so a long wait means
    # a failure that should be reported now rather than in half an hour.
    : "${BOOT_TIMEOUT:=300}"
    : "${BACKUP_TIMEOUT:=240}"
    : "${SSH_TIMEOUT:=120}"
    : "${KEEP_VMS:=0}"

    # Fixed lab addresses. The server dials the agents on this network, so the
    # agents must be reachable at a stable address.
    IP_SERVER="${LAB_NET}.10"
    IP_DEBIAN="${LAB_NET}.20"
    IP_FREEBSD="${LAB_NET}.21"
    IP_WINDOWS="${LAB_NET}.22"

    API="http://127.0.0.1:${PORT_API_SERVER}/api"

    export E2E_DIR REPO_ROOT IMAGES_DIR CACHE_DIR RUNS_DIR SSH_KEY
    export IP_SERVER IP_DEBIAN IP_FREEBSD IP_WINDOWS API
}

# ─── Logging ─────────────────────────────────────────────────────────────────

if [[ -t 1 ]]; then
    C_RESET=$'\033[0m'; C_RED=$'\033[31m'; C_GREEN=$'\033[32m'
    C_YELLOW=$'\033[33m'; C_BLUE=$'\033[34m'; C_DIM=$'\033[2m'
else
    C_RESET=""; C_RED=""; C_GREEN=""; C_YELLOW=""; C_BLUE=""; C_DIM=""
fi

log()   { printf '%s[%s]%s %s\n'  "${C_BLUE}"   "$(date +%H:%M:%S)" "${C_RESET}" "$*" >&2; }
warn()  { printf '%s[warn]%s %s\n' "${C_YELLOW}" "${C_RESET}" "$*" >&2; }
error() { printf '%s[error]%s %s\n' "${C_RED}"   "${C_RESET}" "$*" >&2; }
debug() { [[ "${E2E_DEBUG:-0}" == "1" ]] && printf '%s[debug] %s%s\n' "${C_DIM}" "$*" "${C_RESET}" >&2 || true; }

die() { error "$*"; exit 1; }

# Run a command, retrying until it succeeds or the deadline passes.
# Usage: retry <timeout_seconds> <interval_seconds> <command...>
retry() {
    local timeout="$1" interval="$2"; shift 2
    local deadline=$(( $(date +%s) + timeout ))
    while true; do
        if "$@"; then return 0; fi
        if (( $(date +%s) >= deadline )); then
            return 1
        fi
        sleep "${interval}"
    done
}

require_tools() {
    local missing=()
    for tool in "$@"; do
        command -v "${tool}" >/dev/null 2>&1 || missing+=("${tool}")
    done
    if (( ${#missing[@]} )); then
        die "missing required tools: ${missing[*]}"
    fi
}

# Wait for a process to exit, or kill it hard after <timeout> seconds.
wait_or_kill() {
    local pid="$1" timeout="$2" what="$3"
    local deadline=$(( $(date +%s) + timeout ))
    while kill -0 "${pid}" 2>/dev/null; do
        if (( $(date +%s) >= deadline )); then
            error "${what} did not finish within ${timeout}s"
            kill -9 "${pid}" 2>/dev/null || true
            return 1
        fi
        sleep 5
    done
    return 0
}

# The one ed25519 keypair shared by every guest and by the harness: baked into
# each golden image at build time (Packer's ssh_private_key_file /
# cd_content/http_content-rendered authorized_keys) and used by every
# ssh_run/scp_to call afterwards (lib/remote.sh). Never let a build tool
# generate its own ephemeral key instead — the harness would lose access to
# the image it just built.
ensure_ssh_key() {
    mkdir -p "${CACHE_DIR}"
    if [[ ! -f "${SSH_KEY}" ]]; then
        log "generating harness SSH key"
        ssh-keygen -q -t ed25519 -N '' -C 'woodstock-e2e' -f "${SSH_KEY}"
    fi
    chmod 600 "${SSH_KEY}"
}
