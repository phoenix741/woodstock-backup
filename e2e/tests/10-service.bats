#!/usr/bin/env bats
# §3 Service management.
#
# The client is checked for being *enabled*, not active: the agent refuses to
# start until its certificates exist, and those only appear after the
# enrollment performed in 30-certs.bats. That is the real user sequence.

load test_helper

# ─── Debian server (systemd) ─────────────────────────────────────────────────

@test "woodstock-api is active" {
    skip_unless_server debian
    run ssh_run server "systemctl is-active woodstock-api"
    assert_output "active"
}

@test "woodstock-api runs as the woodstock user" {
    skip_unless_server debian
    run ssh_run server "systemctl show woodstock-api -p User --value"
    assert_output "woodstock"
}

@test "woodstock-client-api is active" {
    skip_unless_server debian
    run ssh_run server "systemctl is-active woodstock-client-api"
    assert_output "active"
}

@test "woodstock-client-api runs as the woodstock user" {
    skip_unless_server debian
    run ssh_run server "systemctl show woodstock-client-api -p User --value"
    assert_output "woodstock"
}

@test "woodstock-worker is active" {
    skip_unless_server debian
    run ssh_run server "systemctl is-active woodstock-worker"
    assert_output "active"
}

@test "woodstock-worker runs as the woodstock user" {
    skip_unless_server debian
    run ssh_run server "systemctl show woodstock-worker -p User --value"
    assert_output "woodstock"
}

@test "woodstock-scheduler is active" {
    skip_unless_server debian
    run ssh_run server "systemctl is-active woodstock-scheduler"
    assert_output "active"
}

@test "woodstock-scheduler runs as the woodstock user" {
    skip_unless_server debian
    run ssh_run server "systemctl show woodstock-scheduler -p User --value"
    assert_output "woodstock"
}

# Regression: the units used to point at a documentation URL that 404s.
@test "woodstock-api documents the correct URL" {
    skip_unless_server debian
    run ssh_run server "systemctl show woodstock-api -p Documentation --value"
    assert_output "https://woodstock.shadoware.org/doc/"
}

@test "woodstock.target groups the 4 services" {
    skip_unless_server debian
    run ssh_run server "systemctl list-dependencies --plain woodstock.target | grep -c woodstock- | grep -qE '^[4-9]'"
    assert_success
}

@test "woodstock-api restarts cleanly" {
    skip_unless_server debian
    run ssh_run server "systemctl restart woodstock-api && sleep 3 && systemctl is-active --quiet woodstock-api"
    assert_success
}

@test "no errors on server services startup (debian)" {
    skip_unless_server debian
    run ssh_run server "journalctl -u woodstock-api -u woodstock-client-api -u woodstock-worker -u woodstock-scheduler --since '-10 min' -p err --no-pager -o cat"
    assert_output ""
}

# ─── Debian client (systemd) ─────────────────────────────────────────────────

@test "the unit is named woodstock-client (debian)" {
    skip_unless_client debian
    run ssh_run debian "systemctl is-enabled woodstock-client"
    assert_output "enabled"
}

# §3.1 — the agent needs root for btrfs snapshots, so the unit must not drop
# privileges.
@test "the agent runs as root, no User= in the unit (debian)" {
    skip_unless_client debian
    run ssh_run debian "systemctl show woodstock-client -p User --value"
    assert_output ""
}

@test "the debian agent is active" {
    skip_unless_client debian
    skip "before enrollment it has no certificates — verified in 30-certs"
}

# ─── FreeBSD client (rc.d) ────────────────────────────────────────────────────
# §3.2 of the test plan calls this the risky area, and it is: ws_client_daemon
# does not fork, so the rc script wraps it in daemon(8) with -P to get a real
# pidfile. Before that fix the script declared a pidfile nothing ever wrote,
# and `service status` could not find the process it had just started.

@test "the service is enabled in rc.conf (freebsd)" {
    skip_unless_client freebsd
    run ssh_run freebsd "sysrc -n woodstock_client_enable"
    assert_output "YES"
}

