# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Woodstock Backup is a centralized, **pull-model** backup solution written in Rust. The server contacts agents to initiate backups (agents never push), so a compromised client cannot tamper with existing backup data. Files are transferred as content-addressed, deduplicated chunks (Blake3-hashed, Zstd/Deflate-compressed) stored in a hierarchical pool. Server↔agent communication is gRPC over mTLS + JWT.

Contributions must relate to backup/storage/administration — see `CONTRIBUTING.md` for scope and PR process (branch from `develop`).

## Strict constraints

- Do not introduce SQL databases. Persistence is Redis/Valkey (job queues, distributed locks) + filesystem/Protobuf (manifests, pool).
- Despite the field/struct name `sha256` used throughout the pool/refcnt code (legacy naming), chunk hashing actually uses **Blake3** (`woodstock-rs/src/utils/chunk_hasher.rs`). SHA2/SHA3 are also available as dependencies but Blake3 is the default.
- `woodstock-rs/woodstock.proto` is the contract for all gRPC/data structures — update it first when changing wire formats, then regenerate (`tonic::include_proto!` handles this at build time, no separate codegen step needed).

## Build & test

```bash
# Build (per-package, from repo root)
cargo build -p woodstock-server-rs      # api_server, client_api_server, job_worker, scheduler
cargo build -p woodstock-client-rs      # ws_client_daemon, ws_client_console
cargo build -p woodstock-cli-rs         # ws_console, ws_sync, ws_restore
cargo build --release -p <same>         # release builds

# Lint / format
cargo clippy
cargo fmt

# Unit tests (all crates)
cargo test
cargo test -p woodstock-rs some_test_name   # single test

# Rust integration tests (in-process client/server, no external services)
cargo test -p e2e-tests

# Full system E2E (QEMU VMs + Packer golden images + bats-core, from docker packages) — heavyweight, see e2e/README.md
cd e2e && cp e2e.conf.example e2e.conf && ./images/build-all.sh && ./run.sh --server debian --clients debian
```

Note: `e2e/` (VM-based, QEMU/Packer/bats) and `e2e-tests/` (Rust crate, `cargo test -p e2e-tests`) are two different, unrelated test suites — don't confuse them. `docker-compose.yml` at the repo root spins up the server stack (website, server-api, etc.) for manual/dev use, not for the E2E suites.

### Frontend (`front/`, Vue 3 + Vuetify + Apollo GraphQL)

```bash
cd front
npm run dev            # vite dev server
npm run build           # type-check + vite build
npm run lint             # eslint --fix
npm run format            # prettier
npm run generate           # graphql-codegen --watch — regenerate src/generated/ after changing server-rs GraphQL schema
```

## Architecture

Four independent server binaries (all in `server-rs/src/bin/`), a standalone agent, and admin CLI tools, sharing core logic from the `woodstock-rs` library.

- **`api_server`**: Axum HTTP server — REST + GraphQL (async-graphql) + WebSocket subscriptions for real-time progress. Serves the frontend. No heavy work.
- **`scheduler`**: Event-driven + dynamic-wakeup planner (single instance, see `docs/developer_guide/SERVER_COMPONENTS.md`) — backs up a host as soon as it registers online, otherwise sleeps until the next real deadline across hosts, archive profiles, and nightly maintenance (each tracked against its own persisted "last done" state, no Apalis cron involved), bounded by a global safety-net ceiling (`wakeupSchedule`, guards only against config changes going unnoticed, not a polling cadence). Honors per-host/global blackout windows.
- **`job_worker`**: Consumes 4 Redis queues via Apalis (schedule, backup, interactive, maintenance); does backups/restores/maintenance; talks to agents over gRPC mTLS; writes to the storage pool and manifests. Horizontally scalable.
- **`client_api_server`**: HTTP/mTLS (not gRPC) — lets agents self-register their network address.
- **`client-rs`**: builds `ws_client_daemon` (passive gRPC server awaiting orders — `ping`, `authenticate`, `execute_command`, `synchronize_file_list`, `get_chunk_hash`, `get_chunk`, `restore_file`, `close_backup`) and `ws_client_console` (local admin). Handles Btrfs snapshots (Linux, implemented) and VSS (Windows, implemented, local drives only); ZFS is planned but not implemented.
- **`cli-rs`**: builds `ws_console` (raw data inspection/repair — see below), `ws_restore` (restore without a running server), `ws_sync` (low-level sync utilities).
- **`woodstock-rs`**: the shared library — source of truth for data structures. No business logic should live outside it if it concerns data structures. Key modules: `src/pool/` (CAS storage, sharded 3-level hex paths, refcounts, integrity checks), `src/manifest/` (FileManifest / IndexManifest, Protobuf-serialized), `src/proto/` (generated from `woodstock.proto`), `src/config/`, `src/events/` (audit log), `src/server/` (state machines: `SaveMachine`, `RestoreMachine`, `RemoveMachine`, `FsckMachine`, `PoolCleanerMachine`, `HashConverterMachine`), `src/statistics/`, `src/utils/` (hashing, encryption, compression, `mangle`/`path_to_vec`, Redis distributed locks).

