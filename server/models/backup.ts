export enum BackupState {
  WAITING,
  RUNNING,
  SUCCESS,
  ABORTED,
  FAILED
}

export interface TaskProgression {
  fileSize?: number
  newFileSize?: number

  newFileCount?: number
  fileCount?: number

  speed?: number
  percent: number
}

export const DEFAULT_PROGRESSION: TaskProgression = {
  newFileCount: 0,
  fileCount: 0,
  newFileSize: 0,
  fileSize: 0,
  speed: 0,
  percent: 0
}

export interface BackupSubTask {
  name: string
  description: string
  state: BackupState
  failable: boolean
  command: (host: BackupTask, task: BackupSubTask) => Promise<void>
  progression?: TaskProgression
}

export interface BackupTask {
  id: string
  host: string
  startDate: Date
  isBackupFull: boolean
  progression?: TaskProgression
  state: BackupState
  subtasks: Array<BackupSubTask>
}
