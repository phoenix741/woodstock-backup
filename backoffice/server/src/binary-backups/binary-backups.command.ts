import { Processor } from '@nestjs/bull';
import { HttpService } from '@nestjs/common';
import { Command, Console } from 'nestjs-console';

import { ApplicationConfigService } from '../config/application-config.service';
import { PoolService } from '../storage/pool/pool.service';
import { BinaryBackupsService } from './binary-backups.service';

@Console({
  name: 'binary-backups',
})
@Processor('queue')
export class BinaryBackupsCommand {
  constructor(
    private configService: ApplicationConfigService,
    private poolService: PoolService,
    private httpService: HttpService,
  ) {}

  @Command({
    command: 'backup',
    description: 'launch backup',
  })
  async launchBackup(): Promise<void> {
    const service = new BinaryBackupsService(
      this.httpService,
      this.configService,
      this.poolService,
      'pc-ulrich.eden.lan',
      -1,
      0,
    );
    await service.start();
  }
}