@test "the freebsd agent is active" {
    skip_unless_client freebsd
    skip "before enrollment it has no certificates — verified in 30-certs"
}

# ─── Windows client (SCM) ─────────────────────────────────────────────────────

@test "the service starts automatically (windows)" {
    skip_unless_client windows
    run ssh_run windows "(Get-Service woodstock_client_daemon).StartType"
    assert_output "Automatic"
}

# LocalSystem, like the root the Unix agents run as: the agent has to read
# arbitrary files and drive VSS.
@test "the service runs as LocalSystem (windows)" {
    skip_unless_client windows
    run ssh_run windows "(Get-CimInstance Win32_Service -Filter \"Name='woodstock_client_daemon'\").StartName"
    assert_output --partial "LocalSystem"
}

# install-service adds it (client-rs/src/winfirewall.rs); without it the
# server's gRPC connection is dropped before the agent ever sees it.
@test "the firewall rule for port 3657 exists (windows)" {
    skip_unless_client windows
    run ssh_run windows "if (-not (Get-NetFirewallRule | Where-Object DisplayName -match 'oodstock')) { exit 1 }"
    assert_success
}

@test "the windows agent is active" {
    skip_unless_client windows
    skip "before enrollment it has no certificates — verified in 30-certs"
}

# ─── FreeBSD server (rc.d) ───────────────────────────────────────────────────

@test "woodstock_api is enabled (freebsd)" {
    skip_unless_server freebsd
    run ssh_run server "sysrc -n woodstock_api_enable"
    assert_output "YES"
}

@test "woodstock_api runs and its pid is findable (freebsd)" {
    skip_unless_server freebsd
    run ssh_run server "service woodstock_api status"
    assert_success
}

@test "woodstock_client_api is enabled (freebsd)" {
    skip_unless_server freebsd
    run ssh_run server "sysrc -n woodstock_client_api_enable"
    assert_output "YES"
}

@test "woodstock_client_api runs and its pid is findable (freebsd)" {
    skip_unless_server freebsd
    run ssh_run server "service woodstock_client_api status"
    assert_success
}

@test "woodstock_worker is enabled (freebsd)" {
    skip_unless_server freebsd
    run ssh_run server "sysrc -n woodstock_worker_enable"
    assert_output "YES"
}

@test "woodstock_worker runs and its pid is findable (freebsd)" {
    skip_unless_server freebsd
    run ssh_run server "service woodstock_worker status"
    assert_success
}

@test "woodstock_scheduler is enabled (freebsd)" {
    skip_unless_server freebsd
    run ssh_run server "sysrc -n woodstock_scheduler_enable"
    assert_output "YES"
}

@test "woodstock_scheduler runs and its pid is findable (freebsd)" {
    skip_unless_server freebsd
    run ssh_run server "service woodstock_scheduler status"
    assert_success
}

@test "the rc.d logs are populated (freebsd)" {
    skip_unless_server freebsd
    run ssh_run server "test -s /var/log/woodstock/api.log || test -s /var/log/woodstock/worker.log"
    assert_success
}

# A stop must actually kill the process, not leave it orphaned — which is what
# would happen if the pidfile were wrong.
#
# The bracket around the first character stops pgrep from matching the shell
# sshd started to run this line: it is a three-command sequence, so that shell
# is necessarily still alive — with the pattern in its own argv — when pgrep
# runs, and the assertion could never pass.
@test "woodstock_api actually stops (freebsd)" {
    skip_unless_server freebsd
    run ssh_run server "service woodstock_api stop >/dev/null 2>&1; sleep 2; ! pgrep -f '[/]usr/local/bin/api_server' >/dev/null"
    assert_success
}

@test "woodstock_api restarts cleanly (freebsd)" {
    skip_unless_server freebsd
    run ssh_run server "service woodstock_api start >/dev/null 2>&1; sleep 5; service woodstock_api status"
    assert_success
}
