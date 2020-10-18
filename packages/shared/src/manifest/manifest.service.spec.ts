import { copyFile, unlink } from 'fs/promises';
import { count, reduce } from 'ix/asynciterable';
import Long from 'long';
import { join } from 'path';
import { ProtoFileManifest, ProtoFileManifestJournalEntry } from '.';
import { FileManifest, FileManifestJournalEntry, longToBigInt } from '..';
import { readAllMessages } from './manifest-wrapper.utils';
import { Manifest } from './manifest.model';
import { ManifestService } from './manifest.service';

async function deleteTest(filename: string) {
  await unlink(join(__dirname, 'fixtures', `${filename}.journal`)).catch(() => undefined);
  await unlink(join(__dirname, 'fixtures', `${filename}.manifest`)).catch(() => undefined);
}

describe('ManifestService', () => {
  let service: ManifestService;

  beforeEach(async () => {
    service = new ManifestService();
  });

  test('#compact, wait promise', async () => {
    await copyFile(join(__dirname, 'fixtures', 'test.journal'), join(__dirname, 'fixtures', 'test-compact.journal'));

    try {
      const manifest = new Manifest('test-compact', join(__dirname, 'fixtures'));

      await service.compact(manifest);

      const manifestIt = readAllMessages(join(__dirname, 'fixtures', 'test-compact.manifest'), ProtoFileManifest);
      expect(await count(manifestIt)).toBe(1000);
    } finally {
      await deleteTest('test-compact');
    }
  });
});
