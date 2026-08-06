#!/usr/bin/env bash
# Woodstock Backup — end-to-end suite.
#
#   ./run.sh --server debian --clients debian
#   ./run.sh --server debian --clients debian,freebsd,windows
#   ./run.sh --only 40-backup --clients debian     # reuse VMs from a previous run
#
# Boots one server VM and one VM per client from the golden images, installs
# the packages under test the way a user would, then runs tests/*.bats with
# bats-core, in file order. Results land in run/<timestamp>/ as TAP, a Markdown
# report and the guest logs.

set -euo pipefail

E2E_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/common.sh
source "${E2E_DIR}/lib/common.sh"
# shellcheck source=lib/remote.sh
source "${E2E_DIR}/lib/remote.sh"
# shellcheck source=lib/qemu.sh
source "${E2E_DIR}/lib/qemu.sh"
# shellcheck source=lib/artifacts.sh
source "${E2E_DIR}/lib/artifacts.sh"
# shellcheck source=lib/woodstock.sh
source "${E2E_DIR}/lib/woodstock.sh"
# shellcheck source=lib/state.sh
source "${E2E_DIR}/lib/state.sh"

SERVER_OS="debian"
CLIENTS="debian"
ONLY=""
REUSE=0
KEEP_VMS=0
# 80-upgrade and 90-uninstall take the installation apart, so they only run
# when asked for: otherwise a --only re-run would find nothing installed.
DESTRUCTIVE=0

usage() {
    sed -n '2,12p' "${BASH_SOURCE[0]}" | sed 's/^# \?//'
    exit "${1:-0}"
}

while (( $# )); do
    case "$1" in
        --server)  SERVER_OS="$2"; shift 2 ;;
        --clients) CLIENTS="${2//,/ }"; shift 2 ;;
        # --only implies --reuse only when a previous run left its VMs up;
        # otherwise there is nothing to reuse and the tests would run against
        # machines that do not exist.
        --only)    ONLY="$2"; shift 2 ;;
        --reuse)   REUSE=1; shift ;;
        --keep)    KEEP_VMS=1; shift ;;
        --destructive) DESTRUCTIVE=1; shift ;;
        -h|--help) usage 0 ;;
        *) error "unknown option: $1"; usage 1 ;;
    esac
done

# Exported so load_config's `: "${KEEP_VMS:=0}"` default does not clobber --keep.
export KEEP_VMS

load_config
require_tools qemu-system-x86_64 qemu-img swtpm ssh scp curl jq unzip python3

BATS_BIN="bats"
command -v bats >/dev/null 2>&1 || die "bats not found — run: sudo apt install bats bats-assert bats-support"

export SERVER_OS CLIENTS

# ─── Run directory ───────────────────────────────────────────────────────────

if (( REUSE )); then
    RUN_DIR="$(find "${RUNS_DIR}" -maxdepth 1 -mindepth 1 -type d | sort | tail -1)"
    [[ -n "${RUN_DIR}" ]] || die "--only/--reuse needs a previous run in ${RUNS_DIR}"
    log "reusing ${RUN_DIR}"
else
    RUN_DIR="${RUNS_DIR}/$(date +%Y%m%d-%H%M%S)"
    mkdir -p "${RUN_DIR}"
fi

VM_DIR="${RUN_DIR}/vms"
mkdir -p "${VM_DIR}"
export RUN_DIR VM_DIR

log "run directory: ${RUN_DIR}"
log "server: ${SERVER_OS} — clients: ${CLIENTS}"

# ─── Teardown ────────────────────────────────────────────────────────────────

