import * as path from 'path'
import * as fs from 'fs'
import * as uuid from 'uuid'
import * as yaml from 'js-yaml'
import * as mkdirp from 'mkdirp'
import * as util from 'util'
import * as logform from 'logform'
import * as winston from 'winston'
import { BackupTaskConfig } from '../models/host'
import { BackupTask, BackupState, DEFAULT_PROGRESSION, BackupSubTask } from '../models/task'
import { executeScript } from '../plugins/script'
import { BackupLogger, BackupProgression } from '../plugins/backups'
import { backup as backupRsync } from '../plugins/rsync'
import { NG_MASS_BACKUP_HOST_PATH } from '../config/index'

const mkdirpPromise = util.promisify(mkdirp)

const { combine, timestamp, printf } = winston.format

const loggerFormat = printf((info: logform.TransformableInfo) => {
  return `${info.timestamp} [${info.label}] ${info.level}: ${info.message}`
})

interface Backup {
  number: number
  isBackupFull: boolean

  startDate: Date
  endDate: Date

  fileCount: number
  newFileCount: number
  existingFileCount: number

  fileSize: number
  existingFileSize: number
  newFileSize: number

  speed: number
}

export type CallbackTaskChangeFn = (task: BackupTask) => void

export function getBackupDirectory (host: string): string {
  return path.join(NG_MASS_BACKUP_HOST_PATH, host)
}

export function getBackupDestinationDirectory (host: string, backupNumber: number): string {
  return path.join(NG_MASS_BACKUP_HOST_PATH, host, '' + backupNumber)
}

export function getBackupFile (host: string): string {
  return path.join(getBackupDirectory(host), 'backup.yml')
}

export async function addBackup (host: string, backup: Backup) {
  const backupFile = getBackupFile(host)

  const backups = await getBackups(host)
  if (backups.length && backups[backups.length - 1].number === backup.number) {
    backups[backups.length - 1] = backup
  } else {
    backups.push(backup)
  }

  const backupsFromStr = yaml.safeDump(backups)
  await fs.promises.writeFile(backupFile, backupsFromStr, 'utf-8')
}

export async function getBackups (host: string): Promise<Array<Backup>> {
  const backupFile = getBackupFile(host)

  try {
    const backupsFromFile = await fs.promises.readFile(backupFile, 'utf8')
    return yaml.safeLoad(backupsFromFile) || []
  } catch (err) {
    return []
  }
}

