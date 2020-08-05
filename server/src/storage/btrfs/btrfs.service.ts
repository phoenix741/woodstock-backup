import { Injectable, Logger } from '@nestjs/common';
import * as fs from 'fs';

import { ExecuteCommandService } from '../../operation/execute-command.service';
import { CommandParameters } from '../../server/tools.model';
import { ToolsService } from '../../server/tools.service';
import { YamlService } from '../../utils/yaml.service';

@Injectable()
export class BtrfsService {
  private logger = new Logger(BtrfsService.name);

  constructor(
    private toolsService: ToolsService,
    private executeCommandService: ExecuteCommandService,
    private yamlService: YamlService,
  ) {}

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
    return value;
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
    const lines = await this.listSubvolume(params);
    await this.markReadWrite(params);
    await this.executeCommandService.executeTool('btrfsDeleteSnapshot', params);
    await this.removeQGroup(lines, params);
  }

  private async removeQGroup(
    lines: {
      id: number;
      host: string;
      number: number;
    }[],
    params: CommandParameters,
  ) {
    const line = lines.find(line => params.hostname === line.host && params.destBackupNumber === line.number);
    const qGroupId = line?.id;
    if (qGroupId) {
      await this.executeCommandService.executeTool('btrfsBackupQGroupDestroy', {
        ...params,
        qGroupId,
      });
    }
  }

  async markReadOnly(params: CommandParameters) {
    await this.executeCommandService.executeTool('btrfsMarkROSubvolume', params);
  }

  async markReadWrite(params: CommandParameters) {
    await this.executeCommandService.executeTool('btrfsMarkRWSubvolume', params);
  }

  async listSubvolume(params: CommandParameters) {
    const { stdout } = await this.executeCommandService.executeTool('btrfsListSubvolume', params);
    const [, , ...lines] = stdout.toString().split(/[\n\r]/);

    return lines
      .map(line => {
        if (line) {
          const [id, , , path] = line.split(/\s+/).filter(n => !!n);

          const pathArray = path.split('/').slice(-2);
          const host = pathArray[0];
          const number = parseInt(pathArray[1]);

          return { id: parseInt(id), host, number };
        }
        return { id: -1, host: '', number: -1 };
      })
      .filter(l => l.id >= 0);
  }
}
