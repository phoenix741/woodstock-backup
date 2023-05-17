import { Test, TestingModule } from '@nestjs/testing';
import { globStringToRegex } from '@woodstock/core';
import { AsyncIterableX, from, toArray } from 'ix/asynciterable';
import * as Long from 'long';
import { IndexManifest } from '../manifest/index-manifest.model.js';
import { FileManifest } from '../protobuf/woodstock.interface.js';
import { FileBrowserService } from './file-browser.service.js';
import { FileReaderService } from './file-reader.service.js';

describe('FileReader', () => {
  let service: FileReaderService;

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
      providers: [FileReaderService, { provide: FileBrowserService, useValue: fakeFileBrowserService }],
    }).compile();

    service = app.get<FileReaderService>(FileReaderService);
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