cleanup() {
    # $? here is the exit status of the last command the script ran before
    # falling off the end into this EXIT trap — for the normal path that is
    # `SUITES_COMPLETED=1`, a plain assignment that is always 0, regardless of
    # whether run_bats_suite actually passed. BATS_RC is what run_bats_suite
    # really returned; fall back to $? only when it was never set (the script
    # died before reaching that point at all, which the block below already
    # forces to rc=1 via the SUITES_COMPLETED check).
    local rc="${BATS_RC:-$?}"
    # `set -e` means an unguarded failure anywhere before run_bats_suite (VM
    # boot, provisioning) aborts the script before a single test runs — and
    # unlike a bats test failure, that leaves no results.tap at all for
    # write_reports to render. A run with no report is much harder to
    # diagnose than a partial one, so synthesize a one-line red one instead of
    # silently producing nothing. This is the same failure mode the old
    # per-suite `source` loop had to guard against ("82 PASS — no failures"
    # for a run that died in 60-incremental and never reached 70-restore) —
    # bats' own declared-before-executed TAP plan (see write_reports) now
    # covers the case where the suite *starts* and dies partway through;
    # this covers the case where it never starts at all.
    if (( ! ${SUITES_COMPLETED:-0} )); then
        {
            echo "TAP version 13"
            echo "1..1"
            echo "not ok 1 the suite started (failure before first test — see provisioning logs)"
        } > "${RUN_DIR}/results.tap"
        : > "${RUN_DIR}/suite-manifest.tsv"
        printf 'run\t1\n' > "${RUN_DIR}/suite-manifest.tsv"
        rc=1
    fi
    # Write the reports first: a run without a report is much harder to diagnose
    # than a partial one.
    write_reports || rc=1
    if (( KEEP_VMS )); then
        warn "leaving the VMs running (--keep); stop them with: pkill -F ${VM_DIR}/<role>.pid"
    elif (( ! REUSE )); then
        collect_logs || true
        vm_stop_all || true
    fi
    exit "${rc}"
}
trap cleanup EXIT

