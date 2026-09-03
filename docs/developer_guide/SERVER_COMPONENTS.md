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
| `backup` | `BackupQueueJob::Save` / `::Remove` | `handle_backup()` / `handle_remove()` |
| `interactive` | `RestoreJobData` | `handle_restore()` |
| `maintenance` | `MaintenanceJobData` (Fsck, CleanupRefcnt, Stats) | `handle_fsck()`, `handle_cleanup_refcnt()`, `handle_stats()` |
| `archive` | `ArchiveJobData::Run` | `handle_archive_run()` — see [Periodic Archiving](ARCHIVING.md) |

* **Backup job lifecycle**:
    1. Dequeued from Redis (via Apalis — no manual BLPOP).
    2. gRPC (mTLS) connection to the `ws_client_daemon` agent.
    3. JWT authentication + `SynchronizeFileList` (metadata streaming).
    4. `GetChunkHash` → check existence in Pool.
    5. `GetChunk` (if missing) → write to Pool.
    6. Update the Manifest.
    7. Publish progress via Redis Pub/Sub (`jobs:progress`).

### 4. `scheduler`

The planner. Must run as a single instance — see the module-level doc comment in
`server-rs/src/bin/scheduler.rs`.

* **Role**: A single unified dynamic-wakeup scanner plus an event-driven subscriber — no
  Apalis `Monitor`/`CronPipe` involved at all, unlike `job_worker`:
  * **Dynamic-wakeup scanner** (`run_scanner_loop`): each iteration runs the scheduling
    decision for every host, every enabled archive profile, and the nightly maintenance
    trigger, then sleeps until the next real deadline across all three
    (`compute_next_wakeup`) instead of ticking on a fixed interval.
    * *Hosts*: the shared decision function `try_schedule_host`
      (`server-rs/src/jobs/decision.rs`) — in order: cooldown → already running → due for
      backup → blackout window → pool fsck lock → reachable → `enqueue_backup_unique`
      (still `SET NX PX 30s` to prevent duplicates). Every refusal/success is recorded in a
      per-host Redis key (`next-attempt`), which both backs off that host and feeds
      `compute_next_wakeup`'s sleep calculation. Backoff durations and the anti-busy-poll
      floor are configurable (`retryBackoffAfterSuccessSecs`/`retryBackoffOnRefusalSecs`/
      `wakeupFloorSecs` in `scheduler.yml`, defaults 5 min/15 min/30s) via
      `SchedulingConfig::from_application_scheduler`, threaded into `try_schedule_host` and
      `compute_next_wakeup` instead of hardcoded constants.
    * *Archive profiles*: `check_and_enqueue_due_archives` checks each `archiving.yml`
      profile's own `schedule_cron` against its persisted `ArchiveRunStatus.last_run` via
      `next_due_at` (`woodstock-rs/src/utils/cron_due.rs`), and fans out
      `ArchiveJobData::Run` jobs for due ones — see [Periodic Archiving](ARCHIVING.md).
    * *Nightly*: `check_and_enqueue_nightly` checks `nightlySchedule` against a Redis-persisted
      last-run timestamp (same `next_due_at` mechanism) and enqueues
      `MaintenanceJobData::CleanupRefcnt` for orphaned chunk cleanup when due.
    * The sleep is bounded by a floor (anti busy-poll) and a global safety-net ceiling — the
      old `wakeupSchedule` cron, now applied once across all three categories to catch config
      changes (new host, re-activated schedule, shortened `backupPeriod`, an added/edited
      archive profile, an edited `nightlySchedule`) that fall between two computed due dates,
      rather than to poll any of them, which real due dates already drive.
  * **Event-driven subscriber** (`run_host_online_subscriber`): subscribes to the Redis
    Pub/Sub channel `HOST_ONLINE_CHANNEL`, published by `SocketAddrResolver::register_service`
    on a genuine offline→online transition (not every heartbeat). On receipt it runs the same
    `try_schedule_host`, bypassing the cooldown gate, so a host that just came back online is
    backed up immediately instead of waiting for the next scan.
* **Blackout windows**: `Schedule.blackout` (per-host, falling back to
  `ApplicationScheduler.default_schedule.blackout`) defines recurring time ranges during
  which `try_schedule_host` refuses to start a new backup, unless
  `blackout_override_after_periods` lets an overdue host through. Pure evaluation logic lives
  in `woodstock-rs/src/config/blackout.rs`; the gate itself is
  `JobUtility::is_in_blackout_now`. Re-checked at job execution time too
  (`server-rs/src/jobs/workers.rs::handle_backup`), since a queued job can outlive the
  blackout-free moment it was enqueued in.
* The two loops are raced against each other via `tokio::select!` in `main()`, so a panic in
  either is fatal to the whole process instead of silently killing the scheduler.
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
