import { copyFile, unlink } from 'fs/promises';
import { count } from 'ix/asynciterable';
import { join } from 'path';
import { ProtoFileManifest } from '../models/object-proto.model';
import { ProtobufService } from '../services/protobuf.service';
import { Manifest } from './manifest.model';
import { ManifestService } from './manifest.service';

async function deleteTest(filename: string) {
  const journal = join(__dirname, 'fixtures', `${filename}.journal`);
  const manifest = join(__dirname, 'fixtures', `${filename}.manifest`);

  await unlink(journal).catch(() => undefined);
  await unlink(manifest).catch(() => undefined);
}

describe('ManifestService', () => {
  let protobufService: ProtobufService;
  let service: ManifestService;

  beforeEach(async () => {
    protobufService = new ProtobufService();
    service = new ManifestService(protobufService);
  });

  test('#compact, wait promise', async () => {
    await copyFile(join(__dirname, 'fixtures', 'test.journal'), join(__dirname, 'fixtures', 'test-compact.journal'));

    try {
      const manifest = new Manifest('test-compact', join(__dirname, 'fixtures'));

      await service.compact(manifest);

      const manifestIt = protobufService.loadFile(
        join(__dirname, 'fixtures', 'test-compact.manifest'),
        ProtoFileManifest,
      );
      expect(await count(manifestIt)).toBe(1000);
    } finally {
      await deleteTest('test-compact');
    }
  });
});
