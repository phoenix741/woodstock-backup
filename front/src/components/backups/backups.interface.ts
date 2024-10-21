export interface BackupSizeByDate {
  startDate: number;
  fileSize: bigint;
  newFileSize: bigint;
}

export interface TreeViewNode {
  sharePath: string;
  path: string[];
  displayName: string;
  children?: TreeViewNode[];
  isHidden: boolean;
}
