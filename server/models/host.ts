export interface BackupTaskShare {
  name: string
  includes?: Array<string>
  excludes?: Array<string>
}

export interface BackupTaskConfig {
  name: string
  addresses?: Array<string>
  backup: {
    share: Array<BackupTaskShare>
    includes?: Array<string>
    excludes?: Array<string>
    timeout: number
    preUserCommand?: string
    postUserCommand?: string
  }
}
