# Agent Installation

Woodstock Backup architecture relies on a central server communicating with agents installed on client machines. This page explains how to install and configure agents on different platforms.

## Overview

Woodstock Backup agents are responsible for:

- Analyzing local filesystems
- Creating filesystem snapshots when a supported backend is available
- Executing backup operations
- Restoring files (if enabled)
- Secure communication with the central server

## Snapshot-backed Backups

To improve consistency, the agent can read files from a snapshot instead of from the live filesystem.

Supported backends today:

- Linux: Btrfs snapshots
- Windows: VSS snapshots on local drive-letter volumes such as `C:\Data`

Current behavior:

- The agent automatically tries to create a snapshot when a compatible backend is available.
- If snapshot creation fails, the backup continues from the live filesystem and the agent logs the error.
- Snapshots are finalized after a successful backup, aborted during cleanup paths, and cleaned again on
   graceful shutdown or on the next authentication if a previous session left an orphaned snapshot.

Current limitations:

- Windows VSS support is limited to local drive-letter paths. UNC paths such as `\\server\share` cannot use VSS.
- Explicit per-job or per-share snapshot policy is not exposed yet in the server-side host configuration.

## Download

When you add a new host in the Woodstock Backup interface, the system automatically generates an installation package containing:

- `config.yaml`: The configuration file for the agent.
- Certificates needed for authentication.
- The agent executable for your operating system (for some platforms).

## Windows Installation

### Prerequisites

