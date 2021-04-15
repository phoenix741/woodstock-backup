import * as Long from 'long';
import { count, reduce } from 'rxjs/operators';

import { globStringToRegex } from '../utils/global-regexp.utils';
import { FileBrowserService } from './file-browser.service';

test('Search file in all directories', async () => {
  const service = new FileBrowserService();
  const value = await service
    .getFiles(Buffer.from(__dirname))(Buffer.alloc(0), [], [globStringToRegex('*.spec.ts')])
    .pipe(count())
    .toPromise();
  expect(value).toBe(4);
});
