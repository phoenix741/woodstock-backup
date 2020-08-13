import { Injectable, Logger } from '@nestjs/common';
import { Observable, Subject } from 'rxjs';
import * as shell from 'shelljs';
import { ExecuteCommandOperation } from 'src/hosts/host-configuration.dto';

import { BackupProgression, Options } from './interfaces/options';
import { ToolsService } from '../server/tools.service';
import { CommandParameters } from '../server/tools.model';

export interface ExecuteCommandOption {
  returnCode?: boolean;
}

@Injectable()
export class ExecuteCommandService {
  private logger = new Logger(ExecuteCommandService.name);

  constructor(private toolsService: ToolsService) {}

  /**
   * Execute a script on the backup host
   *
   * @param command Command to launch
   * @param options Options used for the command
   */
  execute(operation: ExecuteCommandOperation, options: Options): Observable<BackupProgression> {
    options.backupLogger.log(`Execute commande "${operation.command} ...`, options.context);

    const progression = new Subject<BackupProgression>();
    progression.next(new BackupProgression(0));

    this.executeCommand(operation.command)
      .then(({ stdout, stderr }) => {
        stderr && options.backupLogger.error(stderr, options.context);
        stdout && options.backupLogger.log(stdout, options.context);
        progression.next(new BackupProgression(100));
        progression.complete();
      })
      .catch(err => {
        options.backupLogger.error(err.message, err.stack, options.context);
        progression.error(err);
      });

    return progression.asObservable();
  }

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

  async executeTool(command: string, params: CommandParameters, options: ExecuteCommandOption = {}) {
    const commandString = await this.toolsService.getCommand(command, params);
    this.logger.debug(`Execute command: ${commandString}`);
    return await this.executeCommand(commandString, options);
  }
}
