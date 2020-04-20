import { ApiProperty } from '@nestjs/swagger';

export class BtrfsCheckTools {
  @ApiProperty()
  btrfstools?: boolean;
  @ApiProperty()
  compsize?: boolean;
}

export class BtrfsCheck {
  @ApiProperty()
  isBtrfsVolume?: boolean;

  @ApiProperty()
  backupVolume?: string;

  @ApiProperty()
  backupVolumeFileSystem?: string;

  @ApiProperty()
  hasAuthorization?: boolean;

  @ApiProperty({ type: BtrfsCheckTools })
  toolsAvailable: BtrfsCheckTools = {};
}
