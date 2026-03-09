# Client & Agent (`client-rs`)

The `client-rs` agent is a standalone binary designed to run on target machines. It prioritises minimal resource usage at rest and maximum performance during backups.

## Binaries

* **`ws_client_daemon`**: Main daemon — passive gRPC server waiting for orders from the `job_worker`.
* **`ws_client_console`**: Interactive local administration console.

## Agent Architecture

The agent operates in "passive gRPC server" mode. It listens on a configurable TCP port and waits for instructions from the central Woodstock server.

### Internal Components

1. **gRPC Server (`src/server.rs`)**
    * Implements the `WoodstockClientService` trait defined in `woodstock.proto`.
    * Exposed methods:
      * `ping()` — Connectivity check.
      * `authenticate()` — JWT authentication (signed token, sessions with configurable timeout).
      * `execute_command()` — Execute commands on the target machine.
      * `synchronize_file_list()` — Bidirectional streaming of file metadata (scan + change journal).
      * `get_chunk_hash()` — Request file hashes for deduplication.
      * `get_chunk()` — Stream data blocks to the `job_worker`.
      * `restore_file()` — Receive and restore files (bidirectional streaming).
      * `close_backup()` — Signal end of backup session.

2. **Authentication (`src/authentification.rs`)**
    * JWT session management with `HashMap<String, Arc<Mutex<ContextData>>>`.
    * Configurable token expiration validation, backup timeout, and maximum session duration.

3. **Snapshot Manager (`src/storage/snapshots/`)**
    * Before any file read operation, the agent attempts to create a consistent snapshot.
    * **Linux (Btrfs)**: `btrfs.rs` driver — Implemented. Creates a read-only snapshot via `btrfs subvolume snapshot`. Optional sudo support.
    * **Windows (VSS)**: Not yet implemented (planned).
    * **Linux (ZFS)**: Not yet implemented (planned).
    * `FileSystemAccessor` (`src/storage/accessor.rs`): Unified abstraction managing path redirections to the mounted snapshot.

4. **File Scanner (`src/scanner/`)**
    * `file_browser.rs`: Recursive directory traversal (with configurable inclusions/exclusions).
    * `file_reader.rs`: Content extraction and chunk hash computation.
    * `file_writer.rs`: File restoration with preservation of permissions and timestamps.
    * `metadata/`: Portable OS metadata abstraction:
      * `unix.rs`: POSIX permissions, ownership, timestamps.
      * `windows.rs`: Windows equivalents.
      * `acl/`: POSIX ACLs (Linux, feature `acl`) and Windows stub.
      * `xattr/`: Extended attributes (Linux, feature `xattr`) and Windows stub.

5. **Server Discovery (`src/resolve/`)**
    * `direct.rs`: Direct connection via IP address/hostname.
    * `mdns.rs`: Automatic discovery via **mDNS** (feature `mdns`, enabled by default).

6. **Automatic Updates (`src/updater.rs`)**
    * Automatic agent update mechanism via the project's GitHub releases.

## Secure Data Flow

When a file stream is requested by the server:

1. The server requests metadata via `synchronize_file_list` (streaming).
2. The server queries hashes with `get_chunk_hash` for deduplication.
3. For chunks absent from the Pool, the server calls `get_chunk`.
4. The agent opens the file on the snapshot (read-only via `FileSystemAccessor`).
5. The agent reads the data in blocks and streams it over the gRPC TLS channel.
6. Compression/decompression is handled server-side when writing to the Pool.

## Installation & Deployment

The agent is statically compiled (for Linux, using `musl`) with no external dependencies.
On Windows, it installs as a Windows Service (`src/winserv.rs`). On Linux, as a Systemd unit.

**Default Cargo features**: `mdns` (network discovery), `acl` (POSIX ACLs), `xattr` (extended attributes).
