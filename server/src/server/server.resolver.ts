import { Query, Resolver } from '@nestjs/graphql';

import { BtrfsCheck } from '../storage/btrfs/btrfs.dto';
import { BtrfsService } from '../storage/btrfs/btrfs.service';

@Resolver()
export class ServerResolver {
  constructor(public btrfsService: BtrfsService) {}

  @Query(() => BtrfsCheck)
  async status() {
    return this.btrfsService.check();
  }
}
