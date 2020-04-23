import { Injectable } from '@nestjs/common';
import * as shell from 'shelljs';
import { ExecuteCommandOperation } from 'src/hosts/host-config.dto';

import { BackupProgression, Options } from './interfaces/options';

@Injectable()
export class ExecuteCommandService {
  /**
   * Execute a script on the backup host
   *
   * @param command Command to launch
   * @param options Options used for the command
   */
  async execute(operation: ExecuteCommandOperation, options: Options) {
    try {
      options.callbackProgress(new BackupProgression(0));

      options.backupLogger.log(`Execute commande "${operation.command} ...`, options.context);

      const { stdout, stderr } = await this.executeCommand(operation.command);
      stderr && options.backupLogger.error(stderr, options.context);
      stdout && options.backupLogger.log(stdout, options.context);

      options.callbackProgress(new BackupProgression(100));
    } catch (err) {
      options.backupLogger.error(err.message, err.stack, options.context);
      throw err;
    }
  }

  async executeCommand(command: string): Promise<{ stdout: string; stderr: string }> {
    return new Promise((resolve, reject) => {
      shell.exec(command, { silent: true }, (code, stdout, stderr) => {
        if (code === 0) {
          return resolve({ stdout, stderr });
        }

        return reject(new Error(stderr || `Can't execute the command ${command}`));
      });
    });
  }
}
