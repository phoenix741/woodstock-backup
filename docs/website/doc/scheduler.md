# Updating the Scheduler

You can update the default scheduler configuration instead of modifying individual host schedulers.

```yaml
wakeupSchedule: "0 */15 * * * * *"
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
```

## The Application Scheduler

Inside the configuration file, you have the following properties:

| Field           | Default value  | Description                                                           |
| --------------- | -------------  | --------------------------------------------------------------------- |
| wakeupSchedule  | `0 */15 * * * * *` | 7-part cron expression (with seconds) to wake up the backup software and launch all necessary backups |
| nightlySchedule | `0 0 0 * * * *`    | 7-part cron expression (with seconds) to wake up the backup software and generate statistics         |
| defaultSchedule | See above          | The default backup scheduler configuration                                                           |

## The Scheduler

Inside the `scheduler` field:

| Field        | Default value                                                  | Description                                  |
| ------------ | -------------------------------------------------------------- | -------------------------------------------- |
| activated    | true                                                           | Enable or disable automatic backups          |
| backupPeriod | 86400                                                          | Period between two backups in seconds (default: 24 hours) |
| backupToKeep | `{ hourly: 24, daily: 7, weekly: 4, monthly: 12, yearly: 1, yearly_limit: null }`  | Number of backups to keep in each category. The `yearly_limit` parameter sets a maximum limit on the total number of yearly representatives kept, discarding the oldest first (`null` for unlimited). |
