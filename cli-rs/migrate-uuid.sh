#!/usr/bin/env bash
# migrate-uuid.sh — Backfill `id` fields in all hosts' backup.yml files,
# then rename backup directories from `<number>/` to `<uuid>/`.
#
# Pass 1: Iterates over every host directory in HOSTS_PATH and assigns a UUID v7
# (or v4 on Python < 3.12) to each backup entry that is still missing its
# `id` field.  Backups that already have an id are left unchanged.
#
# Pass 2: Renames each backup directory from `hosts/<hostname>/<number>/` to
# `hosts/<hostname>/<uuid>/` using the id assigned in Pass 1.
# Skipped if the target directory already exists or the source is missing.
#
# Usage:
#   ./scripts/migrate-uuid.sh [OPTIONS] [HOSTS_PATH]
#   HOSTS_PATH=/var/lib/woodstock/hosts ./scripts/migrate-uuid.sh [OPTIONS]
#
# Options:
#   --dry-run    Print planned changes without writing anything.
#   --no-rename  Skip Pass 2 (directory renaming).
#   --help       Show this help message.
#
# Arguments:
#   HOSTS_PATH  Path to the hosts directory (overrides the env variable).
#               Default: /var/lib/woodstock/hosts
#
# Dependencies:
#   python3  with PyYAML (pip install pyyaml)
#
# Exit codes:
#   0  Success (or dry-run completed)
#   1  Configuration or dependency error

set -euo pipefail

# ── defaults ────────────────────────────────────────────────────────────────
DRY_RUN=false
NO_RENAME=false
HOSTS_PATH="${HOSTS_PATH:-/var/lib/woodstock/hosts}"

# ── argument parsing ─────────────────────────────────────────────────────────
for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=true ;;
    --no-rename) NO_RENAME=true ;;
    --help)
      sed -n '/^# /s/^# //p' "$0"
      exit 0
      ;;
    -*)
      echo "ERROR: unknown option: $arg" >&2
      echo "       Run '$0 --help' for usage." >&2
      exit 1
      ;;
    *)
      HOSTS_PATH="$arg"
      ;;
  esac
done

# ── pre-flight checks ────────────────────────────────────────────────────────
if [[ ! -d "$HOSTS_PATH" ]]; then
  echo "ERROR: hosts directory not found: $HOSTS_PATH" >&2
  exit 1
fi

if ! command -v python3 &>/dev/null; then
  echo "ERROR: python3 is required but not found in PATH" >&2
  exit 1
fi

if ! python3 -c "import yaml" 2>/dev/null; then
  echo "ERROR: PyYAML is required (pip install pyyaml)" >&2
  exit 1
fi

# ── Python helper (per-host) ─────────────────────────────────────────────────
# Reads backup.yml, assigns UUIDs to entries that are missing one, writes back.
# Prints one line per assigned UUID (or [dry-run] prefix when applicable).
# Last line is always: __ASSIGNED__=<count>
read -r -d '' PYTHON_SCRIPT << 'ENDPY' || true
import sys
import os
import time
import uuid
import yaml
from datetime import datetime, timezone


def gen_uuid_v7(ts_ms: int) -> str:
    """
    Build a UUID v7 from an explicit millisecond timestamp.

    Layout (RFC 9562):
      0                   1                   2                   3
      0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
     +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
     |                        unix_ts_ms [47:16]                     |
     +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
     |  unix_ts_ms [15:0]  | ver=7 |        rand_a [11:0]           |
     +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
     |var=10|              rand_b [61:0]                             |
     +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    """
    rand = os.urandom(10)
    b = bytearray(16)
    # 48-bit timestamp
    b[0] = (ts_ms >> 40) & 0xFF
    b[1] = (ts_ms >> 32) & 0xFF
    b[2] = (ts_ms >> 24) & 0xFF
    b[3] = (ts_ms >> 16) & 0xFF
    b[4] = (ts_ms >>  8) & 0xFF
    b[5] =  ts_ms        & 0xFF
    # version 7 in the high nibble of byte 6
    b[6] = 0x70 | (rand[0] & 0x0F)
    b[7] = rand[1]
    # variant 10xxxxxx in byte 8
    b[8] = 0x80 | (rand[2] & 0x3F)
    b[9:16] = rand[3:10]
    return str(uuid.UUID(bytes=bytes(b)))


def ts_ms_from_backup(backup: dict, index: int) -> int:
    """
    Extract a millisecond timestamp from the backup's start_date.
    Falls back to (current time + index*1ms) so UUIDs remain unique and ordered.
    index is used as a tie-breaker when multiple backups share the same start_date.
    """
    start_date = backup.get("start_date")
    base_ms: int | None = None

    if isinstance(start_date, datetime):
        dt = start_date if start_date.tzinfo else start_date.replace(tzinfo=timezone.utc)
        base_ms = int(dt.timestamp() * 1000)
    elif start_date is not None:
        for fmt in (
            "%Y-%m-%dT%H:%M:%S%z",
            "%Y-%m-%dT%H:%M:%S",
            "%Y-%m-%d %H:%M:%S%z",
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d",
        ):
            try:
                dt = datetime.strptime(str(start_date), fmt)
                if dt.tzinfo is None:
                    dt = dt.replace(tzinfo=timezone.utc)
                base_ms = int(dt.timestamp() * 1000)
                break
            except ValueError:
                continue

    if base_ms is None:
        base_ms = int(time.time() * 1000) + index

    # Add index as sub-millisecond tie-breaker (stays within the ms bucket)
    return base_ms + index


backup_file = sys.argv[1]
dry_run     = sys.argv[2] == "1"
hostname    = sys.argv[3]

