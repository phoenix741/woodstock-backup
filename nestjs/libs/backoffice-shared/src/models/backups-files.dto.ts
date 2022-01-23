import { Field, ObjectType, registerEnumType } from '@nestjs/graphql';
import { ApiProperty } from '@nestjs/swagger';

export enum EnumFileType {
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
export class FileDescription {
  @ApiProperty({ type: String })
  @Field(() => String)
  name!: Buffer;
  type!: EnumFileType;

  dev!: number;
  ino!: number;
  mode!: number;
  nlink!: number;
  uid!: number;
  gid!: number;
  rdev!: number;
  size!: number;
  blksize!: number;
  blocks!: number;
  atimeMs!: number;
  mtimeMs!: number;
  ctimeMs!: number;
  birthtimeMs!: number;
  atime!: Date;
  mtime!: Date;
  ctime!: Date;
  birthtime!: Date;
}
