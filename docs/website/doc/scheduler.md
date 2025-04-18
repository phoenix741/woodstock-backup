# Updating the Scheduler

You can update the default scheduler configuration instead of modifying individual host schedulers.

```yaml
wakeupSchedule: "*/15 * * * *"
nightlySchedule: "0 0 * * *"
defaultSchedule:
  activated: True
  backupPeriod: 86100
  backupToKeep:
    hourly: -1
    daily: 7
    weekly: 4
    monthly: 12
    yearly: -1
```

## The Application Scheduler

Inside the configuration file, you have the following properties:

| Field           | Default value  | Description                                                           |
| --------------- | -------------  | --------------------------------------------------------------------- |
| wakeupSchedule  | `*/15 * * * *` | Cron expression to wake up the backup software and launch all necessary backups |
| nightlySchedule | `0 0 * * *`    | Cron expression to wake up the backup software and generate statistics        |
| defaultSchedule | See above      | The default backup scheduler configuration                                     |

## The Scheduler

Inside the `scheduler` field:

| Field        | Default value                                                  | Description                                  |
| ------------ | -------------------------------------------------------------- | -------------------------------------------- |
| activated    | true                                                           | Enable or disable automatic backups          |
| backupPeriod | 8340                                                           | Period between two backups in minutes (24H - 5 minutes) |
| backupToKeep | `{ hourly: 24, daily: 7, weekly: 4, monthly: 12, yearly: 1 }`  | Number of backups to keep in each category (not currently used) |
