import { ApiProperty } from '@nestjs/swagger';
import { ObjectType } from '@nestjs/graphql';

@ObjectType()
export class ScheduledBackupToKeep {
  hourly?: number;
  daily?: number;
  weekly?: number;
  monthly?: number;
  yearly?: number;
}
@ObjectType()
export class Schedule {
  activated?: boolean;
  backupPerdiod?: number;
  backupToKeep?: ScheduledBackupToKeep;
}

export class ApplicationScheduler {
  wakeupSchedule?: string;
  nightlySchedule?: string;
  defaultSchedule?: Schedule = new Schedule();

  constructor(s: Partial<ApplicationScheduler> = {}) {
    Object.assign(this, s);
  }
}
