# Configuration of Woodstock Backup

## Configuration of the list of backup

In the directory specified by the `CONFIG_PATH` environment variable (default: `/var/lib/woodstock/config`), you must configure in the file `hosts.yml` the list of hosts to backup.

The file is in YAML format and contains a list of host names.

```yaml
- server-1
- server-2
- server-3
```

For each server to backup, you must create a file with the name of the server (e.g., `server-1.yml`) in the same directory.

## Content of the configuration of a host

This is an example of configuration for a host:

```yaml
password: zYWYK1zdYdrvQwEu96n1rIMU4K3LA38xGERMXdDH2chdUV8mJXAuK8XvgWQFoiDq
addresses:
  - 10.0.0.1
operations:
  preCommands:
    - command: /data/prebackup.sh
  operation:
    shares:
      - name: /data/dump
      - name: /data/volumes
        excludes:
          - "**/*.unison.tmp"
```

| Field        | Default value    | Description                                                                                                       |
| ------------ | ---------------- | ----------------------------------------------------------------------------------------------------------------- |
| password     |                  | Password used for authentication with the client agent (must match the password configured on the agent).         |
| addresses    |                  | List of IP addresses associated with the host. This can be used if Direct resolution or mDNS resolution can't be used.       |
| port         | 3657             | Port used to connect to the host.                                                                                 |
| operations   |                  | List of operations to execute on the host.                                                                        |
| schedule     |                  | The scheduler configuration for the backup.                                                                       |

### The scheduler

Inside the `schedule` field:

| Field                       | Default value                                                  | Description                                  |
| --------------------------- | -------------------------------------------------------------- | -------------------------------------------- |
| activated                   | true                                                           | Enable or disable automatic backups          |
| backupPeriod                | 86400                                                           | Period between two backups in seconds (default: 24 hours) |
| backupToKeep                | `{ hourly: -1, daily: 7, weekly: 4, monthly: 12, yearly: -1, yearly_limit: null }` | Number of backups to keep in each category. Defines representatives to keep for each time window. `yearly_limit` sets a maximum number of yearly representatives. |
| blackout                    | none                                                            | List of recurring windows during which a new backup should not be started for this host. See [Updating the Scheduler](./scheduler.md) for the format and how it combines with the global default. |
| blackoutOverrideAfterPeriods | none                                                           | Overrides `blackout` once the host is late by more than this multiple of `backupPeriod` (e.g. `1.5`). Omit to make the blackout strict (no automatic override). |

### Operations

In the list of operations we have three parts:

| Field         | Default value | Description                                   |
| ------------- | ------------- | ----------------------------------------------|
| preCommands   |               | Array of commands to execute before the backup |
| operation     |               | List of shares and folders to backup           |
| postCommands  |               | Array of commands to execute after the backup  |

The `preCommands` and `postCommands` arrays are lists of `ExecuteCommandOperation` objects. The `operation` field is of type `BackupOperation`.

### ExecuteCommandOperation

| Field   | Default value | Description                            |
| ------- | ------------- | -------------------------------------- |
| command |               | A command to execute (ex: `/bin/true`) |

### BackupOperation

| Field    | Default value | Description                                |
| -------- | ------------- | ------------------------------------------ |
| includes | []            | List of files to include                   |
| excludes | []            | List of files to exclude (e.g., *.bak)     |
| timeout  | 120           | Timeout in seconds after an inactive period|
| shares   |               | List of backup shares                      |

Each share has the following properties:

| Field      | Default value | Description                                                          |
| ---------- | ------------- | -------------------------------------------------------------------- |
| name       |               | Name of the share (path on the client to backup)                     |
| includes   | []            | List of files to include (merged with includes of backup)            |
| excludes   | []            | List of files to exclude (merged with excludes of backup)            |

### How to Write Includes and Excludes

