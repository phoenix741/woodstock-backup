import * as shell from 'shelljs'
import { CallbackLoggerFn } from './backups';

export interface ScriptOptions {
  callbackLogger: CallbackLoggerFn
  label: string
}

export async function executeScript (command: string, options: ScriptOptions) {
  try {
    options.callbackLogger({level: 'info', message: command, label: options.label})
    log(options, await executeCommand(command))
  } catch (error) {
    log(options, error)
    throw error
  }
}

async function executeCommand (command: string): Promise<{stdout: string,stderr: string}> {
  return new Promise((resolve, reject) => {
    shell.exec(command, (code, stdout, stderr) => {
      if (code === 0) {
        return resolve({ stdout, stderr })
      }

      return reject({ stdout, stderr: stderr || `Can't execute the command ${command}`})
    })
  })
}

function log (options: ScriptOptions, { stdout, stderr }: { stdout: string, stderr: string}) {
  stderr && options.callbackLogger({ level: 'error', message: stderr, label: options.label })
  stdout && options.callbackLogger({ level: 'info',  message: stdout, label: options.label })
}
