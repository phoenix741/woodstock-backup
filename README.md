# Woodstock Backup

## Build Status

### Master

[![Master Release Status](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/actions/workflows/on-release.yml/badge.svg)](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup)

### Alpha

[![Alpha Release Status](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/actions/workflows/on-release.yml/badge.svg?branch=alpha)](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup)

## Description

**Woodstock Backup** is a centralized, server-initiated backup solution written entirely in **Rust**.

Instead of launching the backup from the client, the **server contacts the client** to initiate the backup — a pull model that removes the need for any client-side scheduling and prevents compromised clients from tampering with backup data.

### How it works

- The **server** (four Rust microservices) orchestrates all backups, manages the storage pool, and exposes a web interface.
- A lightweight **agent** (`ws_client_daemon`) runs on each target machine. It waits passively for gRPC orders from the server.
- Files are transferred as **content-addressed chunks** (16 MB by default, deduplicated by Blake3 hash), stored in a hierarchical pool.
- When a compatible filesystem is available, the agent creates a **snapshot** (Btrfs on Linux, VSS on Windows) before scanning files, ensuring a consistent backup.
- All server-to-agent communication is secured by **mutual TLS (mTLS)** and JWT authentication.

### Server components

| Binary | Role |
|--------|------|
| `api_server` | REST + GraphQL API, serves the Vue.js web interface |
| `client_api_server` | mTLS HTTP gateway for agent self-registration |
| `job_worker` | Asynchronous worker: backups, restores, maintenance |
| `scheduler` | CRON-based scheduler that enqueues jobs into Valkey/Redis |

### Key features

- **Native Windows support** — no rsync or SSH required on clients
- **Chunk-based deduplication** with Blake3/SHA2/SHA3 hashing and Zstd/Deflate compression
- **FUSE filesystem mount** — browse any backup point as a regular directory (`ws_console mount`)
- **Retention policy** — sliding window algorithm (hourly, daily, weekly, monthly, yearly)
- **Prometheus metrics** and real-time progress via GraphQL WebSocket subscriptions
- **Migration tool** from BackupPC (`ws_backuppc_importer`)

## Documentation

Full documentation is available at [woodstockbackup.shadoware.org](https://woodstockbackup.shadoware.org).

- [Installation guide](https://woodstockbackup.shadoware.org/doc/installation)
- [Agent setup](https://woodstockbackup.shadoware.org/doc/agent)
- [Configuration reference](https://woodstockbackup.shadoware.org/doc/configuration)
- [Developer guide](docs/developer_guide/README.md)

## Building from source

```bash
# Server
cargo build --release -p woodstock-server-rs

# Agent
cargo build --release -p woodstock-client

# CLI tools
cargo build --release -p woodstock-cli-rs
```

## License

[MIT](https://choosealicense.com/licenses/mit/)
