# Configure new backup

To configure backup for an host file we need to create a file in the woodstock directory (ex: `/var/lib/woodstock/config`).

## Content of the configuration

This is an exemple of configuration of an host:

```yaml
#schedule:
#  activated: True
addresses:
  - 10.0.0.1
operations:
  tasks:
    - name: ExecuteCommand
      command: ssh -q -x -l root 10.0.0.1 /root/mysql_backup.sh
    - name: RSyncBackup
      share:
        - name: /data/dump
        - name: /data/volumes/volume1
        - name: /data/volumes/volume2
        - name: /data/volumes/volume3
          excludes:
            - dir1
            - dir2
            - dir3
            - dir4
```

| Field        | Default value                                                  | Description                                                                                                       |
| ------------ | -------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| addresses    |                                                                | List of IP addresses associated to the host. If not defined, the name of the host will be used to resolve the IP. |
| backupToKeep | `{ hourly: -1, daily: 7, weekly: 4, monthly: 12, yearly: -1 }` | Number of backup to keep (not used actually)                                                                      |
| dhcp         |                                                                | List of range of IP used to search the host that have the name                                                    |
| operations   |                                                                | List of operations                                                                                                |

### The scheduler

Inside the field `scheduler`:

| Field        | Default value                                                  | Description                                  |
| ------------ | -------------------------------------------------------------- | -------------------------------------------- |
| activated    | true                                                           | Active / Desactive the automatic backup      |
| backupPeriod | 8340                                                           | Period between two backup: 24H - 5 minutes   |
| backupToKeep | `{ hourly: -1, daily: 7, weekly: 4, monthly: 12, yearly: -1 }` | Number of backup to keep (not used actually) |

### DHCP

Each DHCP Addresses will have the form:

| Field   | Default value | Description                                   |
| ------- | ------------- | --------------------------------------------- |
| address |               | IP of the form xxx.yyy.zzz                    |
| start   |               | The last part of the IP (start of the search) |
| end     |               | The last part of the IP (end of the search)   |

### Operations

In the list of operations we have two parts:

| Field         | Default value | Description                                                                               |
| ------------- | ------------- | ----------------------------------------------------------------------------------------- |
| tasks         |               | Array of operation executed, if one operation failed, all next operation will fail too. |
| finalizeTasks |               | Array of operation executed at the end, even if list of tasks has  failed            |

The operation can be of three type:

- `ExecuteCommand`: Execute the given command on the backup serveur,
- `RSyncBackup`: Execute the backup of the host using `rsync`,
- `RSyncdBackup`: Execute the backup of the host using `rsync` and connecting to a `rsyncd` server.

For `ExecuteCommand`:

| Field   | Default value | Description                            |
| ------- | ------------- | -------------------------------------- |
| name    |               | 'ExecuteCommand'                       |
| command |               | A command to execute (ex: `/bin/true`) |

For `RSyncBackup`:

| Field    | Default value | Description                               |
| -------- | ------------- | ----------------------------------------- |
| name     |               | 'RSyncBackup'                             |
| includes | []            | List file to includes                     |
| excludes | []            | List file to excludes (\*.bak, ...)       |
| timeout  | 120           | Timeout of rsync after an inactive period |
| share    |               | List of backup share                      |

For `RSyncdBackup`:

| Field            | Default value | Description                                     |
| ---------------- | ------------- | ----------------------------------------------- |
| name             |               | 'RSyncdBackup'                                  |
| authentification | false         | The rsync should be authenticated to the server |
| username         |               | Username used for the backup                    |
| password         |               | Password used for the backup                    |
| includes         | []            | List file to includes                           |
| excludes         | []            | List file to excludes (\*.bak, ...)             |
| timeout          | 120           | Timeout of rsync after an inactive period       |
| share            |               | List of backup share                            |

Each share has the following property:

| Field      | Default value | Description                                                          |
| ---------- | ------------- | -------------------------------------------------------------------- |
| name       |               | Name of the share (name of the path in the client)                   |
| includes   | []            | List file to includes (merged with includes of backup)               |
| excludes   | []            | List file to excludes (merged with includes of backup - \*.bak, ...) |
| checksum   | false         | True if checksum should be verified on files                         |
| pathPrefix |               | The path prefix prepend to the share name                            |

## Add the host

After the creation of the file with the configuration we need to add it to the file `hosts.yml` like this:

```yaml
- server-ovh-1
- server-ovh-2
- server-ovh-3
```

## Check you can connect to the host without password.

Using the same user of `Woodstock Backup`, try to connect to the host with the command:

```bash
ssh-copy-id root@host
ssh root@host
```