The includes and excludes fields allow you to specify which files should be included or excluded during the backup
process. These fields use patterns that follow the rules of the [globset](https://docs.rs/globset/latest/globset/)
crate.

#### Includes

The `includes` field is a list of patterns that specify which files to include in the backup. If this field is empty,
all files are included by default.

Examples:

* `*`: Matches any file.
* `*.txt`: Matches all files with the `.txt` extension.
* `**/*.log`: Matches all `.log` files in all subdirectories.
* `data/**`: Matches all files and folders within the data directory.

#### Excludes

The excludes field is a list of patterns that specify which files to exclude from the backup. These patterns are
applied after the includes patterns.

Examples:

* `*.bak`: Excludes all files with the .bak extension.
* `temp/**`: Excludes all files and folders within the temp directory.

#### Example Configuration

In this example:

```yaml
includes:
  - "*.txt"
  - "data/**/*.log"
excludes:
  - "*.bak"
  - "temp/**"
```

* All `.txt` files and all `.log` files within the `data` directory and its subdirectories are included.
* All `.bak` files and all files within the `temp` directory and its subdirectories are excluded.

By using these patterns, you can precisely control which files are included or excluded in your backup operations.

## Refresh cache

After modifying the configuration, you must refresh the cache from the web interface.

## Configuration Examples

Here are some examples of host configurations for different use cases:

### Example 1: Basic Server Configuration with Pre-Commands

This example demonstrates a configuration for a remote server with a pre-backup script:

```yaml
password: f8vyj2r9rNyax7yMWAHKmQExo1GrXeI2n8dAJg40TmpntjYx1IDKJHnXfh8RiRp8
schedule:
  activated: false
addresses:
  - 192.168.1.100
operations:
  preCommands:
    - command: /var/app/prebackup.sh
  operation:
    excludes:
      - "*.unison.tmp"
    shares:
      - name: /data/dump
      - name: /data/volumes
```

### Example 2: Linux Home Directory with Extensive Exclusions

This example shows how to back up a Linux home directory while excluding numerous unnecessary files and directories:

```yaml
password: f8vyj2r9rNyax7yMWAHKmQExo1GrXeI2n8dAJg40TmpntjYx1IDKJHnXfh8RiRp8
schedule:
  activated: false
operations:
  operation:
    shares:
      - name: /home
        excludes:
          - "*.vdi"
          - "*.vmdk"
          - "*.sft"
          - "*.safetensors"
          - "*.ckpt"
          - "*.pt"
          - "*.pth"
          - ".cache/lm-studio/"
          - "**/.venv"
          - "**/app/cache"
          - "**/web/public/"
          - "**/mongodb/configdb"
          - "**/mongodb/db"
          - "**/mongodb/dump"
          - "**/var/cache"
          - "**/node_modules"
          - "**/vcpkg"
          - "**/vendor"
          - "**/target/debug"
          - "**/target/release"
          - "user/.android"
          - "user/.AndroidStudio*"
          - "user/.bun"
          - "user/.cache"
          - "user/.cargo"
          - "user/.ccache"
          - "user/.CloudStation"
          - "user/.composer"
          - "user/.config/Code"
          - "user/.config/google-chrome"
          - "user/.gradle"
          - "user/.local/share/flatpak"
          - "user/.local/share/Google"
          - "user/.local/share/Trash"
          - "user/.m2"
          - "user/.meteor"
          - "user/.npm"
          - "user/.nvm"
          - "user/.rustup"
          - "user/.thumbnails"
          - "user/.vagrant.d"
          - "user/.VirtualBox"
          - "user/.vscode"
          - "user/.vscode-server"
          - "user/.wine"
          - "user/snap"
          - "user/tmp"
          - "user/VirtualBox VMs"
      - name: /etc
```

### Example 3: Windows Multi-Drive Backup with Path-Specific Exclusions

This example demonstrates how to configure backups for a Windows system with multiple drives and path-specific exclusions:

```yaml
password: f8vyj2r9rNyax7yMWAHKmQExo1GrXeI2n8dAJg40TmpntjYx1IDKJHnXfh8RiRp8
schedule:
  activated: true
operations:
  operation:
    excludes:
      - "*.vdi"
      - "*.vmdk"
      - "*.sft"
      - "*.safetensors"
      - "*.ckpt"
      - "*.pt"
      - "*.pth"
      - "**\\app\\cache"
      - "**\\web\\public"
      - "**\\mongodb\\configdb"
      - "**\\mongodb\\db"
      - "**\\mongodb\\dump"
      - "**\\var\\cache"
      - "**\\node_modules"
      - "**\\vcpkg"
      - "**\\target\\debug"
      - "**\\target\\release"
      - "*.tmp"
      - "pagefile.sys"
      - "System Volume Information"
      - "$RECYCLE.BIN"
    shares:
      - name: "c:\\tools"
      - name: "c:\\Users"
        excludes:
          - "alice\\.android"
          - "alice\\.rustup"
          - "alice\\.cargo"
          - "alice\\.vscode"
          - "alice\\.gradle"
          - "alice\\AppData\\Roaming\\.minecraft"
          - "alice\\AppData\\Roaming\\Code"
          - "alice\\AppData\\Roaming\\Google"
          - "alice\\AppData\\Roaming\\Mozilla\\Firefox"
          - "alice\\AppData\\Local"
          - "alice\\SynologyDrive"
          - "alice\\OneDrive"
          - "alice\\CrossDevice"
          - "bob\\SynologyDrive"
          - "bob\\OneDrive"
          - "bob\\CrossDevice"
      - name: "d:\\"
        excludes:
          - "Documents public"
```

These examples demonstrate different approaches to configuring backups based on the operating system and specific requirements. You can adapt these examples to your needs by modifying the paths, exclusion patterns, and other settings as appropriate.
