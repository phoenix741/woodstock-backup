#!/usr/bin/env bats
# §5 Package upgrade — does a reinstall preserve local configuration?
#
# The mechanism being tested: dpkg only preserves local edits to files declared
# as conffiles; an ordinary file is overwritten on unpack. So the discriminating
# move is to *modify* /etc/woodstock/config.yaml and reinstall — with the
# conf-files declaration correct the edit survives, with the bug it used to have
# (pointing at config.yml, a file that never existed) it is silently lost.
#
# Asserting on a locally modified file matters: checking that the file merely
# still exists would pass either way.
#
# FreeBSD has no conffile concept; post-install.sh only copies the sample when
# config.yaml is absent, which is the equivalent guarantee.
#
# The reinstall itself is the shared precondition for every check below, not
# an assertion in its own right, so it runs once per client in setup_file.
# Windows has no package, so it is skipped entirely, like the original script.

load test_helper

_reinstall() {
    local client="$1"
    [[ "${client}" == "windows" ]] && return 0

    local cfg marker
    cfg="$(client_config_dir "${client}")/config.yaml"
    marker="e2e-upgrade-marker-$(date +%s%N)-${client}"
    state_set upgrade_marker "${client}" "${marker}"
    ssh_run "${client}" "printf '%s\n' '# ${marker}' >> ${cfg}" >/dev/null

    local pkg
    case "${client}" in
        debian)
            pkg="$(ssh_run debian "ls /tmp/packages/deb/woodstock-client_*.deb" 2>/dev/null | head -1)"
            state_set pkg_available "${client}" "$([[ -n "${pkg}" ]] && echo 1 || echo 0)"
            [[ -n "${pkg}" ]] || return 0
            if ssh_run debian "DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --reinstall '${pkg}'" \
                 > "${RUN_DIR}/upgrade-${client}.log" 2>&1; then
                state_set reinstall_ok "${client}" 1
            else
                state_set reinstall_ok "${client}" 0
                return 0
            fi
            ;;
        freebsd)
            pkg="$(ssh_run freebsd "ls /tmp/packages/pkg/woodstock-client-*.pkg" 2>/dev/null | head -1)"
            state_set pkg_available "${client}" "$([[ -n "${pkg}" ]] && echo 1 || echo 0)"
            [[ -n "${pkg}" ]] || return 0
            if ssh_run freebsd "env ASSUME_ALWAYS_YES=yes pkg add -f '${pkg}'" \
                 > "${RUN_DIR}/upgrade-${client}.log" 2>&1; then
                state_set reinstall_ok "${client}" 1
            else
                state_set reinstall_ok "${client}" 0
                return 0
            fi
            ;;
    esac

    # Leave the agent as the rest of the suite expects to find it.
    ssh_run "${client}" "$(client_restart_cmd "${client}")" >/dev/null 2>&1 || true
}

setup_file() {
    local client
    for client in ${CLIENTS}; do
        [[ "${client}" == "windows" ]] && continue
        _reinstall "${client}"
    done
}

@test "the client package is available for reinstall (debian)" {
    skip_unless_client debian
    [[ "$(state_get pkg_available debian 0)" == "1" ]]
}

@test "reinstall of debian client package" {
    skip_unless_client debian
    [[ "$(state_get reinstall_ok debian 0)" == "1" ]]
}

@test "modified configuration survives reinstall (debian)" {
    skip_unless_client debian
    [[ "$(state_get reinstall_ok debian 0)" == "1" ]] || skip "réinstallation non aboutie"
    run ssh_run debian "grep -qF '$(state_get upgrade_marker debian)' $(client_config_dir debian)/config.yaml"
    assert_success
}

# The certificates deployed at enrollment live in the same directory and are
# not owned by the package; losing them would break every later suite.
@test "the agent certificates are preserved (debian)" {
    skip_unless_client debian
    [[ "$(state_get reinstall_ok debian 0)" == "1" ]] || skip "réinstallation non aboutie"
    run ssh_run debian "test -s $(client_config_dir debian)/rootCA.pem"
    assert_success
}

@test "the agent restarts after reinstall (debian)" {
    skip_unless_client debian
    [[ "$(state_get reinstall_ok debian 0)" == "1" ]] || skip "réinstallation non aboutie"
    run retry 60 3 ssh_run debian "$(client_status_cmd debian)"
    assert_not_infra_failure
    assert_success
}

@test "the client package is available for reinstall (freebsd)" {
    skip_unless_client freebsd
    [[ "$(state_get pkg_available freebsd 0)" == "1" ]]
}

@test "reinstall of freebsd client package" {
    skip_unless_client freebsd
    [[ "$(state_get reinstall_ok freebsd 0)" == "1" ]]
}

@test "modified configuration survives reinstall (freebsd)" {
    skip_unless_client freebsd
    [[ "$(state_get reinstall_ok freebsd 0)" == "1" ]] || skip "réinstallation non aboutie"
    run ssh_run freebsd "grep -qF '$(state_get upgrade_marker freebsd)' $(client_config_dir freebsd)/config.yaml"
    assert_success
}

@test "the agent certificates are preserved (freebsd)" {
    skip_unless_client freebsd
    [[ "$(state_get reinstall_ok freebsd 0)" == "1" ]] || skip "réinstallation non aboutie"
    run ssh_run freebsd "test -s $(client_config_dir freebsd)/rootCA.pem"
    assert_success
}

@test "the agent restarts after reinstall (freebsd)" {
    skip_unless_client freebsd
    [[ "$(state_get reinstall_ok freebsd 0)" == "1" ]] || skip "réinstallation non aboutie"
    run retry 60 3 ssh_run freebsd "$(client_status_cmd freebsd)"
    assert_not_infra_failure
    assert_success
}

# The third §5 case — dpkg prompting when the shipped default changed too —
# needs a second .deb built locally with a bumped version and a different
# default config. Out of scope here: the harness installs the CI artifacts.
@test "dpkg merge prompt if delivered default also changes" {
    skip "exige un second paquet construit localement, non couvert par les artefacts CI"
}
