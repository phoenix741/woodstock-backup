import { resolve } from 'path';
import { loadSync } from 'protobufjs';

const root = loadSync(resolve('woodstock.proto'));

export const ProtoFileManifest = root.lookupType('woodstock.FileManifest');
export const ProtoFileManifestJournalEntry = root.lookupType('woodstock.FileManifestJournalEntry');
export const ProtoPoolRefCount = root.lookupType('woodstock.PoolRefCount');
export const ProtoPoolUnused = root.lookupType('woodstock.PoolUnused');
