import { Field, HideField, ObjectType, registerEnumType } from '@nestjs/graphql';
import { ApiHideProperty, ApiProperty } from '@nestjs/swagger';
import { longToBigInt, mangle } from '@woodstock/core';
import { FileBrowserService, FileManifest, FileManifestAcl, FileManifestStat } from '@woodstock/shared';
import { Exclude, Expose, Transform } from 'class-transformer';
import * as Long from 'long';

export enum EnumFileType {
  SHARE = 'SHARE',
  BLOCK_DEVICE = 'BLOCK_DEVICE',
  CHARACTER_DEVICE = 'CHARACTER_DEVICE',
  DIRECTORY = 'DIRECTORY',
  FIFO = 'FIFO',
  REGULAR_FILE = 'REGULAR_FILE',
  SOCKET = 'SOCKET',
  SYMBOLIC_LINK = 'SYMBOLIC_LINK',
  UNKNOWN = 'UNKNOWN',
}

registerEnumType(EnumFileType, {
  name: 'EnumFileType',
});

@ObjectType()
export class FileStat implements FileManifestStat {
  @Transform(({ value }) => value?.toString())
  @ApiProperty({ type: () => String })
  @Field(() => String)
  ownerId?: Long;
  @Transform(({ value }) => value?.toString())
  @ApiProperty({ type: () => String })
  @Field(() => String)
  groupId?: Long;
  @Transform(({ value }) => value?.toString())
  @ApiProperty({ type: () => String })
  @Field(() => String)
  size?: Long;
  @Transform(({ value }) => value?.toString())
  @ApiProperty({ type: () => String })
  @Field(() => String)
  compressedSize?: Long;
  @Transform(({ value }) => value?.toString())
  @ApiProperty({ type: () => String })
  @Field(() => String)
  lastRead?: Long;
  @Transform(({ value }) => value?.toString())
  @ApiProperty({ type: () => String })
  @Field(() => String)
  lastModified?: Long;
  @Transform(({ value }) => value?.toString())
  @ApiProperty({ type: () => String })
  @Field(() => String)
  created?: Long;
  @Transform(({ value }) => value?.toString())
  @ApiProperty({ type: () => String })
  @Field(() => String)
  mode?: Long;
  @Transform(({ value }) => value?.toString())
  @ApiProperty({ type: () => String })
  @Field(() => String)
  dev?: Long;
  @Transform(({ value }) => value?.toString())
  @ApiProperty({ type: () => String })
  @Field(() => String)
  rdev?: Long;
  @Transform(({ value }) => value?.toString())
  @ApiProperty({ type: () => String })
  @Field(() => String)
  ino?: Long;
  @Transform(({ value }) => value?.toString())
  @ApiProperty({ type: () => String })
  @Field(() => String)
  nlink?: Long;

  constructor(partial: Partial<FileManifestStat>) {
    Object.assign(this, partial);
  }
}

@ObjectType()
export class FileAcl implements FileManifestAcl {
  user?: string;
  group?: string;
  mask?: number;
  other?: number;

  constructor(partial: Partial<FileManifestAcl>) {
    Object.assign(this, partial);
  }
}

@ObjectType()
export class FileDescription implements FileManifest {
  @Transform(({ value }) => mangle(value))
  @ApiProperty({ type: () => String })
  @Field(() => String)
  path: Buffer;
  stats: FileStat | undefined;
  @Field(() => String) // FIXME: GraphQL type for JSON
  xattr: { [key: string]: Buffer };
  acl: FileAcl[];
  @Transform(({ value }) => value && mangle(value))
  @ApiProperty({ type: () => String })
  @Field(() => String)
  symlink?: Buffer | undefined;

  @Exclude()
  @HideField()
  @ApiHideProperty()
  chunks: Buffer[];
  @Exclude()
  @HideField()
  @ApiHideProperty()
  sha256?: Buffer | undefined;

  constructor(partial: Partial<FileManifest>) {
    Object.assign(this, partial);
    this.stats = this.stats && new FileStat(this.stats);
    this.acl = this.acl && this.acl.map((acl) => new FileAcl(acl));
  }

  @Expose()
  @Field(() => EnumFileType)
  get type(): EnumFileType {
    const m = longToBigInt(this.stats?.mode || Long.ZERO);
    if (m === -1n) {
      return EnumFileType.SHARE;
    } else if (FileBrowserService.isDirectory(m)) {
      return EnumFileType.DIRECTORY;
    } else if (FileBrowserService.isRegularFile(m)) {
      return EnumFileType.REGULAR_FILE;
    } else if (FileBrowserService.isSymLink(m)) {
      return EnumFileType.SYMBOLIC_LINK;
    } else if (FileBrowserService.isSocket(m)) {
      return EnumFileType.SOCKET;
    } else if (FileBrowserService.isBlockDevice(m)) {
      return EnumFileType.BLOCK_DEVICE;
    } else if (FileBrowserService.isCharacterDevice(m)) {
      return EnumFileType.CHARACTER_DEVICE;
    } else if (FileBrowserService.isFIFO(m)) {
      return EnumFileType.FIFO;
    } else {
      return EnumFileType.UNKNOWN;
    }
  }
}
