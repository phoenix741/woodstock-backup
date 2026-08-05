# Shared setup for every tests/*.bats file.
#
# bats forks a fresh process per @test (and a *separate* one-off process for
# setup_file/teardown_file), so this whole file — including load_config — runs
# again in each of them. That is what makes lib/state.sh's file-backed state
# necessary: nothing assigned in setup_file is visible to the tests, only what
# is written to disk or inherited via exported environment variables (RUN_DIR,
# SERVER_OS, CLIENTS, … are exported by run.sh before it invokes bats, so they
# reach every test process through normal fork/exec inheritance).

load '/usr/lib/bats/bats-support/load'
load '/usr/lib/bats/bats-assert/load'

E2E_DIR="$(cd "${BATS_TEST_DIRNAME}/.." && pwd)"
# shellcheck source=../lib/common.sh
source "${E2E_DIR}/lib/common.sh"
# shellcheck source=../lib/remote.sh
source "${E2E_DIR}/lib/remote.sh"
# shellcheck source=../lib/qemu.sh
source "${E2E_DIR}/lib/qemu.sh"
# shellcheck source=../lib/artifacts.sh
source "${E2E_DIR}/lib/artifacts.sh"
# shellcheck source=../lib/woodstock.sh
source "${E2E_DIR}/lib/woodstock.sh"
# shellcheck source=../lib/state.sh
source "${E2E_DIR}/lib/state.sh"

load_config

# has_client <name> — true if <name> is part of this run's --clients list.
has_client() {
    [[ " ${CLIENTS} " == *" $1 "* ]]
}

# skip_unless_client <name> — skip the current test with a standard reason
# when that client is not part of this run, instead of silently passing or
# failing. bats has no way to generate a variable number of tests from a
# runtime list, so each client gets its own @test, guarded by this at the top.
skip_unless_client() {
    has_client "$1" || skip "${1} not in --clients"
}

skip_unless_server() {
    [[ "${SERVER_OS}" == "$1" ]] || skip "server is not ${1}"
}

# assert_not_infra_failure — call right after `run` on anything that goes
# through lib/remote.sh's ssh_run/ssh_script/ps_script/scp_to/scp_from, all of
# which wrap the remote call in `timeout "${SSH_TIMEOUT}"`. Exit 124 means "the
# VM stopped answering" (lib/remote.sh's own comment), not "the product
# returned the wrong thing" — collapsing the two into one undifferentiated
# **FAILURE** means every red run needs a manual SSH-in-and-check before
# anyone can tell infra flake from a real regression. TAP has no third status
# beyond ok/not-ok/skip, so the distinction has to live in the failure message
# itself — grep report.md for "TIMEOUT (infra)" to separate the two.
assert_not_infra_failure() {
    if [[ "${status}" -eq 124 ]]; then
        fail "TIMEOUT (infra) — VM no longer responding, not a product failure: ${output}"
    fi
}
