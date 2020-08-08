# Update the scheduler

You can update the default scheduler instead of host scheduler.

```yaml
---
wakeupSchedule: "0 * * * *"
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

## The application scheduler

Inside the file we have the following properties:

| Field           | Default value | Description                                                           |
| --------------- | ------------- | --------------------------------------------------------------------- |
| wakeupSchedule  | 0 \* \* \* \* | Cron to wakeup the backup software and launch all backup if necessary |
| nightlySchedule | 0 0 \* \* \*  | Cron to wakeup the backup software and launch statistics              |
| defaultSchedule | See above     | The default backup scheduler                                          |

## The scheduler

Inside the field `scheduler`:

| Field        | Default value                                                  | Description                                  |
| ------------ | -------------------------------------------------------------- | -------------------------------------------- |
| activated    | true                                                           | Active / Desactive the automatic backup      |
| backupPeriod | 8340                                                           | Period between two backup: 24H - 5 minutes   |
| backupToKeep | `{ hourly: -1, daily: 7, weekly: 4, monthly: 12, yearly: -1 }` | Number of backup to keep (not used actually) |
