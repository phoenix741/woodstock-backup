import { Test, TestingModule } from '@nestjs/testing';
import { AsyncIterableX, from, toArray } from 'ix/asynciterable';
import * as Long from 'long';
import { IndexManifest } from '../manifest/index-manifest.model';
import { FileManifest } from '../models/woodstock';
import { globStringToRegex } from '../utils/global-regexp.utils';
import { FileBrowserService } from './file-browser.service';
import { FileReader } from './file-reader.service';

describe('FileReader', () => {
  let service: FileReader;

  const fakeFileBrowserService = {
    getFiles() {
      return (): AsyncIterableX<FileManifest> => {
        return from([
          {
            path: Buffer.from('/file-reader.service.ts'),
            stats: {
              lastModified: Long.fromValue(122),
              size: Long.fromValue(123),
              mode: Long.fromValue(0o0100000),
            },
          } as FileManifest,
        ]);
      };
    },
  };

  beforeEach(async () => {
    const app: TestingModule = await Test.createTestingModule({
      providers: [FileReader, { provide: FileBrowserService, useValue: fakeFileBrowserService }],
    }).compile();

    service = app.get<FileReader>(FileReader);
  });

  test('Calculate hash of the files', async () => {
    const index = new IndexManifest();
    index.add({
      path: Buffer.from('/file-reader.service.ts'),
      stats: {
        lastModified: Long.fromValue(123),
        size: Long.fromValue(124),
        mode: Long.fromValue(0o0100000),
      },
      acl: [],
      xattr: {},
      chunks: [],
    });

    const value = await toArray(service.getFiles(index, Buffer.from(__dirname), [], [globStringToRegex('*.spec.ts')]));

    expect(value).toMatchSnapshot();
  });
});
