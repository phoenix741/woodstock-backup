# Data Model & Storage

Woodstock uses a **Content-Addressable Storage (CAS)** approach to guarantee deduplication and data integrity. This page describes the physical structure of data on the server disk.

## 1. Filesystem Structure

All data of a Woodstock instance is stored under a common root (e.g. `/var/lib/woodstock/`).

```
<ROOT>/
├── certs/                  # mTLS certificates (CA, Server, Clients)
│   ├── rootCA.pem          # Internal certificate authority
│   ├── server_server.key   # Server private key
│   └── <hostname>_<role>.{pem,key}
│
├── config/                 # YAML configuration files
│   ├── hosts.yml           # List of managed hostnames
│   ├── scheduler.yml       # Scheduler configuration (CRON, retention)
│   ├── statistics.yml      # Global pool statistics
│   └── <hostname>.yml      # Per-machine config (addresses, shares, schedule)
│
├── events/                 # Event journal (Audit log)
│   └── <date>.events       # e.g. 2026-01-11.events (Protobuf format)
│
├── logs/                   # Rotated application logs
│   ├── application-backup-<date>.log.gz
│   ├── application-stats-<date>.log.gz
│   └── jobs/               # Job-specific logs
│
├── hosts/                  # Per-machine backup data
│   └── <hostname>/
│       ├── <backup_id>/    # Backup folder (UUID v7)
│       │   ├── backup.log        # Backup execution log (plain text)
│       │   ├── history.yml       # Backup stats (timestamp, size, dedup)
│       │   ├── REFCNT            # Reference counts specific to this backup
│       │   ├── shares.yml        # List of shares included in this backup
│       │   ├── statistics.yml    # Deduplication stats for this backup
│       │   ├── %2Fetc.manifest   # FileManifest (URL-encoded path)
│       │   └── %2Fetc.log        # Per-share scan errors (Protobuf journal)
│       ├── REFCNT          # Host-level reference count database
│       ├── backup.yml      # List of this host's backups (metadata)
│       ├── history.yml     # Statistics history
│       └── statistics.yml  # Global deduplication stats for this host
│
└── pool/                   # Deduplicated storage (CAS)
    ├── algorithm           # Active hash algorithm (blake3, sha2_256, sha3_256)
    ├── REFCNT.dirty        # Refcount consistency marker
    ├── refcnt/             # Pool reference counters
    │   └── pending/        # Pending refcounts awaiting application
    └── <hex>/<hex>/<hex>/  # 3-level sharding (e.g. 00/00/03/)
        ├── <hash>-<algo>.zz    # Compressed chunk data
        └── <hash>-<algo>.info  # Chunk metadata (Protobuf)
```

## 2. The Pool (Chunk Store)

The **Pool** holds the raw file data. It is agnostic of the original directory tree.

### Sharding and Addressing

To optimise filesystem performance (avoid millions of files in one directory), the pool uses deep sharding on the first 3 bytes of the hash:

* **Full hash**: `000003ef13cc3c...`
* **Path**: `pool/00/00/03/000003ef13cc3c...-sha256.zz`

The active hash algorithm is stored in `pool/algorithm` (e.g. `blake3`, `sha2_256`, `sha3_256`).

### Chunk Files

For each unique data block, two files are created:

1. **`.zz` (Data)**: The chunk content, compressed (Zstd or Deflate depending on configuration) and optionally encrypted.
2. **`.info` (Metadata)**: Technical information about the chunk (original size, compressed size, checksum) in Protobuf format (`ChunkInformation`).

### Reference Counting (RefCnt)

Reference counters are stored under `pool/refcnt/` with a pending mechanism in `pool/refcnt/pending/`. The file `pool/REFCNT.dirty` acts as a consistency marker.

A chunk is never deleted as long as it is referenced by at least one manifest. Cleanup is performed by the `CleanupRefcnt` task (via `ws_console clean-unused` or the GraphQL mutation `cleanupPool`).

## 3. Hosts and Manifests (`hosts/`)

Unlike the Pool, the `hosts/` directory is organised logically by machine and backup identifier.

### Backup Structure (`hosts/<hostname>/<backup_id>/`)

Each backup is identified by a **UUID v7** (time-sortable, monotonically increasing generator) — no longer a sequential integer. It contains:

* **`backup.log`**: Backup execution log (plain text — network errors, progress).
* **`history.yml`**: Backup metadata (start/end timestamp, size, deduplication ratio).
* **`REFCNT`**: Reference counts of chunks specifically for this backup.
* **`shares.yml`**: List of shares included in this backup.
* **`statistics.yml`**: Deduplication and space-saving stats for this backup.
* **Manifests (`.manifest`)**:
  * Filenames are the original paths **URL-encoded**.
  * Example: `/etc` becomes `%2Fetc.manifest`.
  * These files contain the list of files in the share, their permissions (POSIX stat), ACLs, xattrs, and the list of hashes pointing into the Pool.
* **Logs (`.log`)**:
  * Per-share scan logs (e.g. `%2Fetc.log`), in Protobuf `FileManifestJournalEntry` format.
  * Contains specific errors (locked file, permission denied) encountered during the scan.

### Host-Level Files

At the root of `hosts/<hostname>/`, global files are found:

* **`backup.yml`**: List of all backups for this host (metadata — status, dates, size, dedup ratio).
* **`REFCNT`**: Aggregated reference count database for this host.
* **`history.yml`**: Historical deduplication statistics over time.
* **`statistics.yml`**: Aggregated deduplication stats for all backups of this machine.

## 4. Security and Certificates (`certs/`)

The architecture relies on an internal PKI (Public Key Infrastructure) stored in `certs/`.

* **CA**: `rootCA.{pem,key}` signs all certificates.
* **Roles**: Each certificate has a suffix defining its role:
  * `_server`: Used by the `api_server` or `client_api_server` component.
  * `_client`: Used by the `ws_client_daemon` agent to authenticate itself.
  * `_https`: Used to expose the web interface (if configured).

### Reconstruction Example

To restore `/etc/hosts`:

1. The system reads the manifest `%2Fetc.manifest` in `hosts/<hostname>/<backup_id>/`.
2. It finds the `hosts` entry.
3. It reads the associated list of hashes.
4. It retrieves the chunks from the Pool via their sharded path (`pool/xx/xx/xx/...`).
5. It decompresses (`.zz`) and assembles the file.

## 5. Environment Variables

Data locations can be overridden via environment variables:

| Variable | Default |
|---|---|
| `BACKUP_PATH` | `/var/lib/woodstock` |
| `CERTIFICATES_PATH` | `$BACKUP_PATH/certs` |
| `CONFIG_PATH` | `$BACKUP_PATH/config` |
| `HOSTS_PATH` | `$BACKUP_PATH/hosts` |
| `POOL_PATH` | `$BACKUP_PATH/pool` |
| `LOGS_PATH` | `$BACKUP_PATH/logs` |
| `EVENTS_PATH` | `$BACKUP_PATH/events` |
| `REDIS_HOST` | `localhost` |
| `REDIS_PORT` | `6379` |
| `CHUNK_ALGORITHM` | `blake3` |
| `COMPRESSION_FORMAT` | `zstd` |
