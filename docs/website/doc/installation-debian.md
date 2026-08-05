# Installation via Debian Package

This guide describes how to install Woodstock Backup server on a Debian/Ubuntu system using the official `.deb` package.

## Prerequisites

- Debian 13 (Trixie) or Ubuntu 24.04 LTS
- `root` or `sudo` access
- **Valkey** (or Redis) installed and running
- Sufficient disk space for backups

## 1. Configure the Gitea Repository

Add the Woodstock Debian repository:

```bash
# Add the GPG key
curl -fsSL https://gogs.shadoware.org/api/packages/ShadowareOrg/debian/repository.key \
  | gpg --dearmor -o /usr/share/keyrings/woodstock-archive-keyring.gpg

# Add the repository
echo "deb [signed-by=/usr/share/keyrings/woodstock-archive-keyring.gpg] \
  https://gogs.shadoware.org/api/packages/ShadowareOrg/debian trixie main" \
  | tee /etc/apt/sources.list.d/woodstock.list

apt-get update
```

## 2. Install the Server

```bash
apt-get install woodstock-server
```

This installs:

- 4 server binaries: `api_server`, `client_api_server`, `job_worker`, `scheduler`
- Vue.js web interface in `/usr/share/woodstock/static/`
- 4 systemd services + 1 target `woodstock.target`
- Configuration file `/etc/woodstock/server.env`
- System user `woodstock` (dedicated UID, non-root)
- Data directory `/var/lib/woodstock/` with full structure

## 3. Install Valkey (if needed)

Woodstock server requires **Valkey** (Redis-compatible) for job queue and distributed locks:

```bash
# On Debian Bookworm
apt-get install valkey
systemctl enable --now valkey-server
```

Or with Redis:

```bash
apt-get install redis-server
systemctl enable --now redis-server
```

## 4. Configure the Server

Edit the environment file:

```bash
nano /etc/woodstock/server.env
```

Essential parameters:

```ini
# Path to backup data (must have sufficient space)
BACKUP_PATH=/var/lib/woodstock

# Redis/Valkey connection
REDIS_HOST=localhost
REDIS_PORT=6379

# Pre-compiled web interface
STATIC_PATH=/usr/share/woodstock/static

# Log level: error, warn, info, debug
LOG_LEVEL=info

# REST API port (default: 3000)
MANAGEMENT_API_PORT=3000
```

::: tip Disk Space
If your backup data must reside on a different disk or partition, mount it on `/var/lib/woodstock` or modify `BACKUP_PATH` to point to the desired path.
:::

## 5. Start the Services

Woodstock provides a single **systemd target** that starts all 4 services with one command:

```bash
# Enable and start all services
systemctl enable --now woodstock.target

# Check status
systemctl status woodstock-api woodstock-client-api woodstock-worker woodstock-scheduler
```

The 4 services can also be managed individually:

| Service | Role | Port |
|---------|------|------|
| `woodstock-api` | REST API + web interface | 3000 |
| `woodstock-client-api` | mTLS gateway for agents | 8443 |
| `woodstock-worker` | Backup/restore worker | — |
| `woodstock-scheduler` | CRON scheduler | — |

## 6. Verify Installation

```bash
# Access the web interface
curl http://localhost:3000/

# Check logs
journalctl -u woodstock-api -f
journalctl -u woodstock-worker -f
```

The web interface is available at `http://<server-address>:3000`.

## 7. Install Client on Machines to be Backed Up

```bash
apt-get install woodstock-client
```

Then configure the client:

```bash
nano /etc/woodstock/config.yaml
systemctl enable --now woodstock-client
```

See the [agent configuration guide](/doc/agent) for details.

## Firewall

Open necessary ports on the server:

```bash
# Web interface (HTTPS recommended via reverse proxy)
ufw allow 3000/tcp

# mTLS gateway for agents (required)
ufw allow 8443/tcp
```

## Upgrade

```bash
apt-get update && apt-get upgrade woodstock-server
```

Services are automatically restarted after upgrade.

## Uninstallation

```bash
# Remove package (keeps data)
apt-get remove woodstock-server

# Remove package AND all data (/var/lib/woodstock)
apt-get purge woodstock-server
```

::: warning
`apt-get purge` permanently deletes all backups stored in `/var/lib/woodstock`. Make sure you have a copy of your data before using this command.
:::
