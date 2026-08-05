#!/usr/bin/env bats
# §1 Installation and §2 post-install checks of packaging-test-plan.md.
#
# Runs after provisioning, which already installed the packages the way a user
# would (apt install ./*.deb, letting apt resolve the declared dependencies).
# This file asserts on the result.

load test_helper

# ─── Debian server ───────────────────────────────────────────────────────────

@test "woodstock-server is installed" {
    skip_unless_server debian
    run ssh_run server "dpkg-query -W -f='\${Status}' woodstock-server"
    assert_output --partial "install ok installed"
}

@test "woodstock-cli is installed (ws_console/ws_restore)" {
    skip_unless_server debian
    run ssh_run server "dpkg-query -W -f='\${Status}' woodstock-cli"
    assert_output --partial "install ok installed"
}

# Regression: `authors` without an email in Cargo.toml produced a malformed
# Maintainer field (audit of 2026-08-01).
@test "Server maintainer is well-formed" {
    skip_unless_server debian
    run ssh_run server "dpkg-query -W -f='\${Maintainer}' woodstock-server"
    assert_output --partial "<"
}

# The dependency has to be satisfiable by name: `valkey` is a source package
# only, so the old declaration always fell through to redis-server.
@test "valkey-server dependency declared" {
    skip_unless_server debian
    run ssh_run server "dpkg-query -W -f='\${Depends}' woodstock-server"
    assert_output --partial "valkey-server"
}

@test "a key-value server is properly installed (debian)" {
    skip_unless_server debian
    run ssh_run server "dpkg-query -W -f='\${Status}' valkey-server 2>/dev/null | grep -q 'ok installed' || dpkg-query -W -f='\${Status}' redis-server | grep -q 'ok installed'"
    assert_success
}

@test "the server package delivers /usr/bin/api_server" {
    skip_unless_server debian
    run ssh_run server "dpkg -L woodstock-server | grep -qx /usr/bin/api_server"
    assert_success
}

@test "the server package delivers /usr/bin/client_api_server" {
    skip_unless_server debian
    run ssh_run server "dpkg -L woodstock-server | grep -qx /usr/bin/client_api_server"
    assert_success
}

@test "the server package delivers /usr/bin/job_worker" {
    skip_unless_server debian
    run ssh_run server "dpkg -L woodstock-server | grep -qx /usr/bin/job_worker"
    assert_success
}

@test "the server package delivers /usr/bin/scheduler" {
    skip_unless_server debian
    run ssh_run server "dpkg -L woodstock-server | grep -qx /usr/bin/scheduler"
    assert_success
}

@test "the CLI package delivers /usr/bin/ws_console" {
    skip_unless_server debian
    run ssh_run server "dpkg -L woodstock-cli | grep -qx /usr/bin/ws_console"
    assert_success
}

@test "the CLI package delivers /usr/bin/ws_restore" {
    skip_unless_server debian
    run ssh_run server "dpkg -L woodstock-cli | grep -qx /usr/bin/ws_restore"
    assert_success
}

@test "the CLI package delivers /usr/bin/ws_sync" {
    skip_unless_server debian
    run ssh_run server "dpkg -L woodstock-cli | grep -qx /usr/bin/ws_sync"
    assert_success
}

@test "the frontend is installed under /usr/share/woodstock/static" {
    skip_unless_server debian
    run ssh_run server "test -d /usr/share/woodstock/static && test -n \"\$(ls -A /usr/share/woodstock/static)\""
    assert_success
}

# §2.1 — the server must not run as root.
@test "the woodstock system user exists without a login shell (debian)" {
    skip_unless_server debian
    run ssh_run server "getent passwd woodstock"
    assert_output --partial "nologin"
}

@test "/var/lib/woodstock is owned by woodstock:woodstock with 750 permissions" {
    skip_unless_server debian
    run ssh_run server "stat -c '%U %G %a' /var/lib/woodstock"
    assert_output "woodstock woodstock 750"
}

@test "/etc/woodstock/server.env is root:woodstock with 640 permissions" {
    skip_unless_server debian
    run ssh_run server "stat -c '%U %G %a' /etc/woodstock/server.env"
    assert_output "root woodstock 640"
}

# ─── Debian client ───────────────────────────────────────────────────────────

@test "woodstock-client is installed (debian)" {
    skip_unless_client debian
    run ssh_run debian "dpkg-query -W -f='\${Status}' woodstock-client"
    assert_output --partial "install ok installed"
}

