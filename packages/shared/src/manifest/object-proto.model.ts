import { join } from 'path';
import { loadSync } from 'protobufjs';

const root = loadSync(join(__dirname, '..', '..', 'woodstock.proto'));

export const ProtoFileManifest = root.lookupType('woodstock.FileManifest');
export const ProtoFileManifestJournalEntry = root.lookupType('woodstock.FileManifestJournalEntry');
