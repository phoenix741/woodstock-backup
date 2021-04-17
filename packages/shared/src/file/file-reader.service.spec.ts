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
    index.add({
      path: Buffer.from('/file-reader.service.ts'),
      stats: {
        lastModified: Long.fromValue(123),
        size: Long.fromValue(124),
        mode: Long.fromValue(0o0100000),
      },
    });

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

  test('Test on my computer', async () => {
    await new Promise<void>((resolve, reject) => {
      const index = new IndexManifest();
      const service = new FileReader(new FileBrowserService());
      service
        .getFiles(
          index,
          Buffer.from('/home'),
          [],
          [
            globStringToRegex('lost+found'),
            globStringToRegex('phoenix/tensorflow'),
            globStringToRegex('phoenix/tmp'),
            globStringToRegex('phoenix/.composer'),
            globStringToRegex('*node_modules'),
            globStringToRegex('*mongodb/db'),
            globStringToRegex('phoenix/.ccache'),
            globStringToRegex('*mongodb/dump'),
            globStringToRegex('phoenix/usr/android-sdk'),
            globStringToRegex('phoenix/.cache'),
            globStringToRegex('phoenix/.CloudStation'),
            globStringToRegex('phoenix/.android'),
            globStringToRegex('phoenix/.AndroidStudio*'),
            globStringToRegex('phoenix/usr/android-studio'),
            globStringToRegex('*.vmdk'),
            globStringToRegex('phoenix/.nvm'),
            globStringToRegex('*.vdi'),
            globStringToRegex('phoenix/.local/share/Trash'),
            globStringToRegex('phoenix/VirtualBox VMs'),
            globStringToRegex('*mongodb/configdb'),
            globStringToRegex('phoenix/.thumbnails'),
            globStringToRegex('phoenix/.VirtualBox'),
            globStringToRegex('phoenix/.vagrant.d'),
            globStringToRegex('phoenix/vagrant'),
            globStringToRegex('phoenix/.npm'),
            globStringToRegex('phoenix/Pictures'),
            globStringToRegex('phoenix/Documents synhronisés'),
            globStringToRegex('phoenix/dwhelper'),
            globStringToRegex('phoenix/snap'),
            globStringToRegex('phoenix/.local/share/flatpak'),
            globStringToRegex('phoenix/usr/AndroidSdk'),
            globStringToRegex('public/kg/gallery'),
            globStringToRegex('*vcpkg'),
          ],
        )
        .subscribe({
          complete: () => resolve(),
          error: (err) => reject(err),
        });
    });
  },100000000);
});
