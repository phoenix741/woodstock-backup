import { HostConfiguration } from '../hosts/host-configuration.dto';
import { ApiProperty } from '@nestjs/swagger';

export enum BackupState {
  WAITING = 'WAITING',
  RUNNING = 'RUNNING',
  SUCCESS = 'SUCCESS',
  ABORTED = 'ABORTED',
  FAILED = 'FAILED',
}

export class TaskProgression {
  fileSize = 0;
  newFileSize = 0;

  newFileCount = 0;
  fileCount = 0;

  speed = 0;
  percent = 0;
}

export class BackupSubTask {
  context!: string;
  description!: string;
  state!: BackupState;
  progression?: TaskProgression;
}

export class BackupTask {
  host!: string;
  config?: HostConfiguration;
  number?: number;
  ip?: string;
  previousDirectory?: string;
  destinationDirectory?: string;
  startDate?: Date;

  subtasks?: BackupSubTask[];
  state?: BackupState;
  progression?: TaskProgression;

  @ApiProperty({ type: Boolean })
  complete?: boolean;
}
