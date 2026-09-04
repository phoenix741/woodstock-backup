# Updating the Scheduler

Woodstock Backup no longer wakes up on a fixed interval to scan every host, nor does it poll archive profiles or nightly maintenance on their own separate fixed ticks. Instead, hosts, archive profiles, and nightly maintenance are all driven by the same single scheduler loop:

* A backup starts **as soon as a host comes back online** (self-registration or mDNS), if it is due — no need to wait for a periodic scan. This is delivered over a durable Redis Stream, not Pub/Sub, so a scheduler restart or a brief reconnect never silently drops the event.
* Otherwise, the scheduler sleeps until the next *real* deadline across all hosts, archive profiles, and the nightly maintenance trigger — computed from each host's `backupPeriod`, each archive profile's own cron in `archiving.yml`, and `nightlySchedule` respectively — not on a fixed cadence.
* A host that is **due but known offline** is not polled on a timer at all: it is excluded from the sleep computation entirely and relies solely on the online event above. A host that is due and whose reachability is *unknown* (typically a fixed-IP host with no self-registration) is still retried periodically on a short backoff until it's found — see `retryBackoffOnRefusalSecs` below.

There is no periodic safety-net wakeup anymore: a real due date, however far out, is never overridden. This means a config change (a new host, a re-activated schedule, a shortened `backupPeriod`, a new or edited archive profile, an edited `nightlySchedule`) is only picked up once the scheduler process **restarts** — restart the `scheduler` service after any such change. A Redis-based live-notification mechanism to avoid that restart is planned but not yet implemented.

You can update the default scheduler configuration instead of modifying individual host schedulers.

```yaml
nightlySchedule: "0 0 0 * * * *"
defaultSchedule:
  activated: true
  backupPeriod: 86400
  backupToKeep:
    hourly: 24
    daily: 7
    weekly: 4
    monthly: 12
    yearly: 1
    yearly_limit: null
  blackout:
    - days: [1, 2, 3, 4, 5] # Monday..Friday
      start: "08:00"
      end: "18:00"
  blackoutOverrideAfterPeriods: 1.5
wakeupFloorSecs: 30
retryBackoffAfterSuccessSecs: 300
retryBackoffOnRefusalSecs: 900
```

## The Application Scheduler

Inside the configuration file, you have the following properties:

| Field           | Default value  | Description                                                           |
| --------------- | -------------  | --------------------------------------------------------------------- |
| nightlySchedule | `0 0 0 * * * *`    | 7-part cron expression (with seconds) for the nightly maintenance run (orphaned chunk cleanup). Checked against its own last-run state on every scheduler wakeup, same as archive profiles — not a separate fixed tick. |
| defaultSchedule | See above          | The default backup scheduler configuration, used as a fallback for any field a host's own `schedule` doesn't set (including `blackout`) |
| wakeupFloorSecs | `30`  | Floor under the scanner's dynamic sleep, so a host/archive profile/nightly trigger stuck permanently "due" (e.g. an unreachable host) can never turn the loop into a busy-poll. |
| retryBackoffAfterSuccessSecs | `300` (5 min) | Cooldown recorded for a host after a successful enqueue, so the scanner doesn't immediately re-consider it before the job it just enqueued has had a chance to start. |
| retryBackoffOnRefusalSecs | `900` (15 min) | Cooldown recorded for a host after a refused scheduling attempt (already running, blocked by an active pool fsck lock, or unreachable while its reachability isn't otherwise tracked — e.g. a fixed-IP host). A host known offline is not retried on this backoff at all — it relies entirely on the online event, so this setting no longer governs it. |

## The Scheduler

Inside the `scheduler` field:

| Field                       | Default value                                                  | Description                                  |
| --------------------------- | -------------------------------------------------------------- | -------------------------------------------- |
| activated                   | true                                                           | Enable or disable automatic backups          |
| backupPeriod                | 86400                                                          | Period between two backups in seconds (default: 24 hours) |
| backupToKeep                | `{ hourly: 24, daily: 7, weekly: 4, monthly: 12, yearly: 1, yearly_limit: null }`  | Number of backups to keep in each category. The `yearly_limit` parameter sets a maximum limit on the total number of yearly representatives kept, discarding the oldest first (`null` for unlimited). |
| blackout                    | none                                                            | List of recurring windows during which a new backup should not be started, unless `blackoutOverrideAfterPeriods` allows it. Each window has `days` (`0` = Sunday .. `6` = Saturday), `start` and `end` (`"HH:MM"`, local time). `end` may be earlier than `start` to express a window crossing midnight (e.g. `22:00` -> `06:00`) — the window is anchored to `days` by its `start`. Falls back to the global `defaultSchedule.blackout` when a host doesn't set its own. |
| blackoutOverrideAfterPeriods | none                                                           | Overrides `blackout` once a host is late by more than this multiple of `backupPeriod` (e.g. `1.5`). A host that has never been backed up, or whose last backup was aborted, is always let through regardless of blackout — a first/incremental-base backup should never be indefinitely postponed. Omitting this field makes the blackout strict (no automatic override). |

### Example: a working-hours blackout with an escape hatch

```yaml
blackout:
  - days: [1, 2, 3, 4, 5] # Monday..Friday
    start: "08:00"
    end: "18:00"
blackoutOverrideAfterPeriods: 1.5
```

No backup starts for this host on weekdays between 08:00 and 18:00 — unless it has gone more than 1.5x its `backupPeriod` without a successful backup, in which case the blackout is overridden so the host doesn't fall further behind indefinitely.
