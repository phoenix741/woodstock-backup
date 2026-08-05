#!/bin/bash
set -e
cd /workspace

echo "=== Installing system dependencies ==="
apt-get update -qq && apt-get install -y -qq protobuf-compiler cmake make build-essential libacl1-dev libfuse-dev 2>/dev/null

echo "=== Installing Rust ==="
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y -q 2>/dev/null
source ~/.cargo/env

echo "=== Installing cargo-deb ==="
cargo install cargo-deb -q 2>/dev/null
echo "cargo-deb $(cargo deb --version) ready"

echo ""
echo "=== Creating fake post-build artifacts ==="
mkdir -p target/x86_64-unknown-linux-gnu/release
mkdir -p target/release
for bin in api_server client_api_server job_worker scheduler ws_client_daemon ws_client_console ws_console ws_restore ws_sync ws_backuppc_importer; do
  # Create a minimal ELF-like file (empty shell script acts as placeholder)
  printf '#!/bin/sh\n' > target/x86_64-unknown-linux-gnu/release/$bin
  chmod +x target/x86_64-unknown-linux-gnu/release/$bin
  cp target/x86_64-unknown-linux-gnu/release/$bin target/release/$bin
done

echo "=== Creating fake server-rs/front-dist (simulates CI artifact download directly into server-rs/front-dist/) ==="
mkdir -p server-rs/front-dist/assets
echo '<!DOCTYPE html><html><head><title>Woodstock</title></head><body></body></html>' > server-rs/front-dist/index.html
echo 'console.log("woodstock app")' > server-rs/front-dist/assets/main.js
echo '.app { color: red; }' > server-rs/front-dist/assets/style.css

PASS=0
FAIL=0

run_test() {
  local pkg=$1
  local exit_code=0
  echo ""
  echo "=== Testing: $pkg ==="
  cargo deb -p "$pkg" --no-build --no-strip --target x86_64-unknown-linux-gnu 2>&1 | grep -v "DEBUG\|Looking for"
  exit_code=${PIPESTATUS[0]}
  if [ $exit_code -eq 0 ]; then
    echo "RESULT: PASS - $pkg"
    PASS=$((PASS + 1))
  else
    echo "RESULT: FAIL - $pkg (exit code $exit_code)"
    FAIL=$((FAIL + 1))
  fi
}

run_test "woodstock-server-rs"
run_test "woodstock-client-rs"
run_test "woodstock-cli-rs"
run_test "ws_backuppc_importer"

echo ""
echo "=== Generated .deb files ==="
find target -name "*.deb" 2>/dev/null | sort

echo ""
echo "=== SUMMARY: $PASS passed, $FAIL failed ==="

# Cleanup
rm -rf server-rs/front-dist
# Remove fake binaries
for bin in api_server client_api_server job_worker scheduler ws_client_daemon ws_client_console ws_console ws_restore ws_sync ws_backuppc_importer; do
  rm -f target/x86_64-unknown-linux-gnu/release/$bin target/release/$bin
done

[ $FAIL -eq 0 ] || exit 1