Full details, including the GraphQL/REST layer breakdown inside `server-rs`: `docs/developer_guide/CODE_MAP.md`, `docs/developer_guide/ARCHITECTURE.md`, `docs/developer_guide/DATA_STRUCTURES.md`, `docs/developer_guide/ARCHIVING.md`, `docs/developer_guide/RETENTION.md`, `docs/developer_guide/CLIENT_AGENT.md`.

### Typical backup flow

1. `scheduler` enqueues `BackupQueueJob::Save` (atomic dedup via Redis `SET NX`).
2. `job_worker` dequeues it via Apalis, connects to the agent (gRPC mTLS), authenticates (JWT).
3. Agent snapshots the volume, then streams file metadata via `synchronize_file_list()`.
4. Worker checks each hash with `get_chunk_hash()`; existing chunks are referenced, missing ones streamed via `get_chunk()` into the pool.
5. Worker writes the Manifest and updates refcounts; progress is published via Redis Pub/Sub → GraphQL Subscriptions → frontend.
6. Agent deletes the temporary snapshot via `close_backup()`.

### `ws_console` — data inspection/repair toolkit

Primary tool for inspecting raw backup data. Paths passed to it must be absolute (resolved against cwd, not `BACKUP_PATH`).

```bash
ws_console read-protobuf <path> <format>   # formats: file-manifest, file-manifest-journal-entry, ref-count, unused, event, chunk-information
ws_console read-protobuf /var/lib/woodstock/hosts/myhost/<uuid>/%2Fetc.manifest file-manifest --filter-name passwd
ws_console read-protobuf /var/lib/woodstock/hosts/myhost/<uuid>/%2Fetc.manifest file-manifest --filter-chunks <sha256hex>
ws_console read-log <hostname> <backup_number> <share_path>   # e.g. read-log myhost 42 /etc
ws_console get-chunk <sha256hex>              # decompressed chunk content
ws_console search-chunk <sha256hex>           # manifests referencing a chunk
ws_console compare <manifest_source> <manifest_target>   # diff -> journal
ws_console fsck [--dry-run] [--verify-chunks] [--skip-ref-unused]
ws_console clean-unused                       # respects refcounts — never delete pool chunks by hand
ws_console mount <host> <backup_id>           # FUSE read-only mount (Unix only)
```

### Data on disk — `/var/lib/woodstock/`

```
/var/lib/woodstock/
├── certs/                  # mTLS PKI (rootCA, server, per-host client certs)
├── config/                 # YAML configs: hosts.yml + <hostname>.yml
├── events/                 # Audit log: <date>.events  (Protobuf, format: event)
├── logs/                   # Rotated app logs: application-backup-<date>.log.gz
├── hosts/
│   └── <hostname>/
│       ├── <uuid>/         # One dir per backup, named after its UUID
│       │                   # (the sequential number is a display label only)
│       │   ├── backup.log        # Log of the backup process (plaintext)
│       │   ├── history.yml       # Per-backup stats
│       │   ├── REFCNT            # Per-backup reference count of chunks
│       │   ├── shares.yml        # Shares included in this backup
│       │   ├── statistics.yml    # Deduplication savings stats
│       │   ├── %2Fetc.manifest   # FileManifest protobuf (URL-encoded share path)
│       │   └── %2Fetc.log        # Per-share scan errors
│       ├── REFCNT          # Per-host reference count DB
│       ├── backup.yml      # Per-host list of backups
│       ├── history.yml     # Per-host stats history
│       └── statistics.yml  # Deduplication savings stats
└── pool/
    └── <xx>/<xx>/<xx>/     # 3-level hex sharding (first 3 bytes of hash)
        ├── <hash>-sha256.zz    # Compressed chunk data (zstd/deflate)
        └── <hash>-sha256.info  # Chunk metadata (protobuf: chunk-information)
```

Key rules:
- Manifest filenames are URL-encoded share paths with *every* non-alphanumeric byte escaped: `/etc` → `%2Fetc.manifest`, `/srv/my-data` → `%2Fsrv%2Fmy%2Ddata` (`mangle()`, `woodstock-rs/src/utils/path.rs`).
- Pool chunk path from hash `000003ef...`: `pool/00/00/03/000003ef...-sha256.zz`.
- Never delete pool chunks manually — use `ws_console clean-unused` to respect refcounts.

## Conventions

- `eyre::Result` for binaries/handlers, `thiserror` for library errors (`woodstock-rs`).
- `tracing` exclusively for logging — no `println!`/`eprintln!`.
- Async/await everywhere, `tokio` runtime.
- `Arc<ApiServerState>` (or equivalent) injected into Axum handlers.
