import { Controller, Get } from '@nestjs/common';
import { BtrfsService } from '../storage/btrfs/btrfs.service';
import { ApiOkResponse } from '@nestjs/swagger';
import { BtrfsCheck } from '../storage/btrfs/btrfs.dto';

@Controller('server')
export class ServerController {
  constructor(public btrfsService: BtrfsService) {}

  @Get('status')
  @ApiOkResponse({
    description: 'Get the status of the server',
    type: BtrfsCheck,
  })
  async getStatus() {
    return this.btrfsService.check();
  }
}
