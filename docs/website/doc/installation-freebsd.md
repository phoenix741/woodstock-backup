# Installation on FreeBSD

This guide describes how to install Woodstock Backup server on FreeBSD using the official `.pkg` packages.

::: info FreeBSD vs Linux Differences
On FreeBSD, path conventions differ slightly from Linux:

- Data: `/var/db/woodstock/` (instead of `/var/lib/woodstock/`)
- Binaries: `/usr/local/bin/` (instead of `/usr/bin/`)
- Config: `/usr/local/etc/woodstock/` (instead of `/etc/woodstock/`)
- Services: rc.d (instead of systemd)
:::

## Prerequisites

- FreeBSD 15.x — packages declare the ABI of the version they were built for
  (`freebsd:15:x86:64`). `pkg` refuses to install packages with mismatched ABI:
  verify with `pkg config ABI` that your system matches the package you downloaded.
- `root` access
- **Valkey** (or Redis) installed and running
- Sufficient disk space for backups

## 1. Configure Package Repository

Woodstock packages are distributed via Gitea's generic registry. Download the package directly from releases:

```bash
# Get the latest available version (replace X.Y.Z with desired version)
WOODSTOCK_VERSION="X.Y.Z"
fetch -o /tmp/woodstock-server-${WOODSTOCK_VERSION}.pkg \
  "https://gogs.shadoware.org/api/packages/ShadowareOrg/generic/woodstock-freebsd/${WOODSTOCK_VERSION}/woodstock-server-${WOODSTOCK_VERSION}.pkg"
```

## 2. Install the Server

```bash
pkg install /tmp/woodstock-server-${WOODSTOCK_VERSION}.pkg
```

This installs:

- 4 server binaries in `/usr/local/bin/`
- Admin tools `ws_console`, `ws_restore`, and `ws_sync` in
  `/usr/local/bin/` (equivalent to `woodstock-cli` package on Debian)
- Vue.js web interface in `/usr/local/share/woodstock/static/`
- 4 rc.d scripts in `/usr/local/etc/rc.d/`
- Example configuration file `/usr/local/etc/woodstock/server.env.sample`
- System user `woodstock` (UID 565, non-root)
- Data directories in `/var/db/woodstock/`

## 3. Install Valkey

Woodstock server requires **Valkey** (Redis-compatible) for job queue and distributed locks:

```bash
pkg install databases/valkey
```

Enable and start Valkey:

```bash
echo 'valkey_enable="YES"' >> /etc/rc.conf
service valkey start
```

Verify Valkey is operational:

```bash
valkey-cli ping
# Expected response: PONG
```

## 4. Configure the Server

Create the configuration file from the provided example:

```bash
cp /usr/local/etc/woodstock/server.env.sample /usr/local/etc/woodstock/server.env
chmod 640 /usr/local/etc/woodstock/server.env
chown root:woodstock /usr/local/etc/woodstock/server.env
```

Edit the file:

```bash
vi /usr/local/etc/woodstock/server.env
```

Essential parameters:

```ini
# Path to backup data (FreeBSD convention)
BACKUP_PATH=/var/db/woodstock

# Valkey/Redis connection
REDIS_HOST=localhost
REDIS_PORT=6379

# Pre-compiled web interface
STATIC_PATH=/usr/local/share/woodstock/static

# Log level: error, warn, info, debug
LOG_LEVEL=info

# REST API port (default: 3000)
MANAGEMENT_API_PORT=3000

# Worker concurrency
BACKUP_CONCURRENCY=2
RESTORE_CONCURRENCY=8
MAINTENANCE_CONCURRENCY=2
```

::: tip Disk Space
If your backup data must reside on a different disk, mount it on `/var/db/woodstock` or modify `BACKUP_PATH`.
:::

## 5. Enable and Start Services

Add the 4 services to `/etc/rc.conf`:

```bash
# Enable all Woodstock services
sysrc woodstock_worker_enable="YES"
sysrc woodstock_scheduler_enable="YES"
sysrc woodstock_api_enable="YES"
sysrc woodstock_client_api_enable="YES"
```

Start services in the recommended order (worker and scheduler first):

```bash
service woodstock_worker start
service woodstock_scheduler start
service woodstock_api start
service woodstock_client_api start
```

The 4 available services:

| rc.d Service | Role | Port |
|--------------|------|------|
| `woodstock_api` | REST API + web interface | 3000 |
| `woodstock_client_api` | mTLS gateway for agents | 8443 |
| `woodstock_worker` | Backup/restore worker | — |
| `woodstock_scheduler` | CRON scheduler | — |

## 6. Verify Installation

```bash
# Check service status
service woodstock_api status
service woodstock_worker status

# Access web interface
fetch -o - http://localhost:3000/ | head -5

# Check logs
tail -f /var/log/woodstock/*.log
```

The web interface is available at `http://<server-address>:3000`.

## 7. Install Client on Machines to be Backed Up

```bash
WOODSTOCK_VERSION="X.Y.Z"
fetch -o /tmp/woodstock-client-${WOODSTOCK_VERSION}.pkg \
  "https://gogs.shadoware.org/api/packages/ShadowareOrg/generic/woodstock-freebsd/${WOODSTOCK_VERSION}/woodstock-client-${WOODSTOCK_VERSION}.pkg"

pkg install /tmp/woodstock-client-${WOODSTOCK_VERSION}.pkg
```

Then configure the client:

```bash
cp /usr/local/etc/woodstock/config.yaml.sample /usr/local/etc/woodstock/config.yaml
vi /usr/local/etc/woodstock/config.yaml

# Enable and start client service
sysrc woodstock_client_enable="YES"
service woodstock_client start
```

See the [agent configuration guide](/doc/agent) for details.

## Firewall (pf)

If you use `pf`, add the following rules to `/etc/pf.conf`:

```pf
# Woodstock web interface (HTTPS recommended via reverse proxy)
pass in on em0 proto tcp to port 3000

# mTLS gateway for agents (required)
pass in on em0 proto tcp to port 8443
```

Then reload the rules:

```bash
pfctl -f /etc/pf.conf
```

## Data Structure on FreeBSD

```
/var/db/woodstock/
├── certs/          # mTLS certificates
├── config/         # YAML configuration files
├── hosts/          # Data per backed-up machine
├── logs/           # Application logs
├── pool/           # CAS storage (deduplicated chunks)
├── events/         # Audit log
└── jobs/           # Job state

/usr/local/etc/woodstock/
└── server.env      # Server configuration

/usr/local/share/woodstock/
└── static/         # Vue.js web interface
```

## Upgrade

```bash
# Download the new version
WOODSTOCK_VERSION="X.Y.Z"
fetch -o /tmp/woodstock-server-${WOODSTOCK_VERSION}.pkg \
  "https://gogs.shadoware.org/api/packages/ShadowareOrg/generic/woodstock-freebsd/${WOODSTOCK_VERSION}/woodstock-server-${WOODSTOCK_VERSION}.pkg"

# Upgrade (services auto-stopped by pre-deinstall)
pkg upgrade /tmp/woodstock-server-${WOODSTOCK_VERSION}.pkg

# Restart services
service woodstock_worker start
service woodstock_scheduler start
service woodstock_api start
service woodstock_client_api start
```

## Uninstallation

```bash
# Remove package (keeps data in /var/db/woodstock)
pkg delete woodstock-server
```

::: warning Data Retention
The FreeBSD package does **not** automatically remove `/var/db/woodstock` when uninstalling. Manually delete this directory if you want to erase all backups:

```bash
rm -rf /var/db/woodstock
```

:::
