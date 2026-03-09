# Installation

## Overview

Woodstock Backup consists of two main components:

- A **Rust agent** (`ws_client_daemon`) installed on each machine to back up
- A **Rust server** deployed on a central host, composed of four separate services:
  - `api_server` — REST/GraphQL web interface (port 3000)
  - `client_api_server` — mTLS HTTPS gateway for agent communication (port 8443)
  - `job_worker` — Asynchronous backup/restore/maintenance worker
  - `scheduler` — Cron-based scheduler that queues jobs

All server services share the same Docker image and communicate through **Valkey** (a Redis-compatible store).

## Prerequisites

- Sufficient storage space on the server for backups
- For the server: Docker and Docker Compose (recommended) or a Rust toolchain to build from source
- For the agent: A pre-built binary (download from releases) or a Rust toolchain to compile

## Docker Installation (Recommended)

### Standard Configuration

```yaml
services:
  # REST/GraphQL API & Vue.js frontend
  server-api:
    image: phoenix741/woodstock-backup-server:latest
    ports:
      - 3000:3000
    depends_on:
      valkey:
        condition: service_healthy
    environment:
      - REDIS_HOST=valkey
      - REDIS_PORT=6379
      - LOG_LEVEL=info
      - BACKUP_PATH=/backups
      - STATIC_PATH=/app/static
    volumes:
      - "backups_storage:/backups"

  # mTLS HTTPS gateway for agents
  server-client-api:
    image: phoenix741/woodstock-backup-server:latest
    command: ["/app/client_api_server"]
    ports:
      - 8443:8443
    depends_on:
      valkey:
        condition: service_healthy
    environment:
      - REDIS_HOST=valkey
      - REDIS_PORT=6379
      - LOG_LEVEL=info
      - BACKUP_PATH=/backups
    volumes:
      - "backups_storage:/backups"

  # Backup/restore/maintenance job worker
  server-worker:
    image: phoenix741/woodstock-backup-server:latest
    command: ["/app/job_worker"]
    depends_on:
      valkey:
        condition: service_healthy
    environment:
      - REDIS_HOST=valkey
      - REDIS_PORT=6379
      - LOG_LEVEL=info
      - BACKUP_PATH=/backups
      - BACKUP_CONCURRENCY=2
      - RESTORE_CONCURRENCY=8
      - MAINTENANCE_CONCURRENCY=2
    volumes:
      - "backups_storage:/backups"

  # Cron scheduler
  server-scheduler:
    image: phoenix741/woodstock-backup-server:latest
    command: ["/app/scheduler"]
    depends_on:
      valkey:
        condition: service_healthy
    environment:
      - REDIS_HOST=valkey
      - REDIS_PORT=6379
      - LOG_LEVEL=info
      - BACKUP_PATH=/backups
    volumes:
      - "backups_storage:/backups"

  # Valkey (Redis-compatible) – job queue and distributed locks
  valkey:
    image: "valkey/valkey:9-alpine"
    command: valkey-server --save 60 1 --loglevel warning
    ports:
      - "6379:6379"
    volumes:
      - "valkey_data:/data"
    healthcheck:
      test: ["CMD", "valkey-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5

  # Optional: Prometheus monitoring
  prometheus:
    image: prom/prometheus:latest
    volumes:
      - prometheus_storage:/prometheus
      - ./docker/prometheus/prometheus.yml:/etc/prometheus/prometheus.yml
    network_mode: "host"

volumes:
  valkey_data:
  prometheus_storage:
  backups_storage:
    driver: local
    driver_opts:
      type: none
      device: /var/lib/woodstock
      o: bind
```

> **Note**: The `backups_storage` volume maps `/var/lib/woodstock` on the host, which is the default data directory.
> Adjust `device: /var/lib/woodstock` to the path of your choice.

### Prometheus Integration

For monitoring, Prometheus can be configured to collect metrics from the server:

```yaml
scrape_configs:
  - job_name: "woodstock-exporter"
    static_configs:
      - targets: ['localhost:3000']
```

## Manual Installation

### System Requirements

Install the required system dependencies:

