import { Injectable, Logger } from '@nestjs/common';
import { ExecuteCommandService, ToolsService } from '@woodstock/shared';
import { mkdir } from 'fs/promises';
import { CommandCheck, ServerChecks } from './server.dto.js';
import { ApplicationConfigService } from '@woodstock/shared';

@Injectable()
export class ServerService {
  private logger = new Logger(ServerService.name);

  constructor(
    private applicationConfig: ApplicationConfigService,
    private toolsService: ToolsService,
    private executeCommandService: ExecuteCommandService,
  ) {}

  async check(): Promise<ServerChecks> {
    await mkdir(this.applicationConfig.poolPath, { recursive: true });
    await mkdir(this.applicationConfig.backupPath, { recursive: true });
    await mkdir(this.applicationConfig.hostPath, { recursive: true });
    await mkdir(this.applicationConfig.configPath, { recursive: true });
    await mkdir(this.applicationConfig.logPath, { recursive: true });

    const checks = new ServerChecks();

    checks.push(() => this.checkGetFilesystem());
    checks.push(() => this.checkPing());
    checks.push(() => this.checkResolveNetbiosFromHostname());
    checks.push(() => this.checkResolveNetbiosFromIP());
    checks.push(() => this.checkStatsSpaceUsage());
    checks.push(() => this.checkStatsDiskUsage());
    return checks;
  }

  async executeChecks(): Promise<boolean> {
    const checks = await this.check();
    let checksCmd = true;
    for (const check of checks.commands) {
      const commandCheck = await check();
      if (commandCheck.isValid) {
        this.logger.debug(`${commandCheck.command} : OK`, ServerService.name);
      } else {
        checksCmd = false;
        this.logger.error(`${commandCheck.command} : KO - ${commandCheck.error}`, ServerService.name);
      }
    }

    return checksCmd;
  }

  async checkGetFilesystem(): Promise<CommandCheck> {
    const command = await this.toolsService.getCommand('getFilesystem', {});
    const { stdout, stderr } = await this.executeCommandService.executeCommand(command, {
      returnCode: true,
    });
    return {
      command,
      isValid: `${stdout}`.trim() === 'btrfs',
      error: stderr,
    };
  }

  async checkPing(): Promise<CommandCheck> {
    const command = await this.toolsService.getCommand('ping', {
      ip: '127.0.0.1',
    });
    const { code, stderr } = await this.executeCommandService.executeCommand(command, {
      returnCode: true,
    });
    return { command, isValid: !code, error: stderr };
  }

  async checkResolveNetbiosFromHostname(): Promise<CommandCheck> {
    const command = await this.toolsService.getCommand('resolveNetbiosFromHostname', { hostname: 'localhost' });
    const { code, stderr } = await this.executeCommandService.executeCommand(command, {
      returnCode: true,
    });
    return { command, isValid: code !== 127, error: stderr };
  }

  async checkResolveNetbiosFromIP(): Promise<CommandCheck> {
    const command = await this.toolsService.getCommand('resolveNetbiosFromIP', {
      ip: '127.0.0.1',
    });
    const { code, stderr } = await this.executeCommandService.executeCommand(command, {
      returnCode: true,
    });
    return { command, isValid: code !== 127, error: stderr };
  }

  async checkStatsSpaceUsage(): Promise<CommandCheck> {
    const command = await this.toolsService.getCommand('statsSpaceUsage', {});
    const { code, stderr } = await this.executeCommandService.executeCommand(command, {
      returnCode: true,
    });
    return { command, isValid: !code, error: stderr };
  }

  async checkStatsDiskUsage(): Promise<CommandCheck> {
    const command = await this.toolsService.getCommand('statsDiskUsage', {});
    const { code, stderr } = await this.executeCommandService.executeCommand(command, {
      returnCode: true,
    });
    return { command, isValid: !code, error: stderr };
  }
}
