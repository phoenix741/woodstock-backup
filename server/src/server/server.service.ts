import { Injectable } from '@nestjs/common';
import * as fs from 'fs';
import { join } from 'path';

import { ExecuteCommandService } from '../operation/execute-command.service';
import { BtrfsService } from '../storage/btrfs/btrfs.service';
import { ServerChecks, CommandCheck } from './server.dto';
import { ToolsService } from './tools.service';

@Injectable()
export class ServerService {
  constructor(
    private toolsService: ToolsService,
    private executeCommandService: ExecuteCommandService,
    private btrfsService: BtrfsService,
  ) {}

  async check(): Promise<ServerChecks> {
    const checks = new ServerChecks();
    // Is btrfs available on the backup directory

    checks.push(...(await this.checkGetFilesystem()));
    checks.push(...(await this.checkPing()));
    checks.push(...(await this.checkResolveNetbiosFromHostname()));
    checks.push(...(await this.checkResolveNetbiosFromIP()));
    checks.push(...(await this.checkStatsSpaceUsage()));
    checks.push(...(await this.checkBtrfsQGroupEnable()));
    checks.push(...(await this.checkStatsDiskUsage()));
    checks.push(...(await this.checkBtrfsCreateSubvolume()));
    checks.push(...(await this.checkBtrfsCreateSnapshot()));
    checks.push(...(await this.checkBtrfsMarkROSubvolume()));
    checks.push(...(await this.checkBtrfsMarkRWSubvolume()));
    checks.push(...(await this.checkBtrfsListSubvolume()));
    checks.push(...(await this.checkBtrfsGetCompressionSize()));
    checks.push(...(await this.checkBtrfsDeleteSnapshot(98)));
    checks.push(...(await this.checkBtrfsDeleteSnapshot(99)));
    return checks;
  }

  async checkGetFilesystem(): Promise<CommandCheck[]> {
    const command = await this.toolsService.getCommand('getFilesystem', {});
    const { stdout, stderr } = await this.executeCommandService.executeCommand(command, {
      returnCode: true,
    });
    return [
      {
        command,
        isValid: `${stdout}`.trim() === 'btrfs',
        error: stderr,
      },
    ];
  }

  async checkPing(): Promise<CommandCheck[]> {
    const command = await this.toolsService.getCommand('ping', {
      ip: '127.0.0.1',
    });
    const { code, stderr } = await this.executeCommandService.executeCommand(command, {
      returnCode: true,
    });
    return [{ command, isValid: !code, error: stderr }];
  }

  async checkResolveNetbiosFromHostname(): Promise<CommandCheck[]> {
    const command = await this.toolsService.getCommand('resolveNetbiosFromHostname', { hostname: 'localhost' });
    const { code, stderr } = await this.executeCommandService.executeCommand(command, {
      returnCode: true,
    });
    return [{ command, isValid: code !== 127, error: stderr }];
  }

  async checkResolveNetbiosFromIP(): Promise<CommandCheck[]> {
    const command = await this.toolsService.getCommand('resolveNetbiosFromIP', {
      ip: '127.0.0.1',
    });
    const { code, stderr } = await this.executeCommandService.executeCommand(command, {
      returnCode: true,
    });
    return [{ command, isValid: code !== 127, error: stderr }];
  }

  async checkStatsSpaceUsage(): Promise<CommandCheck[]> {
    const command = await this.toolsService.getCommand('statsSpaceUsage', {});
    const { code, stderr } = await this.executeCommandService.executeCommand(command, {
      returnCode: true,
    });
    return [{ command, isValid: !code, error: stderr }];
  }

  async checkBtrfsQGroupEnable(): Promise<CommandCheck[]> {
    const command = await this.toolsService.getCommand('btrfsQGroupEnable', {});
    const { code, stderr } = await this.executeCommandService.executeCommand(command, {
      returnCode: true,
    });
    return [{ command, isValid: !code, error: stderr }];
  }

  async checkStatsDiskUsage(): Promise<CommandCheck[]> {
    const command = await this.toolsService.getCommand('statsDiskUsage', {});
    const { code, stderr } = await this.executeCommandService.executeCommand(command, {
      returnCode: true,
    });
    return [{ command, isValid: !code, error: stderr }];
  }

  async checkBtrfsCreateSubvolume(): Promise<CommandCheck[]> {
    try {
      const path = await this.toolsService.getPath('destBackupPath', { hostname: '__tmp', destBackupNumber: 99 });
      await this.btrfsService.createSnapshot({ hostname: '__tmp', destBackupNumber: 99 });

      await fs.promises.writeFile(join(path, 'file'), 'data', 'utf-8');

      return [{ command: 'createSubvolume', isValid: true }];
    } catch (err) {
      return [{ command: 'createSubvolume', isValid: false, error: err.message }];
    }
  }

  async checkBtrfsCreateSnapshot(): Promise<CommandCheck[]> {
    try {
      await this.btrfsService.createSnapshot({ hostname: '__tmp', destBackupNumber: 98, srcBackupNumber: 99 });

      return [{ command: 'createSnapshot', isValid: true }];
    } catch (err) {
      return [{ command: 'createSnapshot', isValid: false, error: err.message }];
    }
  }

  async checkBtrfsMarkROSubvolume(): Promise<CommandCheck[]> {
    try {
      await this.btrfsService.markReadOnly({ hostname: '__tmp', destBackupNumber: 99 });

      return [{ command: 'markReadOnly', isValid: true }];
    } catch (err) {
      return [{ command: 'markReadOnly', isValid: false, error: err.message }];
    }
  }

  async checkBtrfsMarkRWSubvolume(): Promise<CommandCheck[]> {
    try {
      await this.btrfsService.markReadWrite({ hostname: '__tmp', destBackupNumber: 99 });

      return [{ command: 'markReadWrite', isValid: true }];
    } catch (err) {
      return [{ command: 'markReadWrite', isValid: false, error: err.message }];
    }
  }

  async checkBtrfsDeleteSnapshot(destBackupNumber: number): Promise<CommandCheck[]> {
    try {
      await this.btrfsService.removeSnapshot({ hostname: '__tmp', destBackupNumber });

      return [{ command: 'removeSnapshot', isValid: true }];
    } catch (err) {
      return [{ command: 'removeSnapshot', isValid: false, error: err.message }];
    }
  }

  async checkBtrfsListSubvolume(): Promise<CommandCheck[]> {
    const command = await this.toolsService.getCommand('btrfsListSubvolume', {});
    const { code, stderr } = await this.executeCommandService.executeCommand(command, {
      returnCode: true,
    });
    return [{ command, isValid: !code, error: stderr }];
  }

  async checkBtrfsGetCompressionSize(): Promise<CommandCheck[]> {
    const command = await this.toolsService.getCommand('btrfsGetCompressionSize', {
      hostname: '__tmp',
      destBackupNumber: 99,
    });
    const { code, stderr } = await this.executeCommandService.executeCommand(command, {
      returnCode: true,
    });
    return [{ command, isValid: !code, error: stderr }];
  }
}
