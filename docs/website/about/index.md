# About Woodstock Backup

**Woodstock Backup** is a centralized backup solution designed to efficiently back up multiple hosts to a central server. With advanced deduplication, storage space is optimized.

## Architecture and Technologies

Woodstock Backup combines several modern technologies to offer a robust backup solution:

- **Client-server architecture**: The server contacts clients to initiate backups, simplifying configuration and enhancing security
- **Technologies**: Primarily developed in JavaScript (NodeJS for the server and Vue for the interface), with critical components in Rust for performance
- **Optimized storage**: Uses a sophisticated chunk-based deduplication system with Blake3 hashing, organizing data in a hierarchical pool structure
- **Security**: Mutual TLS (mTLS) authentication and JWT tokens to secure communications

Woodstock Backup is open-source software distributed under the MIT license.

## Key Features

- **Modern interface** with accessible API to launch, manage and monitor backups
- **Native Windows compatibility** without requiring rsync or SSH, works natively with all operating systems
- **Block-based deduplication**: Files are divided into 16 MB blocks (chunks) to optimize storage
- **Cryptographic hashing system**: Using Blake3 to identify and verify the integrity of data blocks
- **FUSE filesystem access** allowing you to mount any backup point as a normal directory
- **Support for extended attributes and ACLs** to preserve all file metadata
- **Backup of computers with dynamic IP** or intermittently connected to the network

## Technical Architecture

### Storage Pool Structure

The storage uses a "pool" system organized based on data block hashes:

- Chunks (blocks) are identified by their Blake3 hash
- Three-level hierarchical organization to optimize access
- Reference counting system to track block usage
- Optimized binary format (Protocol Buffers) for backup manifests

### Authentication and Security

Security is based on multiple layers:

- **mTLS certificates** generated for each client device
- **JWT tokens** for application-level authentication
- **Integrity verification** based on cryptographic hashes

## Why Woodstock Backup?

Woodstock Backup was developed to solve two major limitations of traditional backup solutions:

- **Simplified Windows backups** without dependency on rsync or SSH
- **Easy access to backup data** through FUSE filesystem mounting

## Screenshots

Here are some screenshots of the Woodstock Backup web interface that provide an overview of the different features.

![Devices Light](../images/devices.png =640x)
----

![Pool Light](../images/pool.png =640x)
----

![Tasks Light](../images/tasks.png =640x)
----

![Tasks Dark](../images/tasks_dark.png =640x)
