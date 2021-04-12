import { join } from 'path';
import { loadSync } from 'protobufjs';
import * as Long from 'long';

const root = loadSync(join(__dirname, 'woodstock.proto'));

export const ProtoFileManifest = root.lookupType('woodstock.FileManifest');
export const ProtoFileManifestJournalEntry = root.lookupType('woodstock.FileManifestJournalEntry');

export enum EntryType {
  ADD = 0,
  MODIFY = 1,
  REMOVE = 2,
  CLOSE = 255,
}

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
