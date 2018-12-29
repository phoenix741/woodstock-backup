import * as path from 'path'
import * as util from 'util'
import * as mkdirp from 'mkdirp'
import { CallbackProgressFn, CallbackLoggerFn, BackupLogger, BackupProgression } from '../plugins/backups'
import { BackupTaskConfig } from '../models/host'
import { HostBackup, Backup } from './host-backup'
import { pick } from '../utils/lodash'
import * as logform from 'logform'
import * as winston from 'winston'

import { resolveFromConfig } from '../plugins/resolve'
import { executeScript } from '../plugins/script'
import { backup as backupRsync } from '../plugins/rsync'

const mkdirpPromise = util.promisify(mkdirp)

const { combine, timestamp, printf } = winston.format

const loggerFormat = printf((info: logform.TransformableInfo) => {
  return `${info.timestamp} [${info.label}] ${info.level}: ${info.message}`
})

export type CallbackTaskChangeFn = (task: BackupTask) => void

export enum BackupState {
  WAITING,
  RUNNING,
  SUCCESS,
  ABORTED,
  FAILED
}

export interface ITaskProgression {
  fileSize?: number
  newFileSize?: number

  newFileCount?: number
  fileCount?: number

  speed?: number
  percent: number
}

export const DEFAULT_PROGRESSION: ITaskProgression = {
  newFileCount: 0,
  fileCount: 0,
  newFileSize: 0,
  fileSize: 0,
  speed: 0,
  percent: 0
}

export interface IBackupSubTask {
  name: string
  description: string
  state: BackupState
  failable: boolean
  progress: boolean
  command: (host: BackupTask, task: IBackupSubTask, progressFn: CallbackProgressFn, loggerFn: CallbackLoggerFn) => Promise<any>
  progression?: ITaskProgression
}

export interface IBackupTask {
  number: number
  host: string
  startDate: Date
  progression: ITaskProgression
  state: BackupState
  subtasks: Array<IBackupSubTask>
}

export class BackupTask {
  private _hostBackup: HostBackup
  private isBackupFull = true
  private ip?: string
  private destinationDirectory: string
  private _number: number

  public host: string
  public startDate: Date
  public progression = Object.assign({}, DEFAULT_PROGRESSION)
  public state = BackupState.WAITING
  public subtasks: Array<IBackupSubTask> = []

  private constructor (hostname: string) {
    this.host = hostname
    this._hostBackup = new HostBackup(hostname)
    this.startDate = new Date()
  }

  static async createFromHostConfig (config: BackupTaskConfig): Promise<BackupTask> {
    const task = new BackupTask(config.name)

    const lastBackup = await task._hostBackup.lastBackup
    const lastBackupNumber = lastBackup && lastBackup.number || 0

    task.number = lastBackup && lastBackup.isBackupFull ? lastBackup.number + 1 : lastBackup && lastBackup.number || 0

    // Step 0: Clone previous backup
    if (lastBackupNumber !== task.number) {
      const lastDestinationDirectory = task._hostBackup.getDestinationDirectory(lastBackupNumber)

      task.subtasks.push({
        name: 'clone',
        description: 'Clone the backup from previous',
        state: BackupState.WAITING,
        failable: true,
        progress: false,
        command: async (host, task, progressFn, loggerFn) => backupRsync(null, lastDestinationDirectory, host.destinationDirectory, { rsync: true, includes: [], excludes: [], callbackLogger: loggerFn, callbackProgress: progressFn })
      })
    }

    // Step 1: Resolve addresses
    task.subtasks.push({
      name: 'resolve',
      description: 'Resolve the addresses (and the host)',
      state: BackupState.WAITING,
      failable: true,
      progress: false,
      command: async host => (host.ip = await resolveFromConfig(config))
    })

    // Step 2: Launch pre user command
    if (config.backup.preUserCommand) {
      task.subtasks.push({
        name: 'pre',
        description: 'Pre-user command',
        state: BackupState.WAITING,
        failable: false,
        progress: false,
        command: (host, task, progressFn, loggerFn) => executeScript(config.backup.preUserCommand || '', { callbackLogger: loggerFn, label: 'pre' })
      })
    }

    // Step 3: Launch backup of each share
    for (const share of config.backup.share) {
      task.subtasks.push({
        name: `backup ${share.name}`,
        description: `Backup of ${share.name}`,
        state: BackupState.WAITING,
        failable: true,
        progress: true,
        command: async (host, task, progressFn, loggerFn) => {
          const includes = [...(share.includes || []), ...(config.backup.includes || [])]
          const excludes = [...(share.excludes || []), ...(config.backup.excludes || [])]

          if (! host.ip) {
            throw new Error(`Can't backup host ${host.host}, can't find the IP.`)
          }

          host.isBackupFull = host.isBackupFull && await backupRsync(host.ip, share.name, host.destinationDirectory + '/' + share.name, { rsync: true, username: 'root', includes, excludes, callbackProgress: progressFn, callbackLogger: loggerFn })
        }
      })
    }

    // Step 4: Launch post user command
    if (config.backup.postUserCommand) {
      task.subtasks.push({
        name: 'post',
        description: 'Post-user command',
        state: BackupState.WAITING,
        failable: false,
        progress: false,
        command: (host, task, progressFn, loggerFn) => executeScript(config.backup.postUserCommand || '', { callbackLogger: loggerFn, label: 'pre' })
      })
    }

    // Step 5: Register backup
    task.subtasks.push({
      name: 'register',
      description: 'Register the backup',
      state: BackupState.WAITING,
      failable: true,
      progress: false,
      command: host => host._hostBackup.addBackup(host.toBackup())
    })

    return task
  }

