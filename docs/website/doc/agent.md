# Agent Installation

Woodstock Backup architecture relies on a central server communicating with agents installed on client machines. This page explains how to install and configure agents on different platforms.

## Overview

Woodstock Backup agents are responsible for:

- Analyzing local filesystems
- Executing backup operations
- Restoring files (if enabled)
- Secure communication with the central server

## Download

When you add a new host in the Woodstock Backup interface, the system automatically generates an installation package containing:

- `config.yml`: The configuration file for the agent.
- Certificates needed for authentication.
- The agent executable for your operating system (for some platforms).

## Windows Installation

After downloading, you will have a zip file containing:

- `config.yml`: The configuration file for the agent.
- `ws_client_daemon.exe`: The executable for Windows.
- Certificates needed for authentication.

### Installation Steps

1. Extract the zip file contents to a folder of your choice. For example: `C:\ProgramData\woodstock`.
2. Open a Command Prompt with administrative privileges.
3. Navigate to the folder where you extracted the files:

```powershell
cd C:\ProgramData\woodstock
```

4. Configure Windows Firewall to allow agent communications:

```powershell
.\ws_client_daemon.exe --config-dir C:\ProgramData\woodstock install-fw-rule
```

5. Install the agent as a Windows service:

```powershell
.\ws_client_daemon.exe --config-dir C:\ProgramData\woodstock install-service
```

The agent is now installed and running as a Windows service. It will automatically start with your computer and listen for instructions from the server.

## Linux Installation (Generic)

After downloading, you will have a zip file containing:

- `config.yml`: The configuration file for the agent.
- `ws_client_daemon`: The executable for Linux.
- Certificates needed for authentication.

### Installation Steps

1. Extract the zip file contents to a folder of your choice. For example: `/opt/woodstock`.
2. Open a terminal with administrative privileges.
3. Navigate to the folder where you extracted the files:

```bash
cd /opt/woodstock
```

4. Make the daemon executable:

```bash
chmod +x ws_client_daemon
```

5. Create a systemd service file. Open a text editor with administrative privileges and create a file at `/etc/systemd/system/woodstock.service` with the following content:

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

6. Reload the systemd daemon to recognize the new service and start it:

```bash
sudo systemctl daemon-reload
sudo systemctl enable woodstock.service
sudo systemctl start woodstock.service
```

The agent is now installed and running as a Linux service. It will automatically start with your computer and listen for instructions from the server.

## Linux Installation (Debian/Ubuntu)

After downloading, you will have a zip file containing:

- `config.yml`: The configuration file for the agent.
- Certificates needed for authentication.

### Installation Steps

1. Extract the zip file contents to the `/etc/woodstock/` directory.
2. Open a terminal with administrative privileges.
3. Install the source package with the following commands:

```bash
sudo curl https://gogs.shadoware.org/api/packages/ShadowareOrg/debian/repository.key -o /etc/apt/keyrings/gitea-ShadowareOrg.asc
echo "deb [signed-by=/etc/apt/keyrings/gitea-ShadowareOrg.asc] https://gogs.shadoware.org/api/packages/ShadowareOrg/debian bookworm main" | sudo tee -a /etc/apt/sources.list.d/gitea-shadowareorg.list
sudo apt update
```

4. Install the agent package:

```bash
sudo apt install woodstock-client-rs
```

The agent is now installed and running as a Linux service. It will automatically start with your computer and listen for instructions from the server.

## Configuration

The main configuration is done in the `config.yml` file. Here are all the available options:

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
| `xattr` | Set to true to enable backup of extended attributes on Linux | false |
| `acl` | Set to true to enable backup of ACLs on Linux | false |
| `auto_update` | Enable automatic updates | true on Windows, false on other platforms |
| `update_delay` | Time in seconds between update checks | 86400 (24 hours) |
| `log_directory` | Directory where logs will be stored | Same as the configuration directory |

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
