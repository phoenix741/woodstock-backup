import { ApiExtraModels, ApiProperty, getSchemaPath } from '@nestjs/swagger';
import { IsNotEmpty, IsNumber, Matches, Max, Min, ValidateNested } from 'class-validator';
import { Schedule } from '../scheduler/scheduler.dto';

/**
 * Part of config file.
 *
 * Store information about a share
 */
export class BackupTaskShare {
  @ApiProperty({ example: '/home' })
  name!: string;

  @ApiProperty({ example: ['/home/user'] })
  includes?: string[];

  @ApiProperty({ example: ['*.bak'] })
  excludes?: string[];
}

/**
 * Part of config file
 *
 * Store information about a DHCP Address
 */
export class DhcpAddress {
  @ApiProperty({ example: '192.168.101' })
  @Matches(/^(([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\.){2}([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])$/)
  address!: string;

  @ApiProperty({ example: 0 })
  @Min(0)
  @Max(255)
  start!: number;

  @ApiProperty({ example: 50 })
  @Min(0)
  @Max(255)
  end!: number;
}

export class ExecuteCommandOperation {
  @ApiProperty({ type: String, enum: ['ExecuteCommand'] })
  name!: 'ExecuteCommand';

  @ApiProperty({ example: '/bin/true' })
  command!: string;
}

export class RSyncBackupOperation {
  @ApiProperty({ type: String, enum: ['RSyncBackup'] })
  name!: 'RSyncBackup';

  @ValidateNested()
  share!: Array<BackupTaskShare>;

  @ApiProperty({ example: [] })
  includes?: Array<string>;

  @ApiProperty({ example: [] })
  excludes?: Array<string>;

  @ApiProperty({ example: 1200 })
  @IsNumber()
  timeout!: number;
}

export class RSyncdBackupOperation {
  @ApiProperty({ type: String, enum: ['RSyncdBackup'] })
  name!: 'RSyncdBackup';

  @ApiProperty({ type: Boolean })
  authentification?: boolean;

  username?: string;

  password?: string;

  @ValidateNested()
  share!: Array<BackupTaskShare>;

  @ApiProperty({ example: [] })
  includes?: Array<string>;

  @ApiProperty({ example: [] })
  excludes?: Array<string>;

  @ApiProperty({ example: 1200 })
  @IsNumber()
  timeout!: number;
}

export type Operation = ExecuteCommandOperation | RSyncBackupOperation | RSyncdBackupOperation;

@ApiExtraModels(RSyncBackupOperation, ExecuteCommandOperation, RSyncdBackupOperation)
export class HostConfigOperation {
  @ApiProperty({
    type: 'array',
    items: {
      oneOf: [{ $ref: getSchemaPath(ExecuteCommandOperation) }, { $ref: getSchemaPath(RSyncBackupOperation) }],
    },
  })
  @ValidateNested()
  tasks: Operation[] = [];

  @ApiProperty({
    type: 'array',
    items: {
      oneOf: [{ $ref: getSchemaPath(ExecuteCommandOperation) }, { $ref: getSchemaPath(RSyncBackupOperation) }],
    },
  })
  @ValidateNested()
  finalizeTasks: Operation[] = [];
}

/**
 * Config file for one Host
 *
 * Contains all information that can be used to backup a host.
 */
export class HostConfig {
  @ApiProperty({ example: 'pc-ulrich' })
  @IsNotEmpty()
  name!: string;

  @ApiProperty({ example: [] })
  addresses?: string[];

  @ValidateNested()
  dhcp?: DhcpAddress[];

  @ValidateNested()
  operations = new HostConfigOperation();

  @ValidateNested()
  schedule = new Schedule();
}
