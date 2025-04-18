# Migrating from BackupPC to Woodstock

This guide explains how to migrate your backups from BackupPC to Woodstock Backup using the integrated migration tool.

## Overview

If you're currently using BackupPC and want to switch to Woodstock Backup, you can use the migration tool to transfer your existing backups without losing any data. The migration tool will:

1. Import backups from BackupPC to Woodstock
2. Keep the same folder structure
3. Maintain reference counting in the pool
4. Skip backups that have already been imported
5. Optionally remove old backups that don't exist anymore in BackupPC

The migration tool is designed to be run in one go, but you can also run it multiple times. It will only import backups
that haven't been migrated yet.

## Prerequisites

- A working BackupPC installation
- A working Woodstock Backup installation
- Access to both pool directories

## Using the Migration Tool

The migration tool is distributed as a binary named `ws_backuppc_importer` in the Woodstock Backup package.

### Command Syntax

```bash
ws_backuppc_importer <backuppc_pool> <woodstock_pool> [options]
```

### Arguments and Options

| Argument/Option | Description |
|----------------|-------------|
| `backuppc_pool` | Path to the BackupPC pool directory |
| `woodstock_pool` | Path to the Woodstock pool directory |
| `-o, --only-one` | Import only the first backup (useful for testing) |
| `-d, --dry-run` | Show what would be done without actually performing the migration |
| `--excludes` | List of hosts to exclude from migration (comma-separated) |

### Examples

#### Basic Migration

To migrate all backups from BackupPC to Woodstock:

```bash
ws_backuppc_importer /var/lib/backuppc /var/lib/woodstock/pool
```

#### Test Migration with a Single Backup

To test the migration with only one backup:

```bash
ws_backuppc_importer /var/lib/backuppc /var/lib/woodstock/pool --only-one
```

#### Dry Run

To see what would be migrated without actually performing the migration:

```bash
ws_backuppc_importer /var/lib/backuppc /var/lib/woodstock/pool --dry-run
```

#### Exclude Hosts

To exclude specific hosts from migration:

```bash
ws_backuppc_importer /var/lib/backuppc /var/lib/woodstock/pool --excludes server1,server2
```

#### Scheduled Daily Migration

This example shows how to set up a daily migration job that runs at 7:00 AM, with environment variables to define paths
and exclude specific hosts:

```bash
# Set environment variables for Woodstock paths
BACKUP_PATH=/volume1/docker/woodstock/config/ 
POOL_PATH=/volume2/backup_woodstock_pool/pool 
HOSTS_PATH=/volume2/backup_woodstock_pool/hosts 

export BACKUP_PATH POOL_PATH HOSTS_PATH

# Run the migration, excluding specific hosts
/volume2/backup_woodstock_pool/ws_backuppc_importer /volume2/backup_backuppc/data/ /volume1/docker/woodstock/config/ --excludes pc-user1 --excludes pc-user2
```

You can add this to your crontab to run it automatically:

```bash
# Run BackupPC to Woodstock migration every day at 7:00 AM
0 7 * * * /path/to/migration_script.sh > /path/to/migration_log.txt 2>&1
```

## Migration Process

The migration tool follows these steps:

1. Lists all backups in the Woodstock pool
2. Lists all backups in the BackupPC pool
3. Filters out backups that already exist in Woodstock (based on hostname and timestamp)
4. Imports each BackupPC backup that doesn't exist in Woodstock
5. Optionally removes backups from Woodstock that don't exist in BackupPC (unless using `--only-one`)

## Standard Output and Common Messages

When running the migration tool, you'll see output similar to the following:

```stdout
BackupPC to Woodstock migration tool v2.0.0-alpha.30
Woodstock path:
  - Backup:      "/volume1/docker/woodstock/config/"
  - Certificate: "/volume1/docker/woodstock/config/certs"
  - Config:      "/volume1/docker/woodstock/config/config"
  - Hosts:       "/volume2/backup_woodstock_pool/hosts"
  - Logs:        "/volume1/docker/woodstock/config/logs"
  - Events:      "/volume1/docker/woodstock/config/events"
  - Pool:        "/volume2/backup_woodstock_pool/pool"
  - Jobs:        "/volume1/docker/woodstock/config/logs/jobs"
[1/4] Import BackupPC pool /volume2/backup_backuppc/data/ to Woodstock pool /volume1/docker/woodstock/config/
[2/4] Found 4 backuppc backups
[2025-03-18T08:01:53Z ERROR woodstock::server::backup_client] Can't download chunk for "systemd/system/snap-gnome/x2d3/x2d28/x2d1804-198.mount": File not found (not in attributs): "pc-user1/1460/etc/systemd/system/snap-gnome/x2d3/x2d28/x2d1804-198.mount"
[2025-03-18T08:01:53Z ERROR woodstock::server::backup_client] Can't download chunk for "systemd/system/snap-gtk/x2dcommon/x2dthemes-1535.mount": File not found (not in attributs): "pc-user1/1460/etc/systemd/system/snap-gtk/x2dcommon/x2dthemes-1535.mount"
[3/4] Remove 2 old backups
[4/4] Backups migrate
```

### Understanding the Output

1. **Version and Paths**: The tool displays its version and all the paths it's using for the migration.

2. **Step 1/4**: Shows the source BackupPC pool and destination Woodstock pool.

3. **Step 2/4**: Indicates how many backups will be migrated from BackupPC.

4. **Error Messages**: You may see error messages like `Can't download chunk for...`. These are usually non-critical and indicate specific files that couldn't be migrated, often due to inconsistencies in the BackupPC pool or temporary files that were not properly recorded in the backup.

5. **Step 3/4**: Shows how many old backups will be removed from Woodstock (backups that no longer exist in BackupPC).

6. **Step 4/4**: Confirms the migration process is complete.

## Progress Indicators

The tool displays progress bars showing:

- Overall migration progress
- Individual backup progress with speed estimates
- Estimated time remaining

## Troubleshooting

If errors occur during migration, they will be logged. The most common issues are:

- Permission problems accessing the BackupPC or Woodstock directories
- Insufficient disk space in the destination pool
- Corrupted backups in the source pool

Check the logs for detailed error messages if the migration fails.

## Post-Migration Steps

After migration is complete:

1. Verify that all backups have been correctly imported by checking the Woodstock web interface
2. Run a test restore to ensure data integrity
3. If everything looks good, you can decommission your BackupPC server

## Notes

- The migration process can be time-consuming for large backup pools
- You can run the migration multiple times; it will skip backups that have already been imported
- Consider running with `--only-one` first to verify everything works correctly before doing a full migration
