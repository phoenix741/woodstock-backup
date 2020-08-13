import { ApiProperty } from '@nestjs/swagger';
import { ObjectType } from '@nestjs/graphql';

@ObjectType()
export class ScheduledBackupToKeep {
  hourly = -1;
  daily = 7;
  weekly = 4;
  monthly = 12;
  yearly = -1;
}
@ObjectType()
export class Schedule {
  @ApiProperty({ type: Boolean })
  activated = true;
  backupPerdiod = 24 * 3600 - 5 * 60;
  backupToKeep = new ScheduledBackupToKeep();
}

export class ApplicationScheduler {
  wakeupSchedule = '0 * * * *';
  nightlySchedule = '0 0 * * *';
  defaultSchedule = new Schedule();
}
