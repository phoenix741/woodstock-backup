import { Injectable, Logger } from '@nestjs/common';
import * as fs from 'fs';

import { ApplicationConfigService } from '../../config/application-config.service';
import { ExecuteCommandService } from '../../operation/execute-command.service';
import { CommandParameters } from '../../server/tools.model';
import { BtrfsCheck } from './btrfs.dto';
import { ToolsService } from '../../server/tools.service';

@Injectable()
export class BtrfsService {
  private logger = new Logger(BtrfsService.name);

  constructor(
    private configService: ApplicationConfigService,
    private toolsService: ToolsService,
    private executeCommandService: ExecuteCommandService,
  ) {}

  async check(): Promise<BtrfsCheck> {
    const checks = new BtrfsCheck();
    // Is btrfs available on the backup directory
    {
      const { stdout } = await this.executeCommandService.executeTool('getFilesystem', {});
      checks.backupVolume = this.configService.hostPath;
      checks.backupVolumeFileSystem = `${stdout}`.trim();
      checks.isBtrfsVolume = checks.backupVolumeFileSystem === 'btrfs';
    }

    {
      const { stdout } = await this.executeCommandService.executeTool('btrfsVersion', {});
      checks.toolsAvailable.btrfstools = stdout.startsWith('btrfs-progs');
    }

    {
      try {
        await this.createSnapshot({ hostname: '__tmp', destBackupNumber: 0 });
        await this.removeSnapshot({ hostname: '__tmp', destBackupNumber: 0 });
        checks.hasAuthorization = true;
      } catch (err) {
        checks.hasAuthorization = false;
      }
    }

    {
      try {
        await this.executeCommandService.executeCommand(`which compsize`);
        checks.toolsAvailable.compsize = true;
      } catch (err) {
        checks.toolsAvailable.compsize = false;
      }
    }

    return checks;
  }

  async createSnapshot(params: CommandParameters) {
    try {
      const destBackupNumber = await this.toolsService.getPath('destBackupPath', params);
      await fs.promises.access(destBackupNumber);

      // Crash when previous backup, mark the the backup readable
      await this.markReadWrite(params);

      this.logger.warn(`Directory ${destBackupNumber} already exists`);
    } catch (err) {
      if (err && err.code === 'ENOENT') {
        if (params.srcBackupNumber !== undefined) {
          this.logger.debug(
            `Create snapshot ${params.hostname}/${params.destBackupNumber} from ${params.hostname}/${params.srcBackupNumber}`,
          );
          await this.executeCommandService.executeTool('btrfsCreateSnapshot', params);
        } else {
          this.logger.debug(`Create first volume ${params.hostname}/${params.destBackupNumber}`);
          await this.executeCommandService.executeTool('btrfsCreateSubvolume', params);
        }
      } else {
        throw err;
      }
    }
  }

  async removeSnapshot(params: CommandParameters) {
    await this.markReadWrite(params);
    await this.executeCommandService.executeTool('btrfsDeleteSnapshot', params);
  }

  async markReadOnly(params: CommandParameters) {
    await this.executeCommandService.executeTool('btrfsMarkROSubvolume', params);
  }

  async markReadWrite(params: CommandParameters) {
    await this.executeCommandService.executeTool('btrfsMarkRWSubvolume', params);
  }

  async stats() {
    const { stdout } = await this.executeCommandService.executeCommand(
      `btrfs filesystem du --raw -s "${this.configService.hostPath}"`,
    );
    const [, line] = stdout.toString().split(/[\n\r]/);
    const [total, exclusive, shared] = line
      .split(/\s+/)
      .filter(n => !!n)
      .map(v => parseInt(v));

    return {
      total,
      exclusive,
      shared,
    };
  }
}
