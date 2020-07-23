import { Injectable, Logger } from '@nestjs/common';
import * as fs from 'fs';

import { ApplicationConfigService } from '../../config/application-config.service';
import { ExecuteCommandService } from '../../operation/execute-command.service';
import { CommandParameters } from '../../server/tools.model';
import { ToolsService } from '../../server/tools.service';
import { YamlService } from '../../utils/yaml.service';
import { BtrfsCheck } from './btrfs.dto';
import { cp } from 'shelljs';

@Injectable()
export class BtrfsService {
  private logger = new Logger(BtrfsService.name);

  constructor(
    private configService: ApplicationConfigService,
    private toolsService: ToolsService,
    private executeCommandService: ExecuteCommandService,
    private yamlService: YamlService,
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

  /**
   * Create a qgroup for the host.
   * @param params All know parameters
   */
  private async createQGroupForHost(params: CommandParameters): Promise<number> {
    const qGroupHostPath = await this.toolsService.getPath('qgroupHostPath', params);
    const qGroupId = (Math.random() * 4294967296) >>> 0;

    try {
      await this.executeCommandService.executeTool('btrfsBackupQGroupCreate', { qGroupId, ...params });

      await this.yamlService.writeFile(qGroupHostPath, qGroupId);

      return qGroupId;
    } catch (err) {
      // Try to replay one time
      if (!params.qGroupId) {
        return await this.createQGroupForHost({ qGroupId, ...params });
      } else {
        throw err;
      }
    }
  }

  async getHostGroupId(params: CommandParameters) {
    try {
      const qGroupHostPath = await this.toolsService.getPath('qgroupHostPath', params);
      this.logger.debug(`Get the qGroup for host ${params.hostname} at ${qGroupHostPath}`);

      await fs.promises.access(qGroupHostPath);

      // The qgroup file exist, we can get the qgroup content
      return await this.yamlService.loadFile<number>(qGroupHostPath, -1);
    } catch (err) {
      if (err && err.code === 'ENOENT') {
        return -1;
      } else {
        throw err;
      }
    }
  }

  private async getQGroupOfHost(params: CommandParameters) {
    const value = await this.getHostGroupId(params);
    if (value < 0) {
      return await this.createQGroupForHost(params);
    }
  }

  async createSnapshot(params: CommandParameters) {
    const qGroupId = await this.getQGroupOfHost(params);

    try {
      const destBackupNumber = await this.toolsService.getPath('destBackupPath', { qGroupId, ...params });
      await fs.promises.access(destBackupNumber);

      // Crash when previous backup, mark the the backup readable
      await this.markReadWrite({ qGroupId, ...params });

      this.logger.warn(`Directory ${destBackupNumber} already exists`);
    } catch (err) {
      if (err && err.code === 'ENOENT') {
        if (params.srcBackupNumber !== undefined) {
          this.logger.debug(
            `Create snapshot ${params.hostname}/${params.destBackupNumber} from ${params.hostname}/${params.srcBackupNumber} (assigned to qGroup ${qGroupId})`,
          );
          await this.executeCommandService.executeTool('btrfsCreateSnapshot', { qGroupId, ...params });
        } else {
          this.logger.debug(`Create first volume ${params.hostname}/${params.destBackupNumber}`);
          await this.executeCommandService.executeTool('btrfsCreateSubvolume', { qGroupId, ...params });
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
}
