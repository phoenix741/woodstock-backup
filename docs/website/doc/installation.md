# Installation

## Overview

Woodstock Backup consists of two main components:

- A Rust-based agent installed on client machines
- A Node.js server application deployed on a central host

## Prerequisites

- Sufficient storage space on the server for backups
- For the server: Docker (recommended) or Node.js environment
- For the agent: Rust runtime environment

## Docker Installation (Recommended)

### Standard Configuration

```yaml
services:
  website:
    build:
      context: ./docs/website
      dockerfile: Dockerfile
    image: phoenix741/woodstock-backup-website:develop
    ports:
      - 8080:80
  woodstock:
    build:
      context: ./
      dockerfile: Dockerfile
      args:
        - RUST_VERSION=1
        - NODE_VERSION=20-slim
        - DEBIAN_VERSION=debian:12-slim
    image: phoenix741/woodstock-backup:develop
    ports:
      - 3000:3000
    depends_on:
      - redis
    environment:
      - REDIS_HOST=redis
      - REDIS_PORT=6379
      - LOG_LEVEL=warn
      - NODE_ENV=production
      - BACKUP_PATH=/backups
      - BACKUP_WORKER_INSTANCES=3
      - CLIENT_API_HOSTNAME=myserver.localdomain.com
      - CLIENT_API_PORT=8443
    volumes:
      - "backups_storage:/backups"

  woodstock_client:
    build:
      context: ./
      dockerfile: Dockerfile
      args:
        - RUST_VERSION=1
        - NODE_VERSION=20-slim
        - DEBIAN_VERSION=debian:12-slim
      target: client
    image: phoenix741/woodstock-backup-client:develop
    ports:
      - 3657:3657
    environment:
      - LOG_LEVEL=debug
    volumes:
      - "client_storage:/etc/woodstock"

  redis:
    image: "bitnami/redis:7.4"
    environment:
      - ALLOW_EMPTY_PASSWORD=yes
      - REDIS_DISABLE_COMMANDS=FLUSHDB,FLUSHALL
    ports:
      - "6379:6379"
    volumes:
      - "redis_data:/bitnami/redis/data"

  prometheus:
    image: bitnami/prometheus:2
    volumes:
      - prometheus_storage:/opt/bitnami/prometheus/data
      - ./docker/prometheus/prometheus.yml:/opt/bitnami/prometheus/conf/prometheus.yml
    network_mode: "host"

volumes:
  redis_data:
  prometheus_storage:
  client_storage:
  backups_storage:
    driver: local
    driver_opts:
      type: none
      device: /var/lib/woodstock
      o: bind
```

### Standard Configuration (with mDNS)

### Alternative Configuration (without mDNS)

If you prefer not to use privileged mode, you can bind the server to port 3000:

```yaml
services:
  woodstock:
    image: phoenix741/woodstock-backup:2
    ports:
      - "3000:3000"
    networks:
      - woodstock
    environment:
      - REDIS_HOST=redis
      - REDIS_PORT=6379
      - LOG_LEVEL=warn
      - NODE_ENV=production
      - BACKUP_PATH=/backups
      - BACKUP_WORKER_INSTANCES=3
    volumes:
      - "backups_storage:/backups"

  redis:
    image: "bitnami/redis:7.2"
    environment:
      - ALLOW_EMPTY_PASSWORD=yes
      - REDIS_DISABLE_COMMANDS=FLUSHDB,FLUSHALL
    networks:
      - woodstock
    volumes:
      - "redis_data:/bitnami/redis/data"

  prometheus:
    image: bitnami/prometheus:2
    volumes:
      - prometheus_storage:/opt/bitnami/prometheus/data
      - ./docker/prometheus/prometheus.yml:/opt/bitnami/prometheus/conf/prometheus.yml
    ports:
      - "9090:9090"

networks:
  woodstock:

volumes:
  redis_data:
  prometheus_storage:
  backups_storage:
    driver: local
    driver_opts:
      type: none
      device: /var/lib/woodstock
      o: bind
```

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
apt install redis nodejs protobuf-compiler cmake make build-essential git-lfs libacl1-dev libfuse-dev
```

### Build Steps

1. Install Rust:

    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

2. Build the Project:

    ```bash
    # Clone the project
    git clone https://gogs.shadoware.org/ShadowareOrg/woodstock-backup.git woodstock-backup
    
    # Build the project
    cargo build --release

    # Install and build nodejs dependencies
    npm ci
    (cd shared-rs && npm run build)
    (cd nestjs && npm run buildall)
    (cd front && npm run build)
    ```

### Service Configuration (pm2)

You can run the server using PM2.

```bash
# You can use pm2 and the ecosystem.config.js file to run the server
pm2 startup
```

With the following `ecosystem.config.js` configuration:

```js
module.exports = [
  {
    script: 'apps/api/main.js',
    name: 'api',
    cwd: '/app/nestjs',
    exec_mode: 'cluster',
    instances: parseInt(process.env.API_INSTANCES ?? '1'),
  },
  {
    script: 'apps/clientApi/main.js',
    name: 'clientApi',
    cwd: '/app/nestjs',
    exec_mode: 'cluster',
    instances: parseInt(process.env.API_INSTANCES ?? '1'),
  },
  {
    script: 'apps/backupWorker/main.js',
    name: 'backupWorker',
    cwd: '/app/nestjs',
    instances: parseInt(process.env.BACKUP_WORKER_INSTANCES || '1'),
    env: {
      MAX_BACKUP_TASK: 1,
    },
  },
  {
    script: 'apps/refcntWorker/main.js',
    name: 'refcntWorker',
    cwd: '/app/nestjs',
    instances: process.env.DISABLE_REFCNT === 'true' ? 0 : 1,
  },
  {
    script: 'apps/scheduleWorker/main.js',
    name: 'scheduleWorker',
    cwd: '/app/nestjs',
    instances: process.env.DISABLE_SCHEDULER === 'true' ? 0 : 1,
  },
];
```

### Service Configuration (systemd)

As an alternative to PM2, you can use systemd to manage the services. Create the following service files:

#### API Service

```systemd
# /etc/systemd/system/woodstock-api.service
[Unit]
Description=Woodstock API Service
After=network.target redis.service

