# Server Components (`server-rs`)

The `server-rs` crate implements the application logic of the Woodstock server. It is built on the **Tokio** async ecosystem.

## Key Technologies

* **Axum (v0.8)**: Web framework for the REST API.
* **async-graphql**: GraphQL layer (Queries, Mutations, WebSocket Subscriptions).
* **Tonic**: gRPC framework for communication with client agents.
* **Tower**: Middleware for the HTTP service (timeout, tracing, compression).
* **Apalis + apalis-redis**: Asynchronous job queue management (replaces a manual BLPOP).
* **apalis-cron**: CRON scheduling for the scheduler.
* **Redis-rs**: Redis client for distributed locks and progress Pub/Sub.
* **rcgen / rustls**: mTLS certificate generation and validation.
* **utoipa / Swagger UI**: Automatically generated OpenAPI documentation.

## Binaries

### 1. `api_server`

The entry point for humans (via the UI) and third-party integrations.

* **Role**: Serve REST and GraphQL HTTP requests. Also serves the Vue.js frontend.
* **Architecture**:
  * `src/api/routes.rs`: Defines the Axum router with 13+ REST routes under the `/api/` prefix.
  * `src/api/handlers/`: Async functions handling requests (hosts, backups, files, server, metrics).
  * `src/api/services/`: Business orchestration (hosts, backups, files, queue, metrics, certificate, server).
  * `src/graphql/`: Query, Mutation and Subscription resolvers (WebSocket for real-time progress).
  * `ApiServerState`: An `Arc<ApiServerState>` structure injected into every handler via Axum, providing access to all services.
* **Main REST routes**:

| Method | Route | Role |
|--------|-------|------|
| `GET` | `/api/hosts` | List hosts |
| `GET` | `/api/hosts/{name}` | Get host details |
| `GET` | `/api/hosts/{name}/client` | Download client binary |
| `GET/POST` | `/api/hosts/{name}/backups` | List / Trigger a backup |
| `DELETE` | `/api/hosts/{name}/backups/{id}` | Delete a backup |
| `GET` | `/api/hosts/{name}/backups/{id}/files` | Browse files |
| `GET` | `/api/hosts/{name}/backups/{id}/files/download` | ZIP download |
| `GET` | `/metrics` | Prometheus metrics |
| `GET/POST` | `/graphql` | GraphQL API + GraphiQL IDE |
| `/graphql/ws` | WebSocket | GraphQL Subscriptions (progress) |
| `/api-docs` | — | Swagger UI documentation |

### 2. `client_api_server`

The contact point for agent-initiated communications.

* **Role**: Allows agents to register with the server (report their network address).
* **Architecture**: **Axum HTTP server with mTLS** (not gRPC). The client certificate is validated via middleware before every request.
* **Endpoint**: `POST /api/hosts/{name}/client` — Stores the agent IP/port in the Redis cache.
* **Security**: Strict `rustls` configuration requiring a valid client certificate signed by the internal CA.

### 3. `job_worker`

The execution engine. Uses **Apalis** to manage Redis job queues.

* **Role**: Consume pending tasks from 4 distinct Redis queues.
* **Job Queues**:

| Queue | Job Type | Workers |
|-------|----------|---------|
| `schedule` | `ScheduleQueueJob` | Scanner (cron) → dispatches to the backup queue |
| `backup` | `BackupQueueJob::Save` / `::Remove` | `handle_backup()` / `handle_remove()` |
| `interactive` | `RestoreJobData` | `handle_restore()` |
| `maintenance` | `MaintenanceJobData` (Fsck, CleanupRefcnt, Stats) | `handle_fsck()`, `handle_cleanup_refcnt()`, `handle_stats()` |

* **Backup job lifecycle**:
    1. Dequeued from Redis (via Apalis — no manual BLPOP).
    2. gRPC (mTLS) connection to the `ws_client_daemon` agent.
    3. JWT authentication + `SynchronizeFileList` (metadata streaming).
    4. `GetChunkHash` → check existence in Pool.
    5. `GetChunk` (if missing) → write to Pool.
    6. Update the Manifest.
    7. Publish progress via Redis Pub/Sub (`jobs:progress`).

### 4. `scheduler`

The planner.

* **Role**: Two persistent CRON jobs:
  * **`wakeup`** (every 10 min by default): Iterates configured hosts, enqueues `BackupQueueJob::Save` when the scheduling window is reached. Uses `SET NX PX 30s` to prevent duplicates.
  * **`nightly`** (02:30 UTC by default): Enqueues `MaintenanceJobData::CleanupRefcnt` for orphaned chunk cleanup.
* **Note**: It *never* contacts clients directly.

## State Management

Application state is shared via two nested structures:

* **`SharedState`** (`src/shared_state.rs`): Infrastructure common to the `api_server` and `job_worker` binaries.
  * Contains: `Configuration`, `Scheduler`, `Hosts`, `Backups`, `SocketAddrResolver`, `JobUtility`.
* **`ApiServerState`** (`src/api/state.rs`): Extends `SharedState` with REST/GraphQL services.
  * Contains: `hosts_service`, `backups_service`, `files_service`, `queue_service`, `metrics_service`, `certificate_service`, `server_service`, `producers`, `progress_reader`, `redis_client`.
  * Implements `Deref<Target = SharedState>`.

## Real-Time Progress Tracking

The `src/jobs/progress.rs` module manages state publication via **Redis Pub/Sub**:

* `PUBLISH jobs:progress <event>` — Broadcast progress updates.
* `HSET job:progress:{task_id}` — Snapshot of the current state (TTL 24h).
* GraphQL WebSocket Subscriptions (`/graphql/ws`) subscribe to this channel to push updates to the frontend.
