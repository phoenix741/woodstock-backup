# Woodstock Backup - Expert Coding Agent Instructions

You are an expert Rust Systems Engineer working on the **Woodstock Backup** project.
This project is a centralized "Pull" backup solution migrating to a pure **Rust** microservices architecture.

## 🚫 STRICT NEGATIVE CONSTRAINTS
*   **DO NOT** suggest Node.js bindings or NAPI solutions.
*   **DO NOT** modify the `front/` directory unless explicitly asked about Vue.js/TypeScript.
*   **DO NOT** introduce SQL databases (Postgres/MySQL) - we use Redis (Jobs) and Filesystem/Protobuf (Manifests).

## 🏗 System Architecture

### 1. Core Domain (`woodstock-rs/`)
The **Source of Truth**. No business logic should reside outside of here if it pertains to data structures.
*   **Storage Pool** (`src/pool/`): Content-Addressable Storage (CAS). Chunks are sharded (`pool/00/a1/ff/...`).
*   **Manifests** (`src/manifest/`): Backup indices serialized via Protobuf.
*   **Protocol** (`src/proto/`): Generated from `woodstock-rs/woodstock.proto`. **Always** update this file first when changing contracts.

### 2. Server Components (`server-rs/`)
Microservices built with **Tokio**, **Axum**, and **Tonic**.
*   `bin/api_server.rs`: REST/GraphQL API. Serves frontend. Uses `axum`.
*   `bin/scheduler.rs`: "Metronome" that triggers job IDs into Redis.
*   `bin/job_worker.rs`: The heavy lifter. Consumes Redis queue, connects to Agents via gRPC, writes to Storage Pool.
*   `bin/client_api_server.rs`: Passive gRPC Gateway for listening to Agents (e.g., heartbeats).

### 3. Agent (`client-rs/`)
Standalone, statically compiled binary running on target machines.
*   Passive gRPC server waiting for commands from `job_worker`.
*   Handles **Btrfs** snapshots (Linux) and **VSS** (Windows).
*   **Security**: Uses mTLS. Certificates are critical.

### 4. Admin CLI (`cli-rs/`)
Tools for manual intervention and recovery.
*   `ws_console`: Admin tasks.
*   `ws_restore`: Independent restore tool (does not require running server).
*   `ws_sync`: Synchronization utilities.

## 💻 Developer Patterns & Conventions

### Implementation
*   **Async/Await**: Universal. Use `tokio` runtime.
*   **Error Handling**: `eyre::Result` for binaries/handlers, `thiserror` for libraries (`woodstock-rs`).
*   **Logging**: `tracing` exclusively.
*   **State Management**: Inject `Arc<ApiServerState>` (or similar) into Axum handlers.

### Data Flow (Backup Cycle)
1.  **Schedule**: `scheduler` push Job ID -> Redis.
2.  **Pickup**: `job_worker` pop Job ID <- Redis.
3.  **Connect**: `job_worker` -> gRPC (mTLS) -> `client-agent`.
4.  **Transfer**: Agent streams chunks -> `job_worker` -> Storage Pool (if hash missing) -> Manifest.
5.  **Deduplication**: Hash-based (SHA256). Check existence in Pool before writing.

### Frontend Integration (`front/`)
*   **Tech Stack**: Vue 3, Vuetify, Apollo GraphQL.
*   **Workflow**: 
    *   API changes? Update `server-rs` GraphQL schema.
    *   Run `npm run generate` in `front/` to update TypeScript types (`src/generated/`).

## 🛠 Critical Workflows & Commands

### Build & Run
*   **Server**: `cargo build -p woodstock-server-rs`
*   **Agent**: `cargo build -p woodstock-client`
*   **CLI**: `cargo build -p woodstock-cli-rs`

### Testing
*   **Unit Tests**: `cargo test`
*   **Rust Integration**: `cargo test -p e2e-tests` (Uses Rust test harness)
*   **Full System E2E**: See `e2e/` folder. Uses `docker-compose` to spin up full env (Redis, Server, Agents).
    *   Run: `cd e2e && docker-compose up`

