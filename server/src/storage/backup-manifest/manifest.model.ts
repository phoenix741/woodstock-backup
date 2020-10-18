export enum EntryType {
  ADD = 0,
  MODIFY = 1,
  REMOVE = 2,
  CLOSE = 255,
}
export interface FileManifestStat {
  ownerId?: number;
  groupId?: number;
  size?: number;
  lastRead?: number;
  lastModified?: number;
  created?: number;
  mode?: number;
}

export interface FileManifestAcl {
  user?: string;
  group?: string;
  mask?: number;
  other?: number;
}

export interface FileManifest {
  path: Buffer;
  stats?: FileManifestStat;
  xattr?: Record<string, Buffer>;
  acl?: FileManifestAcl[];
  chunks?: Buffer[];
  sha256?: Buffer;
}

export interface FileManifestJournalEntryRemove {
  type: EntryType.REMOVE;
  path: Buffer;
}

export interface FileManifestJournalEntryAddOrModify {
  type: EntryType.ADD | EntryType.MODIFY;
  manifest: FileManifest;
}

export interface FileManifestJournalEntryClose {
  type: EntryType.CLOSE;
}

export type FileManifestJournalEntry =
  | FileManifestJournalEntryRemove
  | FileManifestJournalEntryAddOrModify
  | FileManifestJournalEntryClose;
