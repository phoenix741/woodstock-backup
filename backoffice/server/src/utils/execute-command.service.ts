import { Injectable, Logger } from '@nestjs/common';
import * as shell from 'shelljs';
import { CommandParameters } from '../server/tools.model';
import { ToolsService } from '../server/tools.service';

export interface ExecuteCommandOption {
  returnCode?: boolean;
}

@Injectable()
export class ExecuteCommandService {
  private logger = new Logger(ExecuteCommandService.name);

  constructor(private toolsService: ToolsService) {}

  async executeCommand(
    command: string,
    options: ExecuteCommandOption = {},
  ): Promise<{ code: number; stdout: string; stderr: string }> {
    return new Promise((resolve, reject) => {
      shell.exec(command, { silent: true }, (code, stdout, stderr) => {
        if (code === 0 || options?.returnCode) {
          return resolve({ code, stdout, stderr });
        }

        return reject(new Error(stderr || `Can't execute the command ${command}`));
      });
    });
  }

  async executeTool(
    command: string,
    params: CommandParameters,
    options: ExecuteCommandOption = {},
  ): Promise<{ code: number; stdout: string; stderr: string }> {
    const commandString = await this.toolsService.getCommand(command, params);
    this.logger.debug(`Execute command: ${commandString}`);
    return await this.executeCommand(commandString, options);
  }
}