export async function createTask (config: BackupTaskConfig): Promise<BackupTask> {
  // Define the location of the backup
  const backups = await getBackups(config.name)
  const lastBackup = backups.length ? backups[backups.length - 1] : { number: 0, isBackupFull: false }
  const lastBackupNumber = lastBackup.isBackupFull ? lastBackup.number : lastBackup.number
  const nextBackupNumber = lastBackup.isBackupFull ? lastBackup.number + 1 : lastBackup.number
  const lastDestinationDirectory = getBackupDestinationDirectory(config.name, lastBackupNumber)
  const nextDestinationDirectory = getBackupDestinationDirectory(config.name, nextBackupNumber)

  await mkdirpPromise(nextDestinationDirectory)

  const task: BackupTask = {
    id: uuid.v1(),
    host: config.name,
    startDate: new Date(),

    progression: {
      newFileCount: 0,
      fileCount: 0,

      newFileSize: 0,
      fileSize: 0,

      percent: 0,
      speed: 0
    },

    isBackupFull: true,

    state: BackupState.WAITING,
    subtasks: [],
    logger: winston.createLogger({
      level: 'info',
      format: combine(
        timestamp(),
        loggerFormat
      ),
      transports: [
        new winston.transports.File({ filename: path.join(nextDestinationDirectory, `backup.errors.log`), level: 'error', tailable: true }),
        new winston.transports.File({ filename: path.join(nextDestinationDirectory, `backup.log`), tailable: true })
      ]
    })
  }

  // Step 0: Clone previous backup
  if (lastBackupNumber !== nextBackupNumber) {
    task.subtasks.push({
      name: 'clone',
      description: 'Clone the backup from previous',
      state: BackupState.WAITING,
      failable: true,
      progress: false,
      command: async (host, task, progressFn) => backupRsync(null, lastDestinationDirectory, nextDestinationDirectory, { rsync: true, includes: [], excludes: [], callbackLogger: obj => host.logger.log(obj), callbackProgress: progressFn })
    })
  }

  // Step 1: Resolve addresses
  task.subtasks.push({
    name: 'resolve',
    description: 'Resolve the addresses (and the host)',
    state: BackupState.WAITING,
    failable: true,
    progress: false,
    command: async task => null
  })

  // Step 2: Launch pre user command
  if (config.backup.preUserCommand) {
    task.subtasks.push({
      name: 'pre',
      description: 'Pre-user command',
      state: BackupState.WAITING,
      failable: false,
      progress: false,
      command: host => executeScript(config.backup.preUserCommand || '', { callbackLogger: (obj: BackupLogger) => host.logger.log(obj), label: 'pre' })
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
      command: async (host, task, progressFn) => {
        const includes = [...(share.includes || []), ...(config.backup.includes || [])]
        const excludes = [...(share.excludes || []), ...(config.backup.excludes || [])]

        host.isBackupFull = host.isBackupFull && await backupRsync(host.host, share.name, nextDestinationDirectory + '/' + share.name, { rsync: true, username: 'root', includes, excludes, callbackProgress: progressFn, callbackLogger: (obj: BackupLogger) => host.logger.log(obj) })
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
      command: host => executeScript(config.backup.postUserCommand || '', { callbackLogger: (obj: BackupLogger) => host.logger.log(obj), label: 'pre' })
    })
  }

  // Step 5: Register backup
  task.subtasks.push({
    name: 'register',
    description: 'Register the backup',
    state: BackupState.WAITING,
    failable: false,
    progress: false,
    command: host => register(host, nextBackupNumber)
  })

  return task
}

export async function launchBackup (task: BackupTask, taskChanged: CallbackTaskChangeFn) {
  let failed = false

  task.state = BackupState.RUNNING
  taskChanged(task)

  for (let subtask of task.subtasks) {
    if (failed && subtask.failable) {
      subtask.state = BackupState.ABORTED
      taskChanged(task)
      continue
    }

    subtask.state = BackupState.RUNNING
    taskChanged(task)

    try {
      subtask.progression = subtask.progression || { percent: 0 }
      await subtask.command(task, subtask, progression => {
        progressTask(task, subtask, progression)
        taskChanged(task)
      })
      subtask.progression.percent = 100
      subtask.state = BackupState.SUCCESS

      progressTask(task, subtask)
      taskChanged(task)
    } catch (err) {
      task.logger.log({ level: 'error', message: err.message, label: subtask.name })
      subtask.state = BackupState.FAILED
      taskChanged(task)
      failed = true
    }
  }

  task.state = failed ? BackupState.FAILED : BackupState.SUCCESS
  taskChanged(task)
}

async function register (host: BackupTask, backupNumber: number) {
  const endDate = new Date()
  const backup: Backup = {
    number: backupNumber,
    isBackupFull: host.isBackupFull,

    startDate: host.startDate,
    endDate,

    fileCount: host.progression.fileCount || 0,
    newFileCount: host.progression.newFileCount || 0,
    existingFileCount: (host.progression.fileCount || 0) - (host.progression.newFileCount || 0),

    fileSize: host.progression.fileSize || 0,
    existingFileSize: (host.progression.fileSize || 0) - (host.progression.newFileSize || 0),
    newFileSize: host.progression.newFileSize || 0,

    speed: (host.progression.newFileSize || 0) / (endDate.getTime() - host.startDate.getTime())
  }

  await addBackup(host.host, backup)
}

function progressTask (host: BackupTask, task: BackupSubTask, progression?: BackupProgression) {
  if (progression) {
    task.progression = progression
  }

  host.progression = host.subtasks.reduce((acc, subtask) => {
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
  host.progression.percent = host.progression.percent / host.subtasks.filter(subtask => subtask.progress).length
}