The [Visual C++ Redistributable 2015-2022 (x64)](https://learn.microsoft.com/cpp/windows/latest-supported-vc-redist)
must be installed. `ws_client_daemon.exe` depends on `VCRUNTIME140.dll`, which
it does not ship; without it, any invocation — including `install-service` —
exits immediately with status `0xC0000135` and no error message.

After downloading, you will have a zip file containing:

- `config.yaml`: The configuration file for the agent.
- `ws_client_daemon.exe`: The executable for Windows.
- Certificates needed for authentication.

### Installation Steps

1. Extract the zip file contents to a folder of your choice. For example: `C:\ProgramData\woodstock`.
2. Open a Command Prompt with administrative privileges.
3. Navigate to the folder where you extracted the files:

```powershell
cd C:\ProgramData\woodstock
```

1. Configure Windows Firewall to allow agent communications:

```powershell
.\ws_client_daemon.exe --config-dir C:\ProgramData\woodstock install-fw-rule
```

1. Install the agent as a Windows service:

```powershell
.\ws_client_daemon.exe --config-dir C:\ProgramData\woodstock install-service
```

The agent is now installed and running as a Windows service. It will automatically start with your computer and listen for instructions from the server.

## Linux Installation (Generic)

After downloading, you will have a zip file containing:

- `config.yaml`: The configuration file for the agent.
- `ws_client_daemon`: The executable for Linux.
- Certificates needed for authentication.

### Installation Steps

1. Extract the zip file contents to a folder of your choice. For example: `/opt/woodstock`.
2. Open a terminal with administrative privileges.
3. Navigate to the folder where you extracted the files:

```bash
cd /opt/woodstock
```

1. Make the daemon executable:

```bash
chmod +x ws_client_daemon
```

1. Create a systemd service file. Open a text editor with administrative privileges and create a file at `/etc/systemd/system/woodstock.service` with the following content:

```systemd
[Unit]
Description=Woodstock Backup Client
After=network.target

[Service]
ExecStart=/opt/woodstock/ws_client_daemon --config-dir /opt/woodstock
Restart=always
User=nobody
Group=nogroup

[Install]
WantedBy=multi-user.target
```

1. Reload the systemd daemon to recognize the new service and start it:

```bash
sudo systemctl daemon-reload
sudo systemctl enable woodstock.service
sudo systemctl start woodstock.service
```

The agent is now installed and running as a Linux service. It will automatically start with your computer and listen for instructions from the server.

## Linux Installation (Debian/Ubuntu)

After downloading, you will have a zip file containing:

- `config.yaml`: The configuration file for the agent.
- Certificates needed for authentication.

### Installation Steps

1. Extract the zip file contents to the `/etc/woodstock/` directory.
2. Open a terminal with administrative privileges.
3. Install the source package with the following commands:

```bash
sudo curl https://gogs.shadoware.org/api/packages/ShadowareOrg/debian/repository.key -o /etc/apt/keyrings/gitea-ShadowareOrg.asc
echo "deb [signed-by=/etc/apt/keyrings/gitea-ShadowareOrg.asc] https://gogs.shadoware.org/api/packages/ShadowareOrg/debian trixie main" | sudo tee -a /etc/apt/sources.list.d/gitea-shadowareorg.list
sudo apt update
```

1. Install the agent package:

```bash
sudo apt install woodstock-client
```

The agent is now installed and running as a Linux service. It will automatically start with your computer and listen for instructions from the server.

## Configuration

The main configuration is done in the `config.yaml` file. Here are all the available options:

| Option | Description | Default Value |
|--------|-------------|-------------------|
| `hostname` | The name that identifies this host on the server | System hostname |
| `bind` | The network address to bind to | "0.0.0.0:3657" |
| `password` | Password for authentication with the server | - |
| `secret` | The secret key for the client | Randomly generated 64-byte hexadecimal string |
| `backup_timeout` | Timeout for backup operations in seconds | 3600 (1 hour) |
| `max_backup_seconds` | Maximum duration of a backup operation in seconds | 43200 (12 hours) |
| `resolution_mode` | Connection method to the server - "Direct" (default), "Mdns" (if compiled with mDNS support), or "None" | "Direct" |
| `mdns_interfaces` | Optional list of network interfaces to use for mDNS discovery | - |
| `server` | Server address when using Direct mode (required if resolution_mode is Direct) | - |
| `disable_restauration` | Set to true to disable restore capabilities for security reasons | false |
| `xattr` | Set to true to enable backup of extended attributes on Linux and FreeBSD | false |
| `acl` | Set to true to enable backup of POSIX ACLs on Linux | false |
| `auto_update` | Enable automatic updates | true on Windows, false on other platforms |
| `update_delay` | Time in seconds between update checks | 86400 (24 hours) |
| `log_directory` | Directory where logs will be stored | Same as the configuration directory |
| `snapshot` | Snapshot preference flag in the client configuration schema | true |

> **Important**: `xattr` and `acl` default to `false` and the `config.yaml` shipped
> in the download bundle does not set them. Extended attributes and ACLs are
> therefore **not** backed up until you add them to the file yourself:
>
> ```yaml
> xattr: true
> acl: true    # Linux only; on FreeBSD this logs a warning at startup
> ```
>
> **Note**: snapshot support is already auto-detected by the current agent implementation. Keep `snapshot: true`.
> Explicit end-to-end disabling or per-job control is not fully wired yet.

You can also start the agent with the `--config-dir` parameter to specify an alternative configuration directory, or set the `CLIENT_PATH` environment variable.

## Command Line Options

The Woodstock Backup agent supports the following options:

| Option | Description |
|--------|-------------|
| `--config-dir <path>` | Specifies a custom configuration directory |
| `--version` | Displays the agent version |

Subcommands (availability depends on platform):

| Subcommand | Platform | Description |
|--------------|-----------|-------------|
| `self-update` | All | Updates the agent to the latest available version |
| `install-service` | Windows | Installs the agent as a Windows service |
| `remove-service` | Windows | Removes the Windows service |
| `restart-service` | Windows | Restarts the Windows service |
| `run-service` | Windows | Runs the program as a Windows service |
| `install-fw-rule` | Windows | Creates necessary Windows firewall rules |
| `remove-fw-rule` | Windows | Removes Windows firewall rules |

## Troubleshooting

### Agent Unavailable

If the agent appears unavailable in the server interface:

1. Check that the service is running
   - Windows: Check Services Manager
   - Linux: `systemctl status woodstock.service`

2. Check log files in the configuration directory

3. Ensure certificates were properly installed

4. Verify that the firewall allows communications on necessary ports
   - TCP port 3657 (default)
   - UDP port 5353 (if mDNS is used)

### Snapshot Creation Fails

If backups on Windows or Linux do not use a snapshot:

1. Verify that the share path is on a supported local filesystem.
   - Windows VSS requires a local drive-letter path such as `C:\Data`.
   - UNC paths and network shares cannot be snapshotted through VSS.

2. Check the agent logs for messages similar to `Failed to create a snapshot for ...`.

3. On Windows, ensure the agent runs with sufficient privileges and that the VSS service is available.

4. If snapshot creation still fails, the backup should continue from the live filesystem.

### `ws_client_daemon.exe` Fails to Start with No Error Message (Windows)

If any invocation of `ws_client_daemon.exe` — including `install-service` — exits
immediately with status `0xC0000135` and no message, the process could not
resolve `VCRUNTIME140.dll`. See the
[Prerequisites](#prerequisites) of the Windows Installation section: the
Visual C++ Redistributable 2015-2022 (x64) is required and is not bundled
with the agent.

### `config.yaml` Rejected with "missing field `password`" (Windows)

`Set-Content -Encoding utf8` in Windows PowerShell 5.1 writes a leading UTF-8
BOM by default. Older agent builds passed that BOM straight into the YAML
parser, which corrupted the first key and produced a confusing "missing
field" error even though the field was present in the file. Current builds
strip a leading BOM automatically; if you still hit this, re-save
`config.yaml` without a BOM (e.g. `Set-Content -Encoding ascii`) or upgrade
the agent.
