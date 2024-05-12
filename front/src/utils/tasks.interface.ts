import { QueueTaskState } from '@/generated/graphql';

export interface ProgressTaskDetailGui {
  progressCurrent?: bigint | null;
  progressMax?: bigint | null;

  fileSize?: bigint | null;
  newFileSize?: bigint | null;

  compressedFileSize?: bigint | null;
  newCompressedFileSize?: bigint | null;

  fileCount?: number | null;
  newFileCount?: number | null;
  errorCount?: number | null;
}

export interface JobTaskDetailGui {
  title?: string;
  description?: string;
  state?: QueueTaskState | null;
  progression?: ProgressTaskDetailGui;
}

export interface ProgressTaskGui {
  newFileCount?: number | null;
  fileCount?: number | null;
  progressCurrent?: bigint | null;
  progressMax?: bigint | null;
  speed?: number | null;
  percent?: number | null;
}
export interface JobTaskGui {
  jobType: string;
  jobId: string;
  hostname?: string;
  ip?: string;
  number?: number;
  startDate?: number;
  state?: QueueTaskState | null;
  progression?: ProgressTaskGui;
  details: JobTaskDetailGui[];
  failedReason?: string | null;
}
