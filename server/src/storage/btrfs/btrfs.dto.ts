import { ObjectType, Field } from '@nestjs/graphql';
import { ApiProperty } from '@nestjs/swagger';

@ObjectType()
export class BtrfsCheckTools {
  @ApiProperty({ type: Boolean })
  @Field(() => Boolean)
  btrfstools?: boolean;

  @ApiProperty({ type: Boolean })
  @Field(() => Boolean)
  compsize?: boolean;
}

@ObjectType()
export class BtrfsCheck {
  @ApiProperty({ type: Boolean })
  @Field(() => Boolean)
  isBtrfsVolume?: boolean;

  backupVolume?: string;
  backupVolumeFileSystem?: string;

  @ApiProperty({ type: Boolean })
  @Field(() => Boolean)
  hasAuthorization?: boolean;

  toolsAvailable: BtrfsCheckTools = {};
}
