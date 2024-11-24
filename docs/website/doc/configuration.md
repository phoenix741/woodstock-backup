# Configuration of Woodstock Backup

## Configuration of the list of backup

In the directory `/var/lib/woodstock/config`, you must configure in the file `hosts.yml` the list of hosts to backup.

The file is in yaml format and is a list of host name.

```yaml
- server-1
- server-2
- server-3
```

For each server to backup, you must create a file with the name of the server in the directory `/var/lib/woodstock/config`.

## Content of the configuration of an host

This is an exemple of configuration of an host:

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
| password     |                  | Password used to make the authentification to the clien (should be equal on the agent).                           |
| addresses    |                  | List of IP addresses associated to the host. If not defined, the name of the host will be used to resolve the IP. |
| port         | 3657             | Port used to connect to the host.                                                                                 |
| operations   |                  | List of operations to execute on the host.                                                                        |
| schedule     |                  | The scheduler of the backup.                                                                                      |

### The scheduler

Inside the field `scheduler`:

| Field        | Default value                                                  | Description                                  |
| ------------ | -------------------------------------------------------------- | -------------------------------------------- |
| activated    | true                                                           | Active / Desactive the automatic backup      |
| backupPeriod | 8340                                                           | Period between two backup: 24H - 5 minutes   |
| backupToKeep | `{ hourly: -1, daily: 7, weekly: 4, monthly: 12, yearly: -1 }` | Number of backup to keep (not used actually) |

### Operations

In the list of operations we have two parts:

| Field         | Default value | Description                                   |
| ------------- | ------------- | ----------------------------------------------|
| preCommands   |               | Array of command to execute before the backup |
| operation     |               | List of share and folder to backup            |
| postCommands  |               | Array of command to execute after the backup  |

The preCommands and postCommands array are a list of ExecuteCommandOperation. The operation is of type `BackupOperation`.

### ExecuteCommandOperation

| Field   | Default value | Description                            |
| ------- | ------------- | -------------------------------------- |
| command |               | A command to execute (ex: `/bin/true`) |

### BackupOperation

| Field    | Default value | Description                                |
| -------- | ------------- | ------------------------------------------ |
| includes | []            | List file to includes                      |
| excludes | []            | List file to excludes (\*.bak, ...)        |
| timeout  | 120           | Timeout of backup after an inactive period |
| share    |               | List of backup share                       |

Each share has the following property:

| Field      | Default value | Description                                                          |
| ---------- | ------------- | -------------------------------------------------------------------- |
| name       |               | Name of the share (name of the path in the client)                   |
| includes   | []            | List file to includes (merged with includes of backup)               |
| excludes   | []            | List file to excludes (merged with includes of backup - \*.bak, ...) |

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
