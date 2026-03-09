import type { FragmentFileDescriptionFragment } from '@/generated/graphql';

export interface BackupSizeByDate {
  startDate: Date;
  fileSize: bigint;
  newFileSize: bigint;
}

export interface TreeViewNode {
  id: string;
  sharePath: string;
  path: string[];
  displayName: string;
  children?: TreeViewNode[];
  node: FragmentFileDescriptionFragment;
  props: Record<string, unknown>;
}