@test "the client package delivers /usr/bin/ws_client_daemon (debian)" {
    skip_unless_client debian
    run ssh_run debian "dpkg -L woodstock-client | grep -qx /usr/bin/ws_client_daemon"
    assert_success
}

@test "config.yaml is declared as conffile (debian)" {
    skip_unless_client debian
    run ssh_run debian "grep -qx /etc/woodstock/config.yaml /var/lib/dpkg/info/woodstock-client.conffiles"
    assert_success
}

# §2.2 — since 2026-08-01 the agent runs as root and creates no user.
@test "the client package creates no woodstock user (debian)" {
    skip_unless_client debian
    run ssh_run debian "! getent passwd woodstock"
    assert_success
}

@test "/etc/woodstock/config.yaml is root:root with 600 permissions (debian)" {
    skip_unless_client debian
    run ssh_run debian "stat -c '%U %G %a' /etc/woodstock/config.yaml"
    assert_output "root root 600"
}

# Renamed from woodstock-client-rs; the transition fields must be there so an
# existing 2.0.0 install upgrades instead of conflicting.
@test "the package replaces the old name woodstock-client-rs" {
    skip_unless_client debian
    run ssh_run debian "dpkg-query -W -f='\${Replaces}' woodstock-client"
    assert_output --partial "woodstock-client-rs"
}

# ─── FreeBSD client ──────────────────────────────────────────────────────────

# `pkg add` refused the package outright if the declared ABI did not match the
# running system — the packages used to claim freebsd:14 while being built on
# 15. Reaching this point at all means the ABI is right.
@test "woodstock-client is installed without ABI override (freebsd)" {
    skip_unless_client freebsd
    run ssh_run freebsd "pkg info -e woodstock-client"
    assert_success
}

# Compare what the package declares against what the system runs, rather than
# the system against itself.
@test "the declared ABI matches the system (freebsd)" {
    skip_unless_client freebsd
    local pkg_abi sys_abi
    pkg_abi="$(ssh_run freebsd "pkg query '%q' woodstock-client" 2>/dev/null || true)"
    sys_abi="$(ssh_run freebsd "pkg config ABI" 2>/dev/null || true)"
    [[ -n "${pkg_abi}" ]]
    [[ "${pkg_abi,,}" == "${sys_abi,,}" ]]
}

@test "FreeBSD package Maintainer" {
    skip_unless_client freebsd
    run ssh_run freebsd "pkg info woodstock-client"
    assert_output --partial "ulrich.vdh@gmail.com"
}

@test "FreeBSD package WWW" {
    skip_unless_client freebsd
    run ssh_run freebsd "pkg info woodstock-client"
    assert_output --partial "woodstock.shadoware.org"
}

@test "the client package delivers /usr/local/bin/ws_client_daemon (freebsd)" {
    skip_unless_client freebsd
    run ssh_run freebsd "pkg info -l woodstock-client | grep -q '/usr/local/bin/ws_client_daemon'"
    assert_success
}

@test "the client package delivers /usr/local/etc/rc.d/woodstock_client (freebsd)" {
    skip_unless_client freebsd
    run ssh_run freebsd "pkg info -l woodstock-client | grep -q '/usr/local/etc/rc.d/woodstock_client'"
    assert_success
}

@test "the client package delivers /usr/local/etc/woodstock/config.yaml.sample (freebsd)" {
    skip_unless_client freebsd
    run ssh_run freebsd "pkg info -l woodstock-client | grep -q '/usr/local/etc/woodstock/config.yaml.sample'"
    assert_success
}

# §2.2 — since 2026-08-01 the agent runs as root and creates no user.
@test "the client package creates no woodstock user (freebsd)" {
    skip_unless_client freebsd
    run ssh_run freebsd "! pw usershow woodstock >/dev/null 2>&1"
    assert_success
}

# post-install.sh is supposed to copy the sample into place on first install.
# It never ran: `pkg create --manifest` ignores +POST_INSTALL files dropped
# next to the manifest, so the packages shipped no lifecycle scripts at all —
# which also means `pkg delete` never stopped the services.
@test "the package includes its lifecycle scripts (freebsd)" {
    skip_unless_client freebsd
    run ssh_run freebsd "pkg info -D woodstock-client 2>/dev/null | grep -qi 'woodstock'"
    assert_success
}

