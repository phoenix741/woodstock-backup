# About the shared-rs crate

- Provides shared Rust code (models, config, events, utils, services) for all Woodstock Backup components.
- Exposes a native Node.js binding (via .node, .js, .d.ts, npm/ for multi-platform).
- Used for cross-language integration and code reuse.
- All comments, logs, and user-facing text MUST be in English.

# Woodstock Backup Project - Copilot Instructions

- The project is split into several Rust crates: woodstock-rs (core), client-rs (daemon), cli-rs (CLI), backuppc-importer-rs (import tool), shared-rs (shared code).
- Architecture: client-server, with a deduplicated pool and verification system using a state machine.
- Use Tokio for async, channels for progress, and always release locks before awaiting async calls.
- Progress reporting must be unified and clear for all verification steps.
- Error handling: use custom error types and propagate errors properly.
- Logging: use structured logs with appropriate levels.
- Testing: unit and integration tests are required for critical code.
- All code, comments, logs, and user-facing text must be in English. Translate any non-English text.
- Keep this file short and focused. Remove outdated or overly detailed content.

# JavaScript/Node.js Integration Guidelines

- JavaScript and TypeScript code must follow modern standards (ESLint, Prettier, strict TypeScript typing).
- Use the native Node.js bindings provided by shared-rs (`.node`, `.js`, `.d.ts`) for cross-language integration.
- Front-end (Vue.js) and back-end (NestJS) applications must consume shared models/types via the generated TypeScript bindings.
- All logs, error messages, and user-facing text must be in English.
- Unit and integration tests are required for critical modules (use Jest for Node.js/NestJS, Vitest or Jest for front-end).
- Configuration should be centralized and typed (use `.env` files, config modules, and schema validation).
- Integration with shared Rust services must be tested (e.g., Node.js ↔ Rust integration tests).
- Document npm scripts and commands in the respective README files.
- Maintain clear separation of concerns (no business logic in NestJS controllers or Vue components).
- Use explicit imports and avoid complex relative paths (prefer TypeScript path aliases).

---

_This instruction document was created on May 10, 2025 for the Woodstock Backup project._
