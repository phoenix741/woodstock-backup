import { readFile } from 'fs/promises';
import { count, from } from 'ix/asynciterable';
import { map } from 'ix/asynciterable/operators';
import { join } from 'path';
import { ProtoFileManifestJournalEntry } from '../shared/woodstock.model.js';
import { FileManifestJournalEntry } from '../shared';
import { ProtobufService } from './protobuf.service.js';

describe('ManifestWrapper', () => {
  let service: ProtobufService;

  beforeEach(() => {
    service = new ProtobufService();
  });

  test('#readAllMessages should read all message', async () => {
    const it = service.loadFile(
      join(__dirname, '..', 'manifest', 'fixtures', 'test.journal'),
      ProtoFileManifestJournalEntry,
    );
    expect(await count(it)).toBe(1000);
  });

  test('#readAllMessages should error', async () => {
    try {
      const it = service.loadFile(
        join(__dirname, '..', 'manifest', 'fixtures', 'noexist.journal'),
        ProtoFileManifestJournalEntry,
      );
      for await (const value of it) {
        expect(value).toBeTruthy();
      }
    } catch (err) {
      expect(err.message).toMatch(/ENOENT: no such file or directory, open/);
    }
  });

  test('#writeAllMessages should capable to write message, readed by readAllMessages', async () => {
    const inFile = join(__dirname, '..', 'manifest', 'fixtures', 'test.journal');
    const outFile = join(__dirname, '..', 'manifest', 'fixtures', 'test.journal.out');

    const it = service.loadFile<FileManifestJournalEntry>(inFile, ProtoFileManifestJournalEntry);
    const manifests = from(it).pipe(map((obj) => obj.message));

    try {
      await service.writeFile(outFile, ProtoFileManifestJournalEntry, manifests);

      expect(await readFile(inFile)).toEqual(await readFile(outFile));
    } finally {
      await service.rmFile(outFile);
    }
  });
});