  async launchBackup (taskChanged: CallbackTaskChangeFn) {
    let failed = false

    this.state = BackupState.RUNNING
    taskChanged(this)

    await mkdirpPromise(this.destinationDirectory)

    const logger = winston.createLogger({
      level: 'info',
      format: combine(
        timestamp(),
        loggerFormat
      ),
      transports: [
        new winston.transports.File({ filename: path.join(this.destinationDirectory, 'backup.errors.log'), level: 'error', tailable: true }),
        new winston.transports.File({ filename: path.join(this.destinationDirectory, 'backup.log'), tailable: true })
      ]
    })

    for (let subtask of this.subtasks) {
      if (failed && subtask.failable) {
        subtask.state = BackupState.ABORTED
        taskChanged(this)
        continue
      }

      subtask.state = BackupState.RUNNING
      taskChanged(this)

      try {
        subtask.progression = subtask.progression || { percent: 0 }
        await subtask.command(this, subtask, progression => {
          this.progressTask(subtask, progression)
          taskChanged(this)
        }, (obj: BackupLogger) => logger.log(obj))
        subtask.progression.percent = 100
        subtask.state = BackupState.SUCCESS

        this.progressTask(subtask)
        taskChanged(this)
      } catch (err) {
        logger.log({ level: 'error', message: err.message, label: subtask.name })
        subtask.state = BackupState.FAILED
        taskChanged(this)
        failed = true
      }
    }

    this.state = failed ? BackupState.FAILED : BackupState.SUCCESS
    taskChanged(this)
    if (failed) {
      throw new Error(`Backup of ${this.host} have been failed`)
    }
  }

  set number (n: number) {
    this._number = n
    this.destinationDirectory = this._hostBackup.getDestinationDirectory(this._number)
  }

  get number (): number {
    return this._number
  }

  toJSON (): IBackupTask {
    return pick(this, 'number', 'host', 'startDate', 'progression', 'state', 'subtasks') as IBackupTask
  }

  toBackup (): Backup {
    const endDate = new Date()
    return {
      number: this.number,
      isBackupFull: this.isBackupFull,

      startDate: this.startDate,
      endDate,

      fileCount: this.progression.fileCount || 0,
      newFileCount: this.progression.newFileCount || 0,
      existingFileCount: (this.progression.fileCount || 0) - (this.progression.newFileCount || 0),

      fileSize: this.progression.fileSize || 0,
      existingFileSize: (this.progression.fileSize || 0) - (this.progression.newFileSize || 0),
      newFileSize: this.progression.newFileSize || 0,

      speed: (this.progression.newFileSize || 0) / (endDate.getTime() - this.startDate.getTime())
    }
  }

  private progressTask (task: IBackupSubTask, progression?: BackupProgression) {
    if (progression) {
      task.progression = progression
    }

    this.progression = this.subtasks.reduce((acc, subtask) => {
      if (subtask.progress) {
        const progression = Object.assign({}, DEFAULT_PROGRESSION, subtask.progression)
        return {
          newFileCount: (acc.newFileCount || 0) + (progression.newFileCount || 0),
          fileCount: (acc.fileCount || 0) + (progression.fileCount || 0),
          newFileSize: (acc.newFileSize || 0) + (progression.newFileSize || 0),
          fileSize: (acc.fileSize || 0) + (progression.fileSize || 0),
          speed: progression.speed,
          percent: acc.percent + progression.percent
        }
      }
      return acc
    }, DEFAULT_PROGRESSION)
    this.progression.percent = this.progression.percent / this.subtasks.filter(subtask => subtask.progress).length
  }
}