collect_logs() {
    local logs="${RUN_DIR}/logs"
    mkdir -p "${logs}"

    # journalctl exists only on the systemd guests; FreeBSD writes plain files
    # under /var/log/woodstock. Collecting the wrong one loses exactly the log
    # that would explain a failure.
    if [[ "${SERVER_OS}" == "freebsd" ]]; then
        ssh_run server "cat /var/log/woodstock/*.log 2>/dev/null | tail -2000" \
            > "${logs}/server-services.log" 2>&1 || true
    else
        ssh_run server "journalctl -u woodstock-api -u woodstock-client-api -u woodstock-worker -u woodstock-scheduler --no-pager" \
            > "${logs}/server-services.log" 2>&1 || true
    fi
    ssh_run server "cat $(server_backup_path)/logs/*.log 2>/dev/null | tail -2000" \
        > "${logs}/server-app.log" 2>&1 || true

    local client
    for client in ${CLIENTS}; do
        case "${client}" in
            freebsd)
                ssh_run "${client}" "cat /var/log/woodstock/client.log 2>/dev/null | tail -500" \
                    > "${logs}/${client}-agent.log" 2>&1 || true
                ;;
            windows)
                # The agent logs to its configuration directory; the service
                # itself only shows up in the Windows event log.
                ssh_run windows 'Get-Content -Tail 500 (Join-Path $env:ProgramData "woodstock\*.log") -ErrorAction SilentlyContinue' \
                    > "${logs}/${client}-agent.log" 2>&1 || true
                ssh_run windows 'Get-WinEvent -FilterHashtable @{LogName="System"; ProviderName="Service Control Manager"} -MaxEvents 40 | Format-List TimeCreated, Message' \
                    >> "${logs}/${client}-agent.log" 2>&1 || true
                ;;
            *)
                ssh_run "${client}" "journalctl -u woodstock-client --no-pager" \
                    > "${logs}/${client}-agent.log" 2>&1 || true
                ;;
        esac
    done
    cp "${VM_DIR}"/*-console.log "${logs}/" 2>/dev/null || true
}

# ─── Boot and provision ──────────────────────────────────────────────────────

golden_for() {
    case "$1" in
        debian)  echo "${IMAGES_DIR}/debian-golden.qcow2"  ;;
        freebsd) echo "${IMAGES_DIR}/freebsd-golden.qcow2" ;;
        windows) echo "${IMAGES_DIR}/windows-golden.qcow2" ;;
    esac
}

boot_vms() {
    vm_stop_stale

    local overlay
    overlay="${VM_DIR}/server.qcow2"
    vm_overlay "$(golden_for "${SERVER_OS}")" "${overlay}"
    vm_start server "${overlay}"

    local client
    for client in ${CLIENTS}; do
        overlay="${VM_DIR}/${client}.qcow2"
        vm_overlay "$(golden_for "${client}")" "${overlay}"
        if [[ "${client}" == "debian" ]]; then
            # A second disk, formatted btrfs and mounted on /home: btrfs
            # snapshots the mount point, so the share has to be its own volume.
            # FreeBSD has no snapshot backend at all, so it needs no extra disk.
            # shellcheck disable=SC2046
            vm_start "${client}" "${overlay}" $(vm_data_disk "${client}")
        else
            vm_start "${client}" "${overlay}"
        fi
    done

    wait_ssh server
    for client in ${CLIENTS}; do
        wait_ssh "${client}"
    done
}

# The lab-network script is platform-specific: FreeBSD has no /sys and no
# iproute2, so it gets its own implementation of the same contract.
lab_net_script() {
    case "$1" in
        freebsd) echo "${E2E_DIR}/provision/lab-net-freebsd.sh" ;;
        *)       echo "${E2E_DIR}/provision/lab-net.sh" ;;
    esac
}

provision() {
    log "provisioning the server (${SERVER_OS})"
    ssh_run server "mkdir -p /tmp/packages" >/dev/null
    scp_to server "${PKG_DIR}/deb" /tmp/packages/
    scp_to server "${PKG_DIR}/pkg" /tmp/packages/
    scp_to server "$(lab_net_script "${SERVER_OS}")" "/tmp/$(basename "$(lab_net_script "${SERVER_OS}")")"
    scp_to server "${E2E_DIR}/provision/server-${SERVER_OS}.sh" /tmp/provision.sh

    # "<name>|<share>" pairs: the share is platform-specific, and the server has
    # no way to guess it — it just writes what it is told into <host>.yml.
    local hostnames=""
    local client
    for client in ${CLIENTS}; do
        hostnames+="$(client_hostname "${client}")|$(client_shares "${client}") "
    done

    # Provisioning installs packages, so it gets a longer allowance than a plain
    # remote command — but still a bounded one.
    SSH_TIMEOUT=$(( SSH_TIMEOUT * 5 )) \
    ssh_run server "LAB_IP='${IP_SERVER}' HOSTS='${hostnames}' HOST_PASSWORD='${HOST_PASSWORD}' bash /tmp/provision.sh" \
        > "${RUN_DIR}/provision-server.log" 2>&1 \
        || { tail -30 "${RUN_DIR}/provision-server.log" >&2; die "server provisioning failed"; }

    for client in ${CLIENTS}; do
        log "provisioning the ${client} client"

        # Windows takes a different route end to end: PowerShell rather than a
        # shell script, no lab-net script (images/packer/windows/cd/setup.ps1
        # configured the NIC by MAC at image build time), and only the .exe it
        # needs.
        if [[ "${client}" == "windows" ]]; then
            ssh_run windows 'New-Item -ItemType Directory -Force -Path C:\packages\windows | Out-Null' >/dev/null
            scp_to windows "${PKG_DIR}/windows" 'C:/packages/'
            scp_to windows "${E2E_DIR}/data/gen-testdata.ps1" 'C:/packages/'
            scp_to windows "${E2E_DIR}/data/gen-checksums.ps1" 'C:/packages/'
            scp_to windows "${E2E_DIR}/provision/client-windows.ps1" 'C:/packages/provision.ps1'

            # Generating ~200 MiB of incompressible test data is the slow part,
            # and PowerShell is no faster at it than dd — hence the wide margin.
            # HOST_UTC lets the guest correct its clock: the image installs on
            # the default Pacific timezone and reads QEMU's UTC RTC as local
            # time, ending up hours ahead in UTC — enough for every JWT the
            # server issues to look already expired.
            SSH_TIMEOUT=$(( SSH_TIMEOUT * 10 )) \
            ssh_run windows "\$env:LAB_IP='$(vm_ip windows)'; \$env:HOSTNAME_='$(client_hostname windows)'; \$env:HOST_UTC='$(date -u +'%Y-%m-%dT%H:%M:%SZ')'; powershell -NoProfile -ExecutionPolicy Bypass -File C:\\packages\\provision.ps1" \
                > "${RUN_DIR}/provision-${client}.log" 2>&1 \
                || { tail -30 "${RUN_DIR}/provision-${client}.log" >&2; die "${client} provisioning failed"; }
            continue
        fi

        ssh_run "${client}" "mkdir -p /tmp/packages" >/dev/null
        scp_to "${client}" "${PKG_DIR}/deb" /tmp/packages/
        scp_to "${client}" "${PKG_DIR}/pkg" /tmp/packages/
        scp_to "${client}" "${PKG_DIR}/windows" /tmp/packages/
        scp_to "${client}" "$(lab_net_script "${client}")" "/tmp/$(basename "$(lab_net_script "${client}")")"
        scp_to "${client}" "${E2E_DIR}/data/gen-testdata.sh" /tmp/
        scp_to "${client}" "${E2E_DIR}/provision/client-${client}.sh" /tmp/provision.sh

        SSH_TIMEOUT=$(( SSH_TIMEOUT * 5 )) \
        ssh_run "${client}" "LAB_IP='$(vm_ip "${client}")' HOSTNAME_='$(client_hostname "${client}")' bash /tmp/provision.sh" \
            > "${RUN_DIR}/provision-${client}.log" 2>&1 \
            || { tail -30 "${RUN_DIR}/provision-${client}.log" >&2; die "${client} provisioning failed"; }
    done

    # The API is reachable from the host only through the port forward.
    retry 120 5 curl -fsS -o /dev/null "${API}/hosts" \
        || die "the management API never answered on 127.0.0.1:${PORT_API_SERVER}"
}

# ─── Reporting ───────────────────────────────────────────────────────────────
#
# results.tap is bats' own TAP13 output — no longer generated from a TSV of
# check() calls (see lib/state.sh header for why the old model broke under
# --only and under bats' per-test isolation). report.md is derived from it
# below.
#
# bats runs a multi-file invocation in exactly the file order given, with each
# file's tests contiguous, so suite-manifest.tsv (one "<name>\t<test count>"
# line per included file, written by run_bats_suite before bats starts)
# is enough to slice the flat, globally-numbered TAP stream back into the
# original per-suite grouping — without bats knowing "suites" exist at all.
#
# The plan-vs-actual comparison in the parser below is what makes a run that
# died partway through — bats itself killed by the deadline, or crashing
# outright — render as red instead of a smaller, quieter green: the "1..N"
# line declares N before a single test runs, so any test short of N that
# never produced an ok/not ok line is reported as a failure, not omitted.

write_reports() {
    local tap="${RUN_DIR}/results.tap"
    local manifest="${RUN_DIR}/suite-manifest.tsv"
    local md="${RUN_DIR}/report.md"

    if [[ ! -f "${tap}" ]]; then
        # Reachable only if run_bats_suite ran (SUITES_COMPLETED=1, so
        # cleanup's own synthesized results.tap did not kick in) yet bats
        # still produced no report.tap to rename — e.g. the report formatter
        # itself failed to write. Silently returning success here is exactly
        # the false-green class this whole reporting rewrite exists to close.
        error "no ${tap} — bats produced no report despite completing"
        return 1
    fi
    [[ -f "${manifest}" ]] || : > "${manifest}"

    local counts
    counts="$(python3 - "${tap}" "${manifest}" "${md}" \
                        "${SERVER_OS}" "${CLIENTS// /, }" "${ARTIFACTS_SOURCE}" \
                        "${GITEA_RUN_ID:-}" "$(basename "${RUN_DIR}")" <<'PY'
import re, sys

tap_path, manifest_path, md_path, server_os, clients, artifacts_source, gitea_run_id, run_name = sys.argv[1:9]

suites = []
with open(manifest_path, encoding="utf-8") as f:
    for line in f:
        line = line.rstrip("\n")
        if not line:
            continue
        name, count = line.split("\t")
        suites.append((name, int(count)))

bounds = []
acc = 0
for name, count in suites:
    acc += count
    bounds.append((acc, name))

def suite_for(n):
    for bound, name in bounds:
        if n <= bound:
            return name
    return bounds[-1][1] if bounds else "?"

with open(tap_path, encoding="utf-8", errors="replace") as f:
    lines = f.readlines()

plan_total = None
line_re = re.compile(r'^(not )?ok (\d+) (.*)$')
rows = []  # (suite, status, desc, detail)
pass_n = fail_n = skip_n = 0

i = 0
while i < len(lines):
    raw = lines[i].rstrip("\n")
    if plan_total is None:
        m = re.match(r'^1\.\.(\d+)$', raw)
        if m:
            plan_total = int(m.group(1))
            i += 1
            continue
    m = line_re.match(raw)
    if not m:
        i += 1
        continue
    is_not_ok = bool(m.group(1))
    num = int(m.group(2))
    rest = m.group(3)
    skip_m = re.search(r'#\s*SKIP\b\s*(.*)$', rest)
    if skip_m:
        desc = rest[:skip_m.start()].rstrip()
        status, detail = "SKIP", skip_m.group(1).strip()
    elif is_not_ok:
        desc, status = rest, "FAIL"
        msg_lines = []
        in_msg = False
        j = i + 1
        while j < len(lines) and not line_re.match(lines[j].rstrip("\n")):
            l = lines[j]
            if l.strip().startswith("message:"):
                in_msg = True
            elif in_msg and l.startswith("    "):
                msg_lines.append(l.strip())
            elif l.strip() == "...":
                in_msg = False
            j += 1
        detail = " ".join(msg_lines)
        i = j - 1
    else:
        desc, status, detail = rest, "PASS", ""

    rows.append((suite_for(num), status, desc, detail))
    if status == "PASS": pass_n += 1
    elif status == "FAIL": fail_n += 1
    elif status == "SKIP": skip_n += 1
    i += 1

seen = len(rows)
incomplete = plan_total is not None and seen < plan_total
if incomplete:
    missing = plan_total - seen
    rows.append((suite_for(plan_total), "FAIL",
                 f"the suite ran to completion ({missing} test(s) never executed — "
                 "stopped before the end, see the run log)", ""))
    fail_n += 1

with open(md_path, "w", encoding="utf-8") as out:
    out.write(f"# E2E Report — {run_name}\n\n")
    out.write("| | |\n|---|---|\n")
    out.write(f"| Server | {server_os} |\n")
    out.write(f"| Clients | {clients} |\n")
    pkg_line = artifacts_source + (f" (run {gitea_run_id})" if gitea_run_id else "")
    out.write(f"| Packages | {pkg_line} |\n")
    out.write(f"| Result | **{pass_n} PASS**, {fail_n} FAIL, {skip_n} SKIP |\n")

    current = None
    for suite, status, desc, detail in rows:
        if suite != current:
            current = suite
            out.write(f"\n## {suite}\n\n")
        if status == "PASS":
            out.write(f"- [x] {desc}\n")
        elif status == "FAIL":
            out.write(f"- [ ] **FAILURE** — {desc}\n")
            if detail:
                detail = detail.replace("`", "'")
                out.write(f"  - `{detail}`\n")
        elif status == "SKIP":
            suffix = f" ({detail})" if detail else ""
            out.write(f"- [ ] _skipped_ — {desc}{suffix}\n")

print(f"{pass_n} {fail_n} {skip_n} {len(rows)}")
PY
)"
    read -r pass fail skipped total <<<"${counts}"

    echo
    if (( fail == 0 )); then
        printf '%s%s PASS, %s SKIP — no failures%s\n' "${C_GREEN}" "${pass}" "${skipped}" "${C_RESET}"
    else
        printf '%s%s FAILURE(S)%s out of %s tests\n' "${C_RED}" "${fail}" "${C_RESET}" "${total}"
        awk -F'## ' '/^## /{suite=$2} /^- \[ \] \*\*FAILURE\*\*/{sub(/^- \[ \] \*\*FAILURE\*\* — /,""); printf "  ✗ [%s] %s\n", suite, $0}' "${md}"
    fi
    echo "rapport : ${md}"
    return $(( fail > 0 ))
}

