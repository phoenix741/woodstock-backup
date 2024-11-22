import { Injectable } from '@nestjs/common';
import * as shell from 'shelljs';

export interface ExecuteCommandOption {
  returnCode?: boolean;
}

@Injectable()
export class ExecuteCommandService {
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
}
