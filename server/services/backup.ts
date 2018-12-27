import * as path from 'path'
import * as fs from 'fs'
import * as uuid from 'uuid'
import * as yaml from 'js-yaml'
import * as mkdirp from 'mkdirp'
import * as util from 'util'
import { BackupTaskConfig } from '../models/host'
import { BackupTask, BackupState, DEFAULT_PROGRESSION, BackupSubTask } from '../models/backup'
import { executeScript } from '../plugins/script';
import { BackupLogger, BackupProgression } from '../plugins/backups'
import { backup as backupRsync } from '../plugins/rsync'
import { NG_MASS_BACKUP_HOST_PATH } from '../config/index'

const mkdirpPromise = util.promisify(mkdirp)

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
  //logger.info(`Register the backup in the file ${backupFile}`)

  const backups = await getBackups(host)
  backups.push(backup)

  const backupsFromStr = yaml.safeDump(backups)
  await fs.promises.writeFile(backupFile, backupsFromStr, 'utf-8')
}

export async function getBackups (host: string): Promise<Array<Backup>> {
  const backupFile = getBackupFile(host)
  //logger.info(`Load the backup from the file ${backupFile}`)

  try {
    const backupsFromFile = await fs.promises.readFile(backupFile, 'utf8')
    return yaml.safeLoad(backupsFromFile) || []
  } catch (err) {
    //logger.info(`File ${backupFile} doesn't exist`)
    return []
  }
}

async function register (host: BackupTask, backupNumber: number) {
  const endDate = new Date()
  const backup: Backup = {
    number: backupNumber,
    isBackupFull: host.isBackupFull,

    startDate: host.startDate,
    endDate,

    fileCount: host.progression.fileCount,
    newFileCount: host.progression.newFileCount,
    existingFileCount: host.progression.fileCount - host.progression.newFileCount,

    fileSize: host.progression.fileSize,
    existingFileSize: host.progression.fileSize - host.progression.newFileSize,
    newFileSize: host.progression.newFileSize,

    speed: host.progression.newFileSize / (endDate.getTime() - host.startDate.getTime())
  }

  await addBackup(host.host, backup)
}

export async function createTask (config: BackupTaskConfig): Promise<BackupTask> {
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
    subtasks: []
  }

  // Define the location of the backup
  const backups = await getBackups(config.name)
  const lastBackup = backups.length ? backups[backups.length - 1] : { number: 0, isBackupFull: false }
  const lastBackupNumber = lastBackup.isBackupFull ? lastBackup.number : lastBackup.number
  const nextBackupNumber = lastBackup.isBackupFull ? lastBackup.number + 1 : lastBackup.number
  const lastDestinationDirectory = getBackupDestinationDirectory(config.name, lastBackupNumber)
  const nextDestinationDirectory = getBackupDestinationDirectory(config.name, nextBackupNumber)

  // Step 0: Clone previous backup
  if (lastBackupNumber !== nextBackupNumber) {
    task.subtasks.push({
      name: 'clone',
      description: 'Clone the backup from previous',
      state: BackupState.WAITING,
      failable: true,
      command: async (host, task) => {
        function callbackProgress (progression: BackupProgression) {
          task.progression = progression

          progressTask(host, task)
        }

        await backupRsync(null, lastDestinationDirectory, nextDestinationDirectory, { rsync: true, includes: [], excludes: [], callbackLogger, callbackProgress })
      }
    })
  } else {
    await mkdirpPromise(nextDestinationDirectory)
  }

  // Step 1: Resolve addresses
  task.subtasks.push({
    name: 'resolve',
    description: 'Resolve the addresses (and the host)',
    state: BackupState.WAITING,
    failable: true,
    command: task => null
  })

  // Step 2: Launch pre user command
  if (config.backup.preUserCommand) {
    task.subtasks.push({
      name: 'pre',
      description: 'Pre-user command',
      state: BackupState.WAITING,
      failable: false,
      command: () => executeScript(config.backup.preUserCommand, { callbackLogger, label: 'pre' })
    })
  }

  // Step 3: Launch backup of each share
  for (const share of config.backup.share) {
    task.subtasks.push({
      name: `backup ${share.name}`,
      description: `Backup of ${share.name}`,
      state: BackupState.WAITING,
      failable: true,
      command: async (host, task) => {
        const includes = [...(share.includes || []), ...(config.backup.includes || [])]
        const excludes = [...(share.excludes || []), ...(config.backup.excludes || [])]

        function callbackProgress (progression: BackupProgression) {
          task.progression = progression

          progressTask(host, task)
        }

        host.isBackupFull = host.isBackupFull && await backupRsync(host.host, share.name, nextDestinationDirectory + '/' + share.name, { rsync: true, username: 'root', includes, excludes, callbackProgress, callbackLogger })
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
      command: () => executeScript(config.backup.postUserCommand, { callbackLogger, label: 'pre' })
    })
  }

  // Step 5: Register backup
  task.subtasks.push({
    name: 'register',
    description: 'Register the backup',
    state: BackupState.WAITING,
    failable: false,
    command: host => register(host, nextBackupNumber)
  })

  return task
}

export async function launchBackup (task: BackupTask) {
  let failed = false

  task.state = BackupState.RUNNING
  for (let subtask of task.subtasks) {
    if (failed && subtask.failable) {
      subtask.state = BackupState.ABORTED
      continue;
    }

    subtask.state = BackupState.RUNNING
    try {
      subtask.progression = subtask.progression || { percent: 0 }
      await subtask.command(task, subtask)
      subtask.progression.percent = 100
      subtask.state = BackupState.SUCCESS

      progressTask(task, subtask)
    } catch (err) {
      callbackLogger({ level: 'error', message: err.message, label: subtask.name })
      subtask.state = BackupState.FAILED
      failed = true
    }
  }

  task.state = failed ? BackupState.FAILED : BackupState.SUCCESS
}

function progressTask (host: BackupTask, task: BackupSubTask) {
  host.progression = host.subtasks.reduce((acc, subtask) => {
    const progression = Object.assign({}, DEFAULT_PROGRESSION, subtask.progression)
    return {
      newFileCount: acc.newFileCount + progression.newFileCount,
      fileCount: acc.fileCount + progression.fileCount,
      newFileSize: acc.newFileSize + progression.newFileSize,
      fileSize: acc.fileSize + progression.fileSize,
      speed: progression.speed,
      percent: acc.percent + progression.percent
    }
  }, DEFAULT_PROGRESSION)
  host.progression.percent = host.progression.percent / host.subtasks.length

  callbackLogger({ level: 'info', message: 'progression: ' + host.progression.percent, label: task.name })
}

function callbackLogger (options: BackupLogger) {
  console.log(options)
}