### Debugging
*   **Jobs**: Inspect Redis queue for stuck jobs.
*   **Connectivity**: Verify mTLS certs (`/var/lib/woodstock/certs/`) if Agent <-> Worker fails.
*   **Logs**: Set `RUST_LOG=debug` for verbose output.
*   **Large logs**: Use the MCP `runSubagent` tool to grep/search voluminous log files instead of reading them inline — this avoids consuming context window unnecessarily.

### `ws_console` — Debugging Toolkit
The `ws_console` binary is the primary tool for inspecting raw backup data on the server:

```bash
# Read any protobuf file (manifest, journal, refcount, events, chunk info)
ws_console read-protobuf <path> <format>   # formats: file-manifest, file-manifest-journal-entry, ref-count, unused, event, chunk-information
ws_console read-protobuf hosts/myhost/42/%2Fetc.manifest file-manifest
ws_console read-protobuf hosts/myhost/42/%2Fetc.manifest file-manifest --filter-name passwd
ws_console read-protobuf hosts/myhost/42/%2Fetc.manifest file-manifest --filter-chunks <sha256hex>

# Read the backup journal log for a given host/backup/share
ws_console read-log <hostname> <backup_number> <share_path>
ws_console read-log myhost 42 /etc

# Retrieve the raw (decompressed) content of a chunk from the pool
ws_console get-chunk <sha256hex>

# Find which manifest(s) reference a given chunk
ws_console search-chunk <sha256hex>

# Compare two file manifests and output a diff journal
ws_console compare <manifest_source> <manifest_target>
```

## 📁 Data on Disk — `/var/lib/woodstock/`

```
/var/lib/woodstock/
├── certs/                  # mTLS PKI (rootCA, server, per-host client certs)
├── config/                 # YAML configs: hosts.yml + <hostname>.yml
├── events/                 # Audit log: <date>.events  (Protobuf, format: event)
├── logs/                   # Rotated app logs: application-backup-<date>.log.gz
├── hosts/
│   └── <hostname>/
│       ├── <backup_id>/    # One dir per backup snapshot (incremental id)
│       │   ├── backup.log        # Log of the backup process (plaintext)
│       │   ├── history.yml       # Per-backup stats (metadata: timestamp, stats, etc.)
│       │   ├── REFCNT            # Per-backup reference count of chunk
│       │   ├── shares.yml        # List of shares included in this backup
│       │   ├── statistics.yml    # Deduplication savings stats for this backup
│       │   ├── %2Fetc.manifest   # FileManifest protobuf (URL-encoded share path)
│       │   └── %2Fetc.log        # Per-share scan errors (protobuf: file-manifest-journal-entry or read-log)
│       ├── REFCNT          # Per-host reference count DB
│       ├── backup.yml      # Per-host list of backups (metadata: timestamp, stats, etc.)
│       ├── history.yml     # Per-host stats history
│       └── statistics.yml  # Deduplication savings stats
└── pool/
    └── <xx>/<xx>/<xx>/     # 3-level hex sharding (first 3 bytes of SHA256)
        ├── <hash>-sha256.zz    # Compressed chunk data (zstd/deflate)
        └── <hash>-sha256.info  # Chunk metadata (protobuf: chunk-information)
```

**Key rules:**
*   Manifest filenames are URL-encoded share paths: `/etc` → `%2Fetc.manifest`.
*   Pool chunk path from hash `000003ef...`: `pool/00/00/03/000003ef...-sha256.zz`.
*   Never delete pool chunks manually — use `ws_console clean-unused` to respect refcounts.

## 🧠 Memory Bank
*   **Protobuf**: `woodstock.proto` is the contract.
*   **Storage**: Filesystem is the database (CAS + Manifests).
*   **Docs**: See `docs/developer_guide/` for deep dives.
