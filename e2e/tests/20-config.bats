#!/usr/bin/env bats
# §4 Configuration changes take effect.
#
# Every mutation of hosts.yml / <host>.yml has to be followed by
# POST /api/server/cache/clear: the configuration is cached in Redis for 24h
# (woodstock-rs/src/config/hosts.rs), so without it the API keeps answering
# from the stale copy and the assertion fails for the wrong reason.
#
# Paths and init system differ per platform; the assertions themselves do not.
# `sed -i` takes no argument on GNU but requires one on BSD, so the edits go
# through a temporary file instead — portable, and it keeps the original mode.

load test_helper

setup_file() {
    # load test_helper (top of this file) already ran in this process, so
    # lib/*.sh and load_config are available without re-sourcing.
    local conf_dir
    conf_dir="$(server_backup_path)/config"

    # -a so the restore below puts back the original owner and mode: the API
    # runs as `woodstock` and silently serves an empty host list if it cannot
    # read hosts.yml, which would make every later test fail for the wrong
    # reason.
    ssh_script server <<EOF >/dev/null 2>&1 || true
cp -a ${conf_dir}/hosts.yml /tmp/hosts.yml.bak
echo "- e2e-scratch" >> ${conf_dir}/hosts.yml
cat > ${conf_dir}/e2e-scratch.yml <<'YAML'
password: scratch
operations:
  operation:
    shares:
      - name: /tmp
YAML
chown woodstock:woodstock ${conf_dir}/e2e-scratch.yml
curl -fsS -X POST http://127.0.0.1:3000/api/server/cache/clear
EOF
}

teardown_file() {
    # Safety net: the tests below already restore hosts.yml and the API port
    # in sequence, but if one of them was interrupted this still leaves the
    # server in its original state for whatever runs next.
    local env_file
    env_file="$(server_config_dir)/server.env"

    local conf_dir
    conf_dir="$(server_backup_path)/config"
    ssh_script server <<EOF >/dev/null 2>&1 || true
cp -a /tmp/hosts.yml.bak ${conf_dir}/hosts.yml
rm -f /tmp/hosts.yml.bak ${conf_dir}/e2e-scratch.yml
curl -fsS -X POST http://127.0.0.1:3000/api/server/cache/clear
EOF

    ssh_script server <<EOF >/dev/null 2>&1 || true
grep -v '^MANAGEMENT_API_PORT=' ${env_file} > /tmp/server.env.new
cat /tmp/server.env.new > ${env_file}
rm -f /tmp/server.env.new
$(server_restart_cmd woodstock-api)
sleep 5
EOF
}

@test "a host added to hosts.yml appears after cache clear" {
    run curl -fsS "${API}/hosts"
    assert_output --partial "e2e-scratch"
}

@test "the removed host disappears from the API" {
    CONF_DIR="$(server_backup_path)/config"
    ssh_script server <<EOF >/dev/null 2>&1 || true
cp -a /tmp/hosts.yml.bak ${CONF_DIR}/hosts.yml
rm -f /tmp/hosts.yml.bak ${CONF_DIR}/e2e-scratch.yml
curl -fsS -X POST http://127.0.0.1:3000/api/server/cache/clear
EOF
    run bash -c "! curl -fsS '${API}/hosts' | grep -q e2e-scratch"
    assert_success
}

# Guard against the restore above having broken the file: an unreadable
# hosts.yml also makes e2e-scratch "disappear".
@test "declared hosts remain visible after restore" {
    run bash -c "curl -fsS '${API}/hosts' | jq -e 'length > 0' >/dev/null"
    assert_success
}

# MANAGEMENT_API_PORT drives the listening port; changing it must be honoured.
@test "MANAGEMENT_API_PORT=3010 is taken into account" {
    ENV_FILE="$(server_config_dir)/server.env"
    ssh_script server <<EOF >/dev/null 2>&1 || true
grep -v '^MANAGEMENT_API_PORT=' ${ENV_FILE} > /tmp/server.env.new
echo 'MANAGEMENT_API_PORT=3010' >> /tmp/server.env.new
cat /tmp/server.env.new > ${ENV_FILE}
$(server_restart_cmd woodstock-api)
sleep 5
EOF
    run ssh_run server "curl -fsS http://127.0.0.1:3010/api/hosts >/dev/null"
    assert_success
}

@test "returning to the default port is effective" {
    ENV_FILE="$(server_config_dir)/server.env"
    ssh_script server <<EOF >/dev/null 2>&1 || true
grep -v '^MANAGEMENT_API_PORT=' ${ENV_FILE} > /tmp/server.env.new
cat /tmp/server.env.new > ${ENV_FILE}
rm -f /tmp/server.env.new
$(server_restart_cmd woodstock-api)
sleep 5
EOF
    run retry 60 3 curl -fsS -o /dev/null "${API}/hosts"
    assert_not_infra_failure
    assert_success
}

@test "the Debian client supports acl and xattr without warning" {
    skip_unless_client debian
    run ssh_run debian "! journalctl -u woodstock-client --since '-30 min' --no-pager 2>/dev/null | grep -iE 'not supported on this platform'"
    assert_success
}