@test "config.yaml is created by post-install as root:wheel 600 (freebsd)" {
    skip_unless_client freebsd
    run ssh_run freebsd "stat -f '%Su %Sg %Lp' /usr/local/etc/woodstock/config.yaml"
    assert_output "root wheel 600"
}

# ─── Windows client ──────────────────────────────────────────────────────────
# There is no Windows package: the CI ships a bare ws_client_daemon.exe and the
# agent registers itself through its own `install-service` subcommand. So the
# assertions are about what provisioning put in place, not about a package
# database.

@test "the agent is deployed under Program Files (windows)" {
    skip_unless_client windows
    run ssh_run windows "if (-not (Test-Path 'C:\\Program Files\\Woodstock\\ws_client_daemon.exe')) { exit 1 }"
    assert_success
}

@test "the administration console is deployed (windows)" {
    skip_unless_client windows
    run ssh_run windows "if (-not (Test-Path 'C:\\Program Files\\Woodstock\\ws_client_console.exe')) { exit 1 }"
    assert_success
}

@test "config.yaml exists before enrollment (windows)" {
    skip_unless_client windows
    run ssh_run windows "if (-not (Test-Path 'C:\\ProgramData\\woodstock\\config.yaml')) { exit 1 }"
    assert_success
}

# install-service bakes --config-dir into the registered command line
# (client-rs/src/winserv.rs). Without it the service would run as LocalSystem
# and resolve %APPDATA%\woodstock under systemprofile, where nothing the
# harness wrote is visible.
@test "the service points to the correct configuration directory (windows)" {
    skip_unless_client windows
    run ssh_run windows "(Get-CimInstance Win32_Service -Filter \"Name='woodstock_client_daemon'\").PathName"
    assert_output --partial "ProgramData"
}

# ─── FreeBSD server ──────────────────────────────────────────────────────────

@test "woodstock-server is installed (freebsd)" {
    skip_unless_server freebsd
    run ssh_run server "pkg info -e woodstock-server"
    assert_success
}

@test "the valkey dependency is resolved automatically (freebsd)" {
    skip_unless_server freebsd
    run ssh_run server "pkg info -e valkey"
    assert_success
}

@test "the server package delivers /usr/local/bin/api_server (freebsd)" {
    skip_unless_server freebsd
    run ssh_run server "pkg info -l woodstock-server | grep -q '/usr/local/bin/api_server$'"
    assert_success
}

@test "the server package delivers /usr/local/bin/client_api_server (freebsd)" {
    skip_unless_server freebsd
    run ssh_run server "pkg info -l woodstock-server | grep -q '/usr/local/bin/client_api_server$'"
    assert_success
}

@test "the server package delivers /usr/local/bin/job_worker (freebsd)" {
    skip_unless_server freebsd
    run ssh_run server "pkg info -l woodstock-server | grep -q '/usr/local/bin/job_worker$'"
    assert_success
}

@test "the server package delivers /usr/local/bin/scheduler (freebsd)" {
    skip_unless_server freebsd
    run ssh_run server "pkg info -l woodstock-server | grep -q '/usr/local/bin/scheduler$'"
    assert_success
}

# These used to be missing entirely: the FreeBSD server package shipped no
# admin CLI, making manifest inspection and restore impossible.
@test "the server package delivers /usr/local/bin/ws_console (freebsd)" {
    skip_unless_server freebsd
    run ssh_run server "pkg info -l woodstock-server | grep -q '/usr/local/bin/ws_console$'"
    assert_success
}

@test "the server package delivers /usr/local/bin/ws_restore (freebsd)" {
    skip_unless_server freebsd
    run ssh_run server "pkg info -l woodstock-server | grep -q '/usr/local/bin/ws_restore$'"
    assert_success
}

@test "the server package delivers /usr/local/bin/ws_sync (freebsd)" {
    skip_unless_server freebsd
    run ssh_run server "pkg info -l woodstock-server | grep -q '/usr/local/bin/ws_sync$'"
    assert_success
}

@test "the woodstock system user exists (freebsd)" {
    skip_unless_server freebsd
    run ssh_run server "pw usershow woodstock"
    assert_output --partial "woodstock"
}

@test "/var/db/woodstock is owned by woodstock:woodstock with 750 permissions (freebsd)" {
    skip_unless_server freebsd
    run ssh_run server "stat -f '%Su %Sg %Lp' /var/db/woodstock"
    assert_output "woodstock woodstock 750"
}
