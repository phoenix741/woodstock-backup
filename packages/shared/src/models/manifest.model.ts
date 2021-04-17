import * as Long from 'long';

export interface FileManifestStat {
  ownerId?: Long;
  groupId?: Long;
  size?: Long;
  lastRead?: Long;
  lastModified?: Long;
  created?: Long;
  mode?: Long;
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

export enum EntryType {
  ADD = 0,
  MODIFY = 1,
  REMOVE = 2,
}

export interface FileManifestJournalEntryRemove {
  type: EntryType.REMOVE;
  path: Buffer;
}

export interface FileManifestJournalEntryAddOrModify {
  type: EntryType.ADD | EntryType.MODIFY;
  manifest: FileManifest;
}

export type FileManifestJournalEntry = FileManifestJournalEntryRemove | FileManifestJournalEntryAddOrModify;
