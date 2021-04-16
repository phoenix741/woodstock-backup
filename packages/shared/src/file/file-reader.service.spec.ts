import { Test, TestingModule } from '@nestjs/testing';
import { Observable, of, from } from 'rxjs';
import { toArray } from 'rxjs/operators';

import { IndexManifest } from '../manifest/index-manifest.model';
import { globStringToRegex } from '../utils/global-regexp.utils';
import { FileBrowserService } from './file-browser.service';
import { FileReader } from './file-reader.service';
import { FileManifest } from '../models';
import * as Long from 'long';

describe('FileReader', () => {
  let service: FileReader;

  const fakeFileBrowserService = {
    getFiles(sharePath: string) {
      return (backupPath: Buffer, includes: RegExp[], excludes: RegExp[]): Observable<FileManifest> => {
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

    await new Promise<void>((resolve, reject) => {
      service
        .getFiles(index, Buffer.from(__dirname), [], [globStringToRegex('*.spec.ts')])
        .pipe(toArray())
        .subscribe({
          next: (value) => {
            expect(value).toMatchSnapshot();
          },
          complete: () => resolve(),
          error: (err) => reject(err),
        });
    });
  });
});
