import { readFile, rm } from 'fs/promises';
import { count, pipe } from 'ix/asynciterable';
import { map } from 'ix/asynciterable/operators';
import { join } from 'path';
import { FileManifestJournalEntry } from 'src';
import { readAllMessages, writeAllMessages } from './manifest-wrapper.utils';
import { ProtoFileManifestJournalEntry } from './object-proto.model';

describe('ManifestWrapper', () => {
  test('#readAllMessages should read all message', async () => {
    const it = readAllMessages(join(__dirname, 'fixtures', 'test.journal'), ProtoFileManifestJournalEntry);
    expect(await count(it)).toBe(1000);
  });

  test('#readAllMessages should error', async () => {
    try {
      const it = readAllMessages(join(__dirname, 'fixtures', 'noexist.journal'), ProtoFileManifestJournalEntry);
      for await (const value of it) {
        expect(value).toBeTruthy();
      }
    } catch (err) {
      expect(err).toMatchSnapshot('readAllMessages.error');
    }
  });

  test('#writeAllMessages should capable to write message, readed by readAllMessages', async () => {
    const inFile = join(__dirname, 'fixtures', 'test.journal');
    const outFile = join(__dirname, 'fixtures', 'test.journal.out');

    const it = readAllMessages<FileManifestJournalEntry>(inFile, ProtoFileManifestJournalEntry);
    const manifests = pipe(
      it,
      map((obj) => obj.message),
    );

    try {
      await writeAllMessages(outFile, ProtoFileManifestJournalEntry, manifests);

      expect(await readFile(inFile)).toEqual(await readFile(outFile));
    } finally {
      await rm(outFile);
    }
  });
});
