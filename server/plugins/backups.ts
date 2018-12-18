export interface BackupProgression {
  percent: number
  newFileSize?: number
  newFileCount?: number
  fileCount?: number
  speed?: number
}

export interface BackupContext extends BackupProgression {
  sharePath: string
}

export type CallbackProgressFn = (progression: BackupProgression) => void

export type CallbackLoggerFn = (options: BackupLogger) => void

export interface BackupLogger {
  level: string
  message: string
  label: string
}

export interface BackupOptions {
  includes: Array<string>
  excludes: Array<string>
  timeout?: number
  callbackProgress: CallbackProgressFn
  callbackLogger: CallbackLoggerFn
}

export interface RSyncBackupOptions extends BackupOptions {
  username: string
}
