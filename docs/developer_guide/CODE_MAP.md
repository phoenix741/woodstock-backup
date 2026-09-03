# Code Organisation (Code Map)

This document describes the physical structure of the repository and the distribution of responsibilities across the different Rust modules.

## Top-Level Layout

```
.
├── woodstock-rs/       # [LIB] System core, shared business logic
├── server-rs/          # [BIN] Server component implementation
├── client-rs/          # [BIN] Standalone backup agent
├── cli-rs/             # [BIN] Command-line administration tools
├── front/              # [UI]  Web interface (Vue.js)
└── woodstock.proto     # [DEF] gRPC definitions (inside woodstock-rs)
```

## 1. woodstock-rs (Library Crate)

This is the foundation of the project. It contains no executable binary (except tests/benchmarks), only the data-manipulation logic.

* `src/pool/` : **Physical storage management**.
  * Chunk read/write (`PoolManager`, `PoolChunkWrapper`).
  * Storage path calculation (3-level sharding).
  * Reference counter management (`Refcnt`, `RefcntApplySens`).
  * Integrity verification (`check_pool_integrity`, `check_backup_integrity`).
* `src/manifest/` : **Backup index management**.
  * `FileManifest` structure (file list, POSIX attributes).
  * `IndexManifest` / `PathManifest` / `IndexFileEntry` for fast indexing.
  * Reading files from the pool (`FileManifestReader`).
* `src/config/` : **Configuration structures and metadata management**.
  * `Configuration` / `ConfigurationPath` (paths — overridable via environment variables).
  * `Backups` / `Hosts` (accessors for host/backup metadata).
  * `ApplicationScheduler` / `Schedule` / `ScheduledBackupToKeep` (scheduling).
  * `blackout.rs` : `BlackoutWindow`, `blackout_status_at()` — pure recurring time-window evaluation for `Schedule.blackout`.
  * Full `BackupStatus` enum (`InProgress`, `Completing`, `Completed`, `Failed`, …).
  * `HostConfiguration` (per-host YAML: addresses, shares, pre/post commands).
* `src/events/` : **Audit log**.
  * Event read/write (`append_events`, `read_events`).
  * `create_event_backup_*` functions for each backup lifecycle step.
* `src/proto/` : Rust code auto-generated from `woodstock.proto` (via `tonic::include_proto!`).
* `src/server/` : **Server-side state machines**.
  * `backup/` : `SaveMachine`, `RestoreMachine`, `RemoveMachine`.
  * `pool/` : `FsckMachine`, `PoolCleanerMachine`, `HashConverterMachine`.
  * `client/grpc.rs` : gRPC `WoodstockClientService` client (connection to agent).
  * `job.rs`, `progression.rs`, `resolve.rs`, `tools.rs`.
* `src/statistics/` : Pool statistics and history (`PoolStatistics`, `HistoricalPoolStatistics`).
* `src/utils/` : Toolbox (Hashing — Blake3/SHA2/SHA3, Encryption, Compression — Zstd/Deflate, DateTime, path utilities `mangle`/`path_to_vec`, Redis distributed locks `PoolLockRedis`).
* `src/view/` : `FileManifest` transformation helpers (aggregation from host/backup).

## 2. server-rs (Binaries Crate)

Contains the application-level server implementation.

* `src/bin/` : Binary entry points.
  * `api_server.rs` : Public Axum HTTP server (REST + GraphQL).
  * `client_api_server.rs` : HTTP/mTLS server for agent registrations.
  * `scheduler.rs` : Event-driven + dynamic-wakeup scheduling process (single instance) — one unified loop drives hosts, archive profiles, and nightly maintenance, no Apalis cron.
  * `job_worker.rs` : Task execution process (backup, restore, maintenance).
* `src/api/` : REST API logic.
  * `handlers/` : HTTP controllers (hosts, backups, files, server, metrics).
  * `dto/` : Data Transfer Objects (backup, files, hosts, events, stats, queue, pool).
  * `services/` : Business orchestration (hosts, backups, files, queue, metrics, certificate, server).
  * `state.rs` : `ApiServerState` injected into every handler.
  * `routes.rs` : Axum router with all REST routes.