with open(backup_file, "r", encoding="utf-8") as fh:
    backups = yaml.safe_load(fh)

if not isinstance(backups, list):
    backups = []

# Sort by sequential number before assigning UUIDs so that
# the time-ordered UUIDs reflect the backup order.
backups.sort(key=lambda b: b.get("number", 0) if isinstance(b, dict) else 0)

assigned = 0
for index, backup in enumerate(backups):
    if not isinstance(backup, dict):
        continue
    if backup.get("id"):
        continue                          # already has an id, skip

    ts_ms      = ts_ms_from_backup(backup, index)
    new_uuid   = gen_uuid_v7(ts_ms)
    number     = backup.get("number",     "?")
    start_date = backup.get("start_date", "?")

    if dry_run:
        print(f"[dry-run] {hostname} #{number} ({start_date}) \u2192 id {new_uuid}")
    else:
        backup["id"] = new_uuid
        print(f"{hostname} #{number} ({start_date}): assigning id {new_uuid}")

    assigned += 1

if not dry_run and assigned > 0:
    with open(backup_file, "w", encoding="utf-8") as fh:
        yaml.dump(backups, fh, default_flow_style=False, allow_unicode=True)

print(f"__ASSIGNED__={assigned}")
ENDPY

# ── main loop ────────────────────────────────────────────────────────────────
total_hosts=0
total_assigned=0

if [[ "$DRY_RUN" == "true" ]]; then
  echo "[dry-run] scanning $HOSTS_PATH"
else
  echo "Scanning $HOSTS_PATH"
fi

for host_dir in "$HOSTS_PATH"/*/; do
  [[ -d "$host_dir" ]] || continue
  backup_file="${host_dir}backup.yml"
  [[ -f "$backup_file" ]] || continue

  hostname="$(basename "$host_dir")"
  total_hosts=$(( total_hosts + 1 ))

  dry_flag="$([[ "$DRY_RUN" == "true" ]] && echo "1" || echo "0")"

  # Run the Python helper; capture its stdout
  output="$(echo "$PYTHON_SCRIPT" | python3 - "$backup_file" "$dry_flag" "$hostname")"

  # Extract the __ASSIGNED__ sentinel line
  assigned="$(echo "$output" | grep '^__ASSIGNED__=' | cut -d= -f2)"
  # Print human-readable lines (all except the sentinel)
  echo "$output" | grep -v '^__ASSIGNED__=' || true

  total_assigned=$(( total_assigned + ${assigned:-0} ))
done

# ── summary ──────────────────────────────────────────────────────────────────
echo ""
if [[ "$DRY_RUN" == "true" ]]; then
  echo "[dry-run] would assign ${total_assigned} UUID(s) across ${total_hosts} host(s) — no files written"
else
  echo "Done. Assigned ${total_assigned} UUID(s) across ${total_hosts} host(s)"
fi

# ── pass 2 : rename dirs <number>/ → <uuid>/ ─────────────────────────────────
if [[ "$NO_RENAME" == "true" ]]; then
  echo "Pass 2 skipped (--no-rename)"
  exit 0
fi

echo ""
if [[ "$DRY_RUN" == "true" ]]; then
  echo "[dry-run] Pass 2: scanning for directories to rename in $HOSTS_PATH"
else
  echo "Pass 2: renaming backup directories <number>/ → <uuid>/"
fi

# Python helper: emit one "<number> <uuid>" line per backup entry in backup.yml
read -r -d '' RENAME_SCRIPT << 'ENDPY2' || true
import sys
import yaml

backup_file = sys.argv[1]

try:
    with open(backup_file, "r", encoding="utf-8") as fh:
        backups = yaml.safe_load(fh)
except Exception:
    sys.exit(0)

if not isinstance(backups, list):
    sys.exit(0)

for backup in backups:
    if not isinstance(backup, dict):
        continue
    number = backup.get("number")
    uuid   = backup.get("id")
    if number is not None and uuid:
        print(f"{number} {uuid}")
ENDPY2

total_renamed=0
total_skipped=0

for host_dir in "$HOSTS_PATH"/*/; do
  [[ -d "$host_dir" ]] || continue
  backup_file="${host_dir}backup.yml"
  [[ -f "$backup_file" ]] || continue

  hostname="$(basename "$host_dir")"

  pairs="$(echo "$RENAME_SCRIPT" | python3 - "$backup_file")"
  [[ -z "$pairs" ]] && continue

  while IFS=' ' read -r number uuid; do
    [[ -z "$number" || -z "$uuid" ]] && continue
    old_dir="${host_dir}${number}"
    new_dir="${host_dir}${uuid}"

    if [[ ! -d "$old_dir" ]]; then
      # Source dir does not exist (already renamed or never existed)
      total_skipped=$(( total_skipped + 1 ))
      continue
    fi

    if [[ -d "$new_dir" ]]; then
      # Target dir already exists
      echo "  skip $hostname/$number (target $uuid already exists)"
      total_skipped=$(( total_skipped + 1 ))
      continue
    fi

    if [[ "$DRY_RUN" == "true" ]]; then
      echo "  [dry-run] mv ${host_dir}${number} → ${host_dir}${uuid}"
    else
      mv "$old_dir" "$new_dir"
      echo "  $hostname: $number → $uuid"
    fi
    total_renamed=$(( total_renamed + 1 ))
  done <<< "$pairs"
done

echo ""
if [[ "$DRY_RUN" == "true" ]]; then
  echo "[dry-run] Pass 2: would rename ${total_renamed} director(ies), ${total_skipped} skipped"
else
  echo "Pass 2 done. Renamed ${total_renamed} director(ies), ${total_skipped} skipped"
fi