```bash
apt install valkey protobuf-compiler cmake make build-essential libacl1-dev libfuse-dev
```

> **Note**: `nodejs` and `npm` are only needed if you want to build the Vue.js frontend from source.
> Pre-built Docker images include the frontend already.

### Build Steps

1. Install Rust:

    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

2. Build the Project:

    ```bash
    # Clone the project
    git clone https://gogs.shadoware.org/ShadowareOrg/woodstock-backup.git woodstock-backup
    cd woodstock-backup

    # Build all server binaries (api_server, client_api_server, job_worker, scheduler, ws_console, ...)
    cargo build --release -p woodstock-server-rs

    # Optional: build the Vue.js frontend
    npm ci
    (cd front && npm run build)
    ```

    Built binaries are placed in `target/release/`.

### Service Configuration (systemd)

Create one systemd service file per component. Assuming the binaries are installed to `/opt/woodstock/bin/`:

#### API Server

```systemd
# /etc/systemd/system/woodstock-api.service
[Unit]
Description=Woodstock API Server
After=network.target valkey.service

[Service]
Type=simple
User=woodstock
ExecStart=/opt/woodstock/bin/api_server
Restart=always
Environment=BACKUP_PATH=/var/lib/woodstock
Environment=STATIC_PATH=/opt/woodstock/static
Environment=REDIS_HOST=localhost
Environment=REDIS_PORT=6379

[Install]
WantedBy=multi-user.target
```

#### Client API Server

```systemd
# /etc/systemd/system/woodstock-client-api.service
[Unit]
Description=Woodstock Client API Server (mTLS gateway for agents)
After=network.target valkey.service

[Service]
Type=simple
User=woodstock
ExecStart=/opt/woodstock/bin/client_api_server
Restart=always
Environment=BACKUP_PATH=/var/lib/woodstock
Environment=REDIS_HOST=localhost
Environment=REDIS_PORT=6379

[Install]
WantedBy=multi-user.target
```

#### Job Worker

```systemd
# /etc/systemd/system/woodstock-worker.service
[Unit]
Description=Woodstock Job Worker
After=network.target valkey.service

[Service]
Type=simple
User=woodstock
ExecStart=/opt/woodstock/bin/job_worker
Restart=always
Environment=BACKUP_PATH=/var/lib/woodstock
Environment=REDIS_HOST=localhost
Environment=REDIS_PORT=6379
Environment=BACKUP_CONCURRENCY=2
Environment=RESTORE_CONCURRENCY=8

[Install]
WantedBy=multi-user.target
```

#### Scheduler

```systemd
# /etc/systemd/system/woodstock-scheduler.service
[Unit]
Description=Woodstock Scheduler
After=network.target valkey.service

[Service]
Type=simple
User=woodstock
ExecStart=/opt/woodstock/bin/scheduler
Restart=always
Environment=BACKUP_PATH=/var/lib/woodstock
Environment=REDIS_HOST=localhost
Environment=REDIS_PORT=6379

[Install]
WantedBy=multi-user.target
```

#### Enable and start the services

```bash
sudo systemctl enable woodstock-api.service woodstock-client-api.service woodstock-worker.service woodstock-scheduler.service
sudo systemctl start  woodstock-api.service woodstock-client-api.service woodstock-worker.service woodstock-scheduler.service
```

## Configuration Reference

### Core Environment Variables

| Environment Variable      | Default Value                            | Description                                                           |
|---------------------------|------------------------------------------|-----------------------------------------------------------------------|
| LOG_LEVEL                 | `info`                                   | The level of log to display (error, warn, info, debug, trace)         |
| CHUNK_ALGORITHM           | `blake3`                                 | The hash algorithm for chunks (blake3, sha2_256, sha3_256)            |

### Paths Environment Variables