* `src/graphql/` : GraphQL layer (via `async-graphql`).
  * `resolvers/query.rs` : Queries (hosts, backups, stats, events, …).
  * `resolvers/mutation.rs` : Mutations (createBackup, removeBackup, restoreBackup, fsck, cleanupPool).
  * `resolvers/types.rs` : GraphQL types (`BackupEx`, …).
  * `progress/` : WebSocket Subscriptions for real-time job tracking.
* `src/jobs/` : Asynchronous task execution logic (via **Apalis**).
  * `workers.rs` : Job executors (`handle_backup`, `handle_restore`, `handle_remove`, `handle_fsck`, …).
  * `storage.rs` : Configuration of the 4 Redis queues (schedule, backup, interactive, maintenance).
  * `producers.rs` : Job enqueue methods (`enqueue_backup_unique`, `enqueue_restore`, …).
  * `decision.rs` : `try_schedule_host()` — shared scheduling gate (cooldown, running, due, blackout, fsck lock, reachability) used by both the scanner loop and the event-driven subscriber in `bin/scheduler.rs`.
  * `progress.rs` : Redis Pub/Sub for publishing progress states.
  * `types.rs` : Payload definitions (`BackupQueueJob`, `RestoreJobData`, `MaintenanceJobData`).
  * `layers/` : Apalis middlewares (progress, job_log).
* `src/client_api/` : HTTP/mTLS server for agent-initiated connections.
  * Endpoint `POST /api/hosts/{name}/client` : Registers an agent's network address.
  * `auth.rs`, `middleware.rs` : mTLS client certificate validation.
* `src/shared_state.rs` : Infrastructure shared between `api_server` and `job_worker` (config, hosts, backups, resolver).

## 3. client-rs (Binary Crate)

The lightweight agent deployed on target machines. Produces two binaries:

* **`ws_client_daemon`** : Passive gRPC server waiting for orders from the `job_worker`.
* **`ws_client_console`** : Interactive local administration console.

Internal modules:

* `src/server.rs` : Implementation of the gRPC service `WoodstockClientService` (receives orders from `job_worker`) — methods: `ping`, `authenticate`, `execute_command`, `synchronize_file_list`, `get_chunk_hash`, `get_chunk`, `restore_file`, `close_backup`.
* `src/scanner/` : Filesystem traversal engine.
  * `file_browser.rs` : Recursive directory traversal.
  * `file_reader.rs` : Content and metadata extraction (chunk hashing).
  * `file_writer.rs` : File writing during restore.
  * `metadata/` : OS metadata abstraction (ACLs, Xattrs, permissions — Unix/Windows split with feature gating).
* `src/storage/` : Snapshot management.
  * `snapshots/btrfs.rs` : **Btrfs** driver (implemented, Linux).
  * `snapshots/vss.rs` : **VSS** driver (implemented, Windows — local drive-letter volumes only).
  * `snapshots/` : ZFS — **planned, not yet implemented**.
  * `accessor.rs` : `FileSystemAccessor` — unified access with path redirections for snapshots.
* `src/authentification.rs` : JWT authentication and session management (configurable timeout).
* `src/resolve/` : Server discovery — Direct or **mDNS** (feature gated).
* `src/updater.rs` : Automatic agent update mechanism.

## 4. cli-rs (Binary Crate)

Direct administration tools, useful for debugging or disaster recovery without going through the API.

* `ws_console` : Raw server data inspection toolkit.
  * `read-protobuf <path> <format>` : Read a Protobuf file (manifest, journal, refcount, events, chunk-info).
  * `read-log <hostname> <backup_id> <share>` : Read a backup journal log.
  * `get-chunk <sha256hex>` : Retrieve the decompressed content of a chunk.
  * `search-chunk <sha256hex>` : Find manifests referencing a given chunk.
  * `compare <source> <target>` : Diff between two manifests (generates a journal).
  * `compact-refcnt <host> <backup_id>` : Add a reference count to the pool for a backup.
  * `clean-unused` : Clean unused chunks from the pool (respects refcounts).
  * `check-compression` : Check compression ratios and report anomalies.
  * `fsck` : Verify pool integrity (`--dry-run`, `--verify-chunks`, `--skip-ref-unused`).
  * `convert-hash-repo` : Convert the hash format of a repository.
  * `resolve-host <hostname>` : Resolve a hostname via the Redis cache (debug).
  * `mount <host> <backup_id>` : Mount a backup read-only via FUSE (Unix only).
* `ws_restore` : Manual restore from local storage (without a running server).
* `ws_sync` : Low-level synchronisation utilities.
