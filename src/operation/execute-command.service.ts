import { Logger, Injectable } from '@nestjs/common';
import * as shell from 'shelljs';
import { ExecuteCommandOperation } from 'src/hosts/host-config.dto';

import { Options, BackupProgression } from './interfaces/options';

@Injectable()
export class ExecuteCommandService {
  private logger = new Logger(ExecuteCommandService.name);

  /**
   * Execute a script on the backup host
   *
   * @param command Command to launch
   * @param options Options used for the command
   */
  async execute(operation: ExecuteCommandOperation, options: Options) {
    try {
      options.callbackProgress(new BackupProgression(0));

      this.logger.log({ ...operation, ...options });

      const { stdout, stderr } = await this.executeCommand(operation.command);
      stderr && this.logger.error({ message: stderr, ...operation, ...options });
      stdout && this.logger.error({ message: stdout, ...operation, ...options });

      options.callbackProgress(new BackupProgression(100));
    } catch (error) {
      this.logger.error({
        message: error.message,
        ...operation,
        ...options,
      });
      throw error;
    }
  }

  async executeCommand(command: string): Promise<{ stdout: string; stderr: string }> {
    return new Promise((resolve, reject) => {
      shell.exec(command, { silent: true }, (code, stdout, stderr) => {
        if (code === 0) {
          return resolve({ stdout, stderr });
        }

        return reject({
          stdout,
          stderr: stderr || `Can't execute the command ${command}`,
        });
      });
    });
  }
}