# run_bats_suite — resolve the filtered, ordered list of tests/*.bats, write
# suite-manifest.tsv (name + test count per file, in the order bats will run
# them), then invoke bats once for all of them under an overall deadline.
run_bats_suite() {
    local -a files=()
    local f name
    for f in "${E2E_DIR}"/tests/*.bats; do
        name="$(basename "${f}" .bats)"
        if [[ -n "${ONLY}" && "${name}" != *"${ONLY}"* ]]; then
            continue
        fi
        if [[ "${name}" == 8* || "${name}" == 9* ]] && (( ! DESTRUCTIVE )) && [[ -z "${ONLY}" ]]; then
            continue
        fi
        files+=("${f}")
    done
    (( ${#files[@]} > 0 )) || die "no test file matched --only ${ONLY}"

    local manifest="${RUN_DIR}/suite-manifest.tsv"
    : > "${manifest}"
    for f in "${files[@]}"; do
        name="$(basename "${f}" .bats)"
        printf '%s\t%s\n' "${name}" "$("${BATS_BIN}" --count "${f}")" >> "${manifest}"
    done

    # --formatter drives what streams to this terminal live — "pretty" (ticks/
    # crosses, one line per test, running total) when attached to a terminal,
    # plain "tap" otherwise so a backgrounded or piped run still gets one line
    # per test instead of silence until the end. Either way it is teed to
    # console.log so the transcript survives past the terminal.
    #
    # --report-formatter/--output writes the *same* run as TAP13 to disk
    # regardless of what the console sees — that copy, moved to results.tap
    # below, is what write_reports parses. Confirmed empirically that this
    # file is flushed incrementally: killing bats mid-run (SUITE_TIMEOUT)
    # still leaves the "1..N" plan line plus every "ok"/"not ok" emitted
    # before the kill, which is what lets write_reports notice the shortfall
    # instead of reporting a smaller, quieter green.
    local formatter="tap"
    [[ -t 1 ]] && formatter="pretty"

    local rc=0
    timeout "${SUITE_TIMEOUT:-2400}" \
        "${BATS_BIN}" --formatter "${formatter}" --report-formatter tap13 --output "${RUN_DIR}" \
        "${files[@]}" 2>&1 | tee "${RUN_DIR}/console.log" || rc=$?
    mv -f "${RUN_DIR}/report.tap" "${RUN_DIR}/results.tap" 2>/dev/null || true

    if (( rc == 124 )); then
        error "suite deadline exceeded (${SUITE_TIMEOUT:-2400}s) — bats was killed mid-run"
    fi
    return "${rc}"
}

# ─── Main ────────────────────────────────────────────────────────────────────

HOST_PASSWORD="e2e-shared-secret"
export HOST_PASSWORD

if (( ! REUSE )); then
    artifacts_prepare
    boot_vms
    provision
else
    PKG_DIR="${RUN_DIR}/packages"
    export PKG_DIR
fi

# Each @test is its own bats subprocess (see lib/state.sh), so a hang or a
# non-zero exit inside one test cannot abort the rest the way `source`-ing a
# plain script under `set -e` could — the `timeout` inside run_bats_suite is
# only a last-resort watchdog for something hanging outside any individual
# ssh_run's own SSH_TIMEOUT.
BATS_RC=0
run_bats_suite || BATS_RC=$?

# Reaching this line is what "the suite ran to the end" means; the EXIT trap
# turns its absence into a failure. A non-zero BATS_RC (real test failures, or
# the deadline) still counts as completion — it is already reflected in
# results.tap and reported by write_reports.
SUITES_COMPLETED=1

# The reports are produced by the EXIT trap, so they exist whether the suite
# ran to the end or a test aborted it.