| Environment Variable      | Default Value                            | Description                                                           |
|---------------------------|------------------------------------------|-----------------------------------------------------------------------|
| STATIC_PATH               | -                                        | The path where the vuetify client will be served                      |
| BACKUP_PATH               | `/var/lib/woodstock`                     | The path where the backup will be stored                              |
| CERTIFICATES_PATH         | `$BACKUP_PATH/certs`                     | The path where the certificates will be stored                        |
| CONFIG_PATH               | `$BACKUP_PATH/config`                    | The path where the configuration of devices will be stored            |
| HOSTS_PATH                | `$BACKUP_PATH/hosts`                     | The path where the file list of each device will be stored            |
| LOGS_PATH                 | `$BACKUP_PATH/logs`                      | The path where the logs will be stored                                |
| POOL_PATH                 | `$BACKUP_PATH/pool`                      | The path where the pool of backup will be stored                      |
| JOBS_PATH                 | `$LOGS_PATH/jobs`                        | The path where logs of jobs will be stored                            |
| EVENTS_PATH               | `$BACKUP_PATH/events`                    | The path where event data will be stored                              |

### Valkey/Redis Environment Variables

| Environment Variable      | Default Value                            | Description                                                           |
|---------------------------|------------------------------------------|-----------------------------------------------------------------------|
| REDIS_HOST                | `localhost`                              | The host of Valkey/Redis to connect                                   |
| REDIS_PORT                | `6379`                                   | The port of Valkey/Redis to connect                                   |

### File view Environment Variables

| Environment Variable      | Default Value                            | Description                                                           |
|---------------------------|------------------------------------------|-----------------------------------------------------------------------|
| CACHE_TTL                 | `24h`                                    | The time to live of the cache, where the config of devices are cached |
| CACHE_SIZE                | `10`                                     | The size of the cache for items like file manifests                   |
| FILE_VIEW_MAX_ELEMENTS    | `10`                                     | The number of backups file list to store in the cache                 |
| FILE_VIEW_TTL_CACHE       | `15min`                                  | The time to live of the cache of the file list                        |

### API Server Environment Variables

| Environment Variable      | Default Value | Description                                                       |
|---------------------------|---------------|-------------------------------------------------------------------|
| MANAGEMENT_API_LISTEN     | `0.0.0.0`     | Bind address for the REST/GraphQL API server                      |
| MANAGEMENT_API_PORT       | `3000`        | Listening port for the REST/GraphQL API server                    |

### Client API Server Environment Variables

| Environment Variable      | Default Value | Description                                                       |
|---------------------------|---------------|-------------------------------------------------------------------|
| CLIENT_API_LISTEN         | `0.0.0.0`     | Bind address for the mTLS HTTPS agent gateway                     |
| CLIENT_API_PORT           | `8443`        | Listening port for the mTLS HTTPS agent gateway                   |

### Job Worker Environment Variables

| Environment Variable      | Default Value | Description                                                       |
|---------------------------|---------------|-------------------------------------------------------------------|
| BACKUP_CONCURRENCY        | `2`           | Number of backup jobs to run in parallel                          |
| RESTORE_CONCURRENCY       | `8`           | Number of restore jobs to run in parallel                         |
| MAINTENANCE_CONCURRENCY   | `2`           | Number of maintenance jobs (fsck, cleanup) to run in parallel     |
| PROGRESS_SNAPSHOT_TTL     | `86400`       | Duration in seconds to keep progress snapshots in Redis           |
| HOST_LOCK_TTL_MS          | `60000`       | Host-level lock TTL in milliseconds (prevents concurrent backups) |

## Used ports

| Port | Protocol | Description                                                                                                      | Required |
|------|----------|------------------------------------------------------------------------------------------------------------------|----------|
| 3000 | TCP      | HTTP API (Management interface) - Used to manage backups, view them, start/delete tasks and monitor the system   | Yes      |
| 8443 | TCP/TLS  | HTTPS Client API - Used by client agents to authenticate and report their presence to the server (priority over mDNS) | Yes      |
| 3657 | TCP      | Default listening port on client agents to receive instructions from the server                                  | Yes      |
| 5353 | UDP      | mDNS Discovery - Used to automatically discover clients on the local network (alternative method)                | No       |
| 9090 | TCP      | Prometheus - For collecting and visualizing monitoring metrics                                                   | No       |
| 6379 | TCP      | Valkey - Storage of temporary data, queues and communication between components                                  | Yes      |
