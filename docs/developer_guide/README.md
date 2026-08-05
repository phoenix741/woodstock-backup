# Woodstock Backup - Developer Guide

This folder contains the technical documentation for developers who want to contribute to or understand the internal architecture of Woodstock Backup (Rust version).

## Table of Contents

1. **[Global Architecture](ARCHITECTURE.md)**
    * Overview of components (Server, Client, CLI).
    * Data flows and protocols (gRPC, mTLS, GraphQL).
    * Job management and scaling.

2. **[Code Organisation](CODE_MAP.md)**
    * Monorepo structure.
    * Details of Rust crates (`woodstock-rs`, `server-rs`, `client-rs`, `cli-rs`).
    * Responsibilities of each module.
    * Available `ws_console` commands.

3. **[Data Model & Storage](DATA_STRUCTURES.md)**
    * How the **Pool** (Content Addressable Storage) works.
    * Complete filesystem layout under `/var/lib/woodstock/`.
    * **Manifest** structure (Protobuf).
    * Reference counting (RefCnt) and deduplication.
    * Configuration environment variables.

4. **[Server Components](SERVER_COMPONENTS.md)**
    * Details of binaries: `api_server`, `scheduler`, `job_worker`, `client_api_server`.
    * REST API (routes), GraphQL (queries, mutations, subscriptions).
    * Apalis job queues (4 queues) and worker types.
    * Real-time progress tracking (Redis Pub/Sub).

5. **[Client & Agent](CLIENT_AGENT.md)**
    * Binaries: `ws_client_daemon`, `ws_client_console`.
    * Implemented gRPC methods (`WoodstockClientService`).
    * Snapshot drivers (Btrfs and VSS/Windows implemented; ZFS planned).
    * File scanner and metadata handling (ACL, Xattr).
    * Network discovery (mDNS) and automatic updates.

6. **[Backup Retention](RETENTION.md)**
    * Time-based sliding window algorithm (Hourly, Daily, Weekly, Monthly, Yearly).
    * Removal process and synchronization locking (`backup.yml`).

## Packaging & Deployment

### Debian / Ubuntu

The `woodstock-server` Debian package is built using `cargo-deb` and includes:
* All 4 server binaries (`api_server`, `client_api_server`, `job_worker`, `scheduler`)
* The pre-compiled Vue.js frontend in `/usr/share/woodstock/static/`
* 4 systemd services + `woodstock.target` (in `server-rs/debian/`)
* A `woodstock` system user and `/var/lib/woodstock/` data structure

Build locally: `cargo deb -p woodstock-server-rs` (requires `front/dist/` to be present)

### FreeBSD

FreeBSD packages are built with `pkg create` using manifests and scripts in `server-rs/freebsd/` and `client-rs/freebsd/`:
* 4 rc.d scripts, data under `/var/db/woodstock/`, config in `/usr/local/etc/woodstock/`
* Build script: `scripts/create-freebsd-pkg.sh <version> <binaries_dir> <output_dir> [front_dist_dir]`
* Published to the Gitea generic package registry (not a native pkg repo)
  * GraphQL integration.
