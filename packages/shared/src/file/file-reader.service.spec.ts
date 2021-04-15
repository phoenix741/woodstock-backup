import { toArray } from 'rxjs/operators';

import { IndexManifest } from '../manifest/index-manifest.model';
import { globStringToRegex } from '../utils/global-regexp.utils';
import { FileBrowserService } from './file-browser.service';
import { FileReader } from './file-reader.service';

test('Calculate hash of the files', async () => {
  const index = new IndexManifest();
  const service = new FileReader(new FileBrowserService());

  await new Promise<void>((resolve, reject) => {
    service
      .getFiles(index, Buffer.from(__dirname), [], [globStringToRegex('*.spec.ts')])
      .pipe(toArray())
      .subscribe({
        next: (value) => {
          expect(value.length).toEqual(3);

          expect(value[0].sha256?.toString('hex')).toMatchSnapshot();
          expect(value[0].chunks?.map((c) => c.toString('hex'))).toMatchSnapshot();

          expect(value[1].sha256?.toString('hex')).toMatchSnapshot();
          expect(value[1].chunks?.map((c) => c.toString('hex'))).toMatchSnapshot();

          expect(value[2].sha256?.toString('hex')).toMatchSnapshot();
          expect(value[2].chunks?.map((c) => c.toString('hex'))).toMatchSnapshot();
        },
        complete: () => resolve(),
        error: (err) => reject(err),
      });
  });
});

test('Test', async () => {
  const index = new IndexManifest();
  const service = new FileReader(new FileBrowserService());

  console.time('test');
  await new Promise<void>((resolve, reject) => {
    service.getFiles(index, Buffer.from('/home'), [], []).subscribe({
      complete: () => resolve(),
      error: (err) => reject(err),
    });
  });
  console.timeEnd('test');
}, 1000000000);
