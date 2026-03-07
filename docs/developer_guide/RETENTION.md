# Backup Retention Policy

## Overview

The Backup Retention Policy in Woodstock Backup is designed to manage long-term storage consumption automatically. Instead of a simple "keep the last N backups", Woodstock employs a time-based **Sliding Window Algorithm** (also known as a Grandfather-Father-Son approach). It categorizes backups into multiple tiers based on age: Hourly, Daily, Weekly, Monthly, and Yearly.

The system selectively keeps representatives for each configured time window and marks the rest as surplus, subsequently orchestrating their deletion.

## Retention Algorithm (`woodstock-rs/src/server/backup/retention.rs`)

The `retention.rs` module performs the core task of classifying backups.

1. **Terminal State Prerequisite:** Only backups in terminal states (Completed, Failed, Aborted) are considered. Incomplete backups are ignored to prevent race conditions during cleanup. By default, every completed backup starts as `Surplus`.
2. **First Backup Seed:** Since all windows are evaluated relative to the newest backup, the algorithm starts by finding the most recent `Completed` backup as its timestamp base `t0`.
3. **Time Bucket Evaluation:** Each backup's start time is evaluated against boundary limits derived from the `ScheduledBackupToKeep` configuration:
    - **Hourly**: Retained if the time is `>= t0 - (hourly * 1 hour)`
    - **Daily**: Retained if the time is `>= t0 - (daily * 1 day)`
    - **Weekly**: Retained if the time is `>= t0 - (weekly * 7 days)`
    - **Monthly**: Retained if the time is `>= t0 - (monthly * 1 month)`
    - **Yearly**: Retained if the time is `>= t0 - (yearly_limit * 1 year)` if applicable.

When multiple backups fall within the same discrete time slot (e.g., the same precise day or same week number), only the **newest** backup within that slot is retained as the representative.

### Yearly Limit Parameter

A `yearly_limit` configuration dictates a ceiling for the number of yearly representatives evaluated globally. 
If infinite (`None`), Woodstock will indefinitely retain valid yearly representatives. If set (e.g., `5`), only the newest `5` yearly representatives are kept, while older ones fall back into the `Surplus` pile to be wiped.

### Last Backup Guard

A strict safety constraint exists to guarantee total loss never occurs due to a misaligned policy: the last successful backup is ALWAYS promoted from `Surplus` to `LastBackup` and cannot be deleted. The active backup being performed also holds an implicit lock globally while the retention algorithm processes.

## Cleanup Process & Integration (`server-rs`)

Backup deletion is orchestrated asynchronously avoiding blockages on the Axum endpoint or API server threads.

### 1. Job Enqueuing (`job_worker.rs`, `producers.rs`)
After a successful backup, the worker node triggers `enforce_retention_policy()`. 
This function calls `get_backups_to_delete()`, producing a `Vec<Uuid>` identifying obsolete backups.
For each stale UUID, an apalis `Remove` job is spawned into the `backup_storage` Redis queue via `enqueue_retention_removals()`.

Additionally, the GraphQL `enforce_retention_for_host` mutation allows manual pruning, which also re-enqueues jobs that previously crashed midway into a `Removing` status.

### 2. Disk Synchronization (`backups.rs`)

Due to the destructive nature of removing data, write operations bypass the typical file cache layer. 
The system acquires an exclusive Redis lock (`PoolLockRedis::lock_exclusive()`) scoped locally to the host's `backup.yml` file. This lock prevents multiple agents (such as concurrent `Remove` jobs) from corrupting or erasing shared metadata during read-modify-write phases. 

The `backup.yml` file is immediately synced to disk and un-cached before its parent directory (and chunks) proceed for complete physical deletion.

## Categories Exposed (`ApiServerState`)

The result of the retention algorithm is mapped to standard output to the end-user.
Via GraphQL (`BackupEx`), each returned backup natively embeds a `retention_category` representing the retention chip visible on the frontend matrix (e.g., Daily, Hourly, Surplus) for transparent administration.