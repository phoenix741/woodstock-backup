# Contributing to Woodstock Backup

Contributions are welcome! Before you start, please read the guidelines below.

## Scope

Contributions must align with the goals of this backup application. Features unrelated to backup, storage, or administration will not be accepted (e.g., a feature that "makes coffee during a backup" would be declined).

If you want to add a new feature, **open an issue first** to check whether it has already been started and whether a pull request would be accepted. This saves everyone time.

## Tech Stack

- **Backend**: Rust (stable toolchain). All binaries in `server-rs/`, `client-rs/`, `cli-rs/`, `woodstock-rs/`.
- **Frontend**: Vue 3 + Vuetify + Apollo GraphQL (TypeScript). Located in `front/`.
- **Storage**: Filesystem (CAS pool + Protobuf manifests) + Valkey/Redis (job queues and distributed locks).
- **Communication**: gRPC over mTLS (server ↔ agent), REST/GraphQL (server ↔ frontend).

## Building

```bash
# Build all server components
cargo build -p woodstock-server-rs

# Build the agent
cargo build -p woodstock-client

# Build the CLI tools
cargo build -p woodstock-cli-rs

# Run unit tests
cargo test

# Run integration tests (requires Docker)
cargo test -p e2e-tests
```

## Code Style

- Follow idiomatic Rust (`cargo clippy`, `cargo fmt`).
- Use `eyre::Result` for binary/handler errors, `thiserror` for library errors.
- Use `tracing` for all logging (no `println!` or `eprintln!`).
- Async/await everywhere; `tokio` runtime.

## Submitting Changes

1. Fork the repository.
2. Create a branch from `develop` (the development branch).
3. Make your changes and add tests where appropriate.
4. Open a pull request. The maintainer will review and may request changes.

## Governance

This project currently follows a BDFL (Benevolent Dictator for Life) model. The maintainer may accept or decline contributions based on fit with the project vision and code quality. This may change if the project receives significant community contributions.
