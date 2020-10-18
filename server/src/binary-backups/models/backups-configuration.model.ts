export interface Share {
  name: string;
  includes?: string[];
  excludes?: string[];
  pathPrefix?: string;
}

export interface Task {
  command?: string;
  shares?: Share[];
  includes?: string[];
  excludes?: string[];
}

export interface Operations {
  tasks?: Task[];
  finalizedTasks?: Task[];
}

export interface BackupConfiguration {
  operations: Operations;
}
