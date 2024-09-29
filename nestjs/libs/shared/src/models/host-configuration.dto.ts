import { Field, ObjectType } from '@nestjs/graphql';
import { ApiExtraModels, ApiProperty } from '@nestjs/swagger';
import { Exclude } from 'class-transformer';
import { IsNumber, ValidateNested } from 'class-validator';

import { Schedule } from '../config';

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
}

@ObjectType()
export class ExecuteCommandOperation {
  @ApiProperty({ example: '/bin/true' })
  command!: string;
}

@ObjectType()
export class BackupOperation {
  @ValidateNested()
  shares!: Array<BackupTaskShare>;

  @ApiProperty({ example: [] })
  includes?: Array<string>;

  @ApiProperty({ example: [] })
  excludes?: Array<string>;

  @ApiProperty({ example: 1200 })
  @IsNumber()
  timeout?: number;
}

@ApiExtraModels(BackupOperation, ExecuteCommandOperation)
@ObjectType()
export class HostConfigOperation {
  @ApiProperty({ type: [ExecuteCommandOperation] })
  @ValidateNested()
  @Field(() => [ExecuteCommandOperation])
  preCommands?: ExecuteCommandOperation[];

  @ApiProperty({ type: BackupOperation })
  @ValidateNested()
  @Field(() => BackupOperation)
  operation?: BackupOperation;

  @ApiProperty({ type: [ExecuteCommandOperation] })
  @ValidateNested()
  @Field(() => [ExecuteCommandOperation])
  postCommands?: ExecuteCommandOperation[];
}

/**
 * Config file for one Host
 *
 * Contains all information that can be used to backup a host.
 */
@ObjectType()
export class HostConfiguration {
  @ApiProperty({ type: Boolean })
  @Field(() => Boolean)
  isLocal?: boolean;

  @Exclude()
  password: string;

  /**
   * Max number of concurrent downloads for this host. By default, it's 1.
   */
  @ApiProperty({ example: 1 })
  maxConcurrentDownloads?: number;

  @ApiProperty({ example: [] })
  addresses?: string[];

  @ApiProperty({ example: 5678 })
  port: number;

  @ValidateNested()
  operations?: HostConfigOperation = new HostConfigOperation();

  @ValidateNested()
  schedule?: Schedule;
}