[Service]
Type=simple
User=woodstock
WorkingDirectory=/app/nestjs
ExecStart=/usr/bin/node apps/api/main.js
Restart=always
Environment=NODE_ENV=production

[Install]
WantedBy=multi-user.target
```

#### Client API Service

```systemd
# /etc/systemd/system/woodstock-client-api.service
[Unit]
Description=Woodstock Client API Service
After=network.target redis.service

[Service]
Type=simple
User=woodstock
WorkingDirectory=/app/nestjs
ExecStart=/usr/bin/node apps/clientApi/main.js
Restart=always
Environment=NODE_ENV=production

[Install]
WantedBy=multi-user.target
```

#### Backup Worker Service

```systemd
# /etc/systemd/system/woodstock-backup-worker.service
[Unit]
Description=Woodstock Backup Worker Service
After=network.target redis.service woodstock-api.service

[Service]
Type=simple
User=woodstock
WorkingDirectory=/app/nestjs
ExecStart=/usr/bin/node apps/backupWorker/main.js
Restart=always
Environment=NODE_ENV=production
Environment=MAX_BACKUP_TASK=1

[Install]
WantedBy=multi-user.target
```

#### Reference Count Worker Service

```systemd
# /etc/systemd/system/woodstock-refcnt-worker.service
[Unit]
Description=Woodstock Reference Count Worker Service
After=network.target redis.service woodstock-api.service

[Service]
Type=simple
User=woodstock
WorkingDirectory=/app/nestjs
ExecStart=/usr/bin/node apps/refcntWorker/main.js
Restart=always
Environment=NODE_ENV=production

[Install]
WantedBy=multi-user.target
```

#### Schedule Worker Service

```systemd
# /etc/systemd/system/woodstock-schedule-worker.service
[Unit]
Description=Woodstock Schedule Worker Service
After=network.target redis.service woodstock-api.service

[Service]
Type=simple
User=woodstock
WorkingDirectory=/app/nestjs
ExecStart=/usr/bin/node apps/scheduleWorker/main.js
Restart=always
Environment=NODE_ENV=production

[Install]
WantedBy=multi-user.target
```

#### Enable and start the services

```bash
systemctl enable woodstock-api.service
systemctl enable woodstock-client-api.service
systemctl enable woodstock-backup-worker.service
systemctl enable woodstock-refcnt-worker.service
systemctl enable woodstock-schedule-worker.service

systemctl start woodstock-api.service
systemctl start woodstock-client-api.service
systemctl start woodstock-backup-worker.service
systemctl start woodstock-refcnt-worker.service
systemctl start woodstock-schedule-worker.service
```

## Configuration Reference

### Core Environment Variables

| Environment Variable      | Default Value                            | Description                                                           |
|---------------------------|------------------------------------------|-----------------------------------------------------------------------|
| STATIC_PATH               | -                                        | The path where the vuetify client will be served                      |
| BACKUP_PATH               | `/var/lib/woodstock`                     | The path where the backup will be stored                              |
| CERTIFICATES_PATH         | `/etc/woodstock/certs`                   | The path where the certificates will be stored                        |
| CONFIG_PATH               | `/etc/woodstock/config`                  | The path where the configuration of devices will be stored            |
| HOSTS_PATH                | `/etc/woodstock/hosts`                   | The path where the file list of each device will be stored            |
| LOGS_PATH                 | `/var/log/woodstock/logs`                | The path where the logs will be stored                                |
| POOL_PATH                 | `/var/lib/woodstock/pool`                | The path where the pool of backup will be stored                      |
| JOBS_PATH                 | `/var/lib/woodstock/jobs`                | The path where logs of jobs will be stored                            |
| REDIS_HOST                | -                                        | The host of redis to connect                                          |
| REDIS_PORT                | -                                        | The port of redis to connect                                          |
| CACHE_TTL                 | `24h`                                    | The time to live of the cache, where the config of devices are cached |
| LOG_LEVEL                 | `info`                                   | The level of log to display                                           |
| FILE_VIEW_MAX_ELEMENTS    | `10`                                     | The number of backups file list to store in the cache                 |
| FILE_VIEW_TTL_CACHE       | `15min`                                  | The time to live of the cache of the file list                        |

### Docker-Specific Environment Variables

| Environment Variable      | Default Value                            | Description                                                           |
|---------------------------|------------------------------------------|-----------------------------------------------------------------------|
| API_INSTANCES             | `1`                                      | The number of instance of the api to run                              |
| BACKUP_WORKER_INSTANCES   | `1`                                      | The number of instance of the backup worker to run                    |
| DISABLE_REFCNT            | `false`                                  | Disable the refcnt of the backup (to be run on another container)     |
| DISABLE_SCHEDULER         | `false`                                  | Disable the scheduler of the backup (to be run on another container)  |
| MAX_BACKUP_TASK           | `2`                                      | The number of backup task to run in parallel in each worker instance  |

## Used ports

| Port | Protocol | Description       | Required |
|------|----------|-------------------|----------|
| 3000 | TCP      | HTTP API          | Yes      |
| 5353 | UDP      | mDNS Discovery    | No       |
| 9090 | TCP      | Prometheus        | No       |
| 6379 | TCP      | Redis             | Yes      |
