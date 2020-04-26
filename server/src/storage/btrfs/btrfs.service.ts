import { Injectable, Logger } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import * as fs from 'fs';
import { join } from 'path';

import { ExecuteCommandService } from '../../operation/execute-command.service';
import { BtrfsCheck } from './btrfs.dto';
import { ApplicationConfigService } from '../../config/application-config.service';

@Injectable()
export class BtrfsService {
  private logger = new Logger(BtrfsService.name);

  constructor(private configService: ApplicationConfigService, private executeCommandService: ExecuteCommandService) {}

  async check(): Promise<BtrfsCheck> {
    const checks = new BtrfsCheck();
    // Is btrfs available on the backup directory
    {
      const { stdout } = await this.executeCommandService.executeCommand(
        `stat -f --format=%T ${this.configService.hostPath}`,
      );
      checks.backupVolume = this.configService.hostPath;
      checks.backupVolumeFileSystem = `${stdout}`.trim();
      checks.isBtrfsVolume = checks.backupVolumeFileSystem === 'btrfs';
    }

    {
      const { stdout } = await this.executeCommandService.executeCommand(`btrfs version`);
      checks.toolsAvailable.btrfstools = stdout.startsWith('btrfs-progs');
    }

    {
      try {
        const tmpVolume = join(this.configService.hostPath, '__tmp');
        await this.createSnapshot(tmpVolume);
        await this.removeSnapshot(tmpVolume);
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

  async createSnapshot(nextBackup: string, previousBackup?: string) {
    try {
      await fs.promises.access(nextBackup);

      this.logger.warn(`Directory ${nextBackup} already exists`);
    } catch (err) {
      if (err && err.code === 'ENOENT') {
        if (previousBackup) {
          this.logger.debug(`Create snapshot ${nextBackup} from ${previousBackup}`);
          await this.executeCommandService.executeCommand(
            `btrfs subvolume snapshot "${previousBackup}" "${nextBackup}"`,
          );
        } else {
          this.logger.debug(`Create first volume ${nextBackup}`);
          await this.executeCommandService.executeCommand(`btrfs subvolume create "${nextBackup}"`);
        }
      } else {
        throw err;
      }
    }
  }

  async removeSnapshot(path: string) {
    await this.markReadWrite(path);
    await this.executeCommandService.executeCommand(`btrfs subvolume delete "${path}"`);
  }

  async markReadOnly(path: string) {
    await this.executeCommandService.executeCommand(`btrfs property set -ts "${path}" ro true`);
  }

  async markReadWrite(path: string) {
    await this.executeCommandService.executeCommand(`btrfs property set -ts "${path}" ro false`);
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
