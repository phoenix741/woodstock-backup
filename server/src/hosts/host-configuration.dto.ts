import { ObjectType, Field, createUnionType } from '@nestjs/graphql';
import { ApiExtraModels, ApiProperty, getSchemaPath } from '@nestjs/swagger';
import { IsNumber, Matches, Max, Min, ValidateNested } from 'class-validator';

import { Schedule } from '../scheduler/scheduler.dto';

/**
 * Part of config file.
 *
 * Store information about a share
 */
@ObjectType()
export class BackupTaskShare {
  @ApiProperty({ example: '/home' })
  name!: string;

  @ApiProperty({ example: ['/home/user'] })
  includes?: string[];

  @ApiProperty({ example: ['*.bak'] })
  excludes?: string[];

  @ApiProperty({ type: Boolean })
  @Field(() => Boolean)
  checksum?: boolean;
}

/**
 * Part of config file
 *
 * Store information about a DHCP Address
 */
@ObjectType()
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

@ObjectType()
export class ExecuteCommandOperation {
  @ApiProperty({ type: String, enum: ['ExecuteCommand'] })
  @Field(() => String)
  name!: 'ExecuteCommand';

  @ApiProperty({ example: '/bin/true' })
  command!: string;
}

@ObjectType()
export class RSyncBackupOperation {
  @ApiProperty({ type: String, enum: ['RSyncBackup'] })
  @Field(() => String)
  name!: 'RSyncBackup';

  @ValidateNested()
  share!: Array<BackupTaskShare>;

  @ApiProperty({ example: [] })
  includes?: Array<string>;

  @ApiProperty({ example: [] })
  excludes?: Array<string>;

  @ApiProperty({ example: 1200 })
  @IsNumber()
  timeout?: number;
}

@ObjectType()
export class RSyncdBackupOperation {
  @ApiProperty({ type: String, enum: ['RSyncdBackup'] })
  @Field(() => String)
  name!: 'RSyncdBackup';

  @ApiProperty({ type: Boolean })
  @Field(() => Boolean)
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
  timeout?: number;
}

export type Operation = ExecuteCommandOperation | RSyncBackupOperation | RSyncdBackupOperation;

export const OperationUnion = createUnionType({
  name: 'Operation',
  types: () => [ExecuteCommandOperation, RSyncBackupOperation, RSyncdBackupOperation],
  resolveType(value) {
    if (value.command) {
      return ExecuteCommandOperation;
    } else if (value.authentification) {
      return RSyncdBackupOperation;
    } else if (value.share) {
      return RSyncBackupOperation;
    }
    return null;
  },
});

@ApiExtraModels(RSyncBackupOperation, ExecuteCommandOperation, RSyncdBackupOperation)
@ObjectType()
export class HostConfigOperation {
  @ApiProperty({
    type: 'array',
    items: {
      oneOf: [{ $ref: getSchemaPath(ExecuteCommandOperation) }, { $ref: getSchemaPath(RSyncBackupOperation) }],
    },
  })
  @ValidateNested()
  @Field(() => [OperationUnion])
  tasks?: Operation[];

  @ApiProperty({
    type: 'array',
    items: {
      oneOf: [{ $ref: getSchemaPath(ExecuteCommandOperation) }, { $ref: getSchemaPath(RSyncBackupOperation) }],
    },
  })
  @ValidateNested()
  @Field(() => [OperationUnion])
  finalizeTasks?: Operation[];
}

/**
 * Config file for one Host
 *
 * Contains all information that can be used to backup a host.
 */
@ObjectType()
export class HostConfiguration {
  @ApiProperty({ example: [] })
  addresses?: string[];

  @ValidateNested()
  dhcp?: DhcpAddress[];

  @ValidateNested()
  operations? = new HostConfigOperation();

  @ValidateNested()
  schedule?: Schedule;
}
