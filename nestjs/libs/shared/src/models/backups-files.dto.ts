import { Field, HideField, Int, ObjectType, registerEnumType } from '@nestjs/graphql';
import { ApiHideProperty, ApiProperty } from '@nestjs/swagger';
import {
  JsFileManifest,
  JsFileManifestAcl,
  JsFileManifestAclQualifier,
  JsFileManifestStat,
  JsFileManifestType,
  JsFileManifestXAttr,
} from '@woodstock/shared-rs';
import { Exclude, Expose, Transform } from 'class-transformer';

import { mangle } from '../utils';

export enum EnumFileType {
  RegularFile = 0,
  Symlink = 1,
  Directory = 2,
  BlockDevice = 3,
  CharacterDevice = 4,
  Fifo = 5,
  Socket = 6,
  Unknown = 99,
}

export enum FileManifestAclQualifier {
  Undefined = 0,
  UserObj = 1,
  GroupObj = 2,
  Other = 3,
  UserId = 4,
  GroupId = 5,
  Mask = 6,
}

registerEnumType(EnumFileType, {
  name: 'EnumFileType',
});

registerEnumType(FileManifestAclQualifier, {
  name: 'FileManifestAclQualifier',
});

@ObjectType()
export class FileStat {
  @ApiProperty({ type: () => Number })
  @Field(() => Int)
  ownerId: number;

  @ApiProperty({ type: () => Number })
  @Field(() => Int)
  groupId: number;

  @Transform(({ value }) => value?.toString())
  @ApiProperty({ type: () => String })
  @Field(() => String)
  size: bigint;

  @Transform(({ value }) => value?.toString())
  @ApiProperty({ type: () => String })
  @Field(() => String)
  compressedSize: bigint;

  @ApiProperty({ type: () => Number })
  @Field(() => String)
  lastRead: number;

  @ApiProperty({ type: () => Number })
  @Field(() => String)
  lastModified: number;

  @ApiProperty({ type: () => Number })
  @Field(() => String)
  created: number;

  @ApiProperty({ type: () => Number })
  @Field(() => Int)
  mode: number;

  @ApiProperty({ type: () => Number, enum: EnumFileType })
  @Field(() => EnumFileType)
  type: JsFileManifestType;

  @ApiProperty({ type: () => String })
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  dev: bigint;

  @ApiProperty({ type: () => String })
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  rdev: bigint;

  @ApiProperty({ type: () => String })
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  ino: bigint;

  @ApiProperty({ type: () => String })
  @Transform((v) => BigInt(v.value))
  @Field(() => BigInt)
  nlink: bigint;

  constructor(partial: Partial<JsFileManifestStat>) {
    Object.assign(this, partial);
  }
}

@ObjectType()
export class FileAcl implements JsFileManifestAcl {
  @ApiProperty({ type: () => FileManifestAclQualifier })
  @Field(() => FileManifestAclQualifier)
  qualifier: JsFileManifestAclQualifier;
  @ApiProperty({ type: () => Number })
  @Field(() => Int)
  id: number;
  @ApiProperty({ type: () => Number })
  @Field(() => Int)
  perm: number;

  constructor(partial: Partial<JsFileManifestAcl>) {
    Object.assign(this, partial);
  }
}

@ObjectType()
export class FileXattr implements JsFileManifestXAttr {
  @Transform(({ value }) => mangle(value))
  @ApiProperty({ type: () => String })
  @Field(() => String)
  key: Buffer;

  @Transform(({ value }) => mangle(value))
  @ApiProperty({ type: () => String })
  @Field(() => String)
  value: Buffer;

  constructor(partial: Partial<JsFileManifestXAttr>) {
    Object.assign(this, partial);
  }
}

@ObjectType()
export class FileDescription {
  @Transform(({ value }) => mangle(value))
  @ApiProperty({ type: () => String })
  @Field(() => String)
  path: Buffer;

  stats: FileStat | undefined;

  @Field(() => FileXattr)
  xattr: FileXattr[];

  @Field(() => FileAcl)
  acl: FileAcl[];

  @Transform(({ value }) => value && mangle(value))
  @ApiProperty({ type: () => String })
  @Field(() => String)
  symlink: Buffer;

  @Exclude()
  @HideField()
  @ApiHideProperty()
  chunks: Buffer[];

  @Exclude()
  @HideField()
  @ApiHideProperty()
  hash: Buffer;

  @Exclude()
  @HideField()
  @ApiHideProperty()
  metadata: Record<string, Buffer>;

  constructor(partial: JsFileManifest) {
    Object.assign(this, partial);
    this.stats = this.stats && new FileStat(this.stats);
    this.acl = this.acl && this.acl.map((acl) => new FileAcl(acl));
  }

  @Expose()
  @ApiProperty({ type: Number, enum: EnumFileType })
  @Field(() => EnumFileType)
  get type(): JsFileManifestType {
    return this.stats?.type ?? JsFileManifestType.Unknown;
  }
}
