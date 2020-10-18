import { count } from 'ix/asynciterable';
import { globStringToRegex } from '../utils/global-regexp.utils';
import { FileBrowserService } from './file-browser.service';

describe('FileBrowserService', () => {
  test('Search file in all directories', async () => {
    const service = new FileBrowserService();
    const value = await count(
      service.getFiles(Buffer.from(__dirname))(Buffer.alloc(0), [], [globStringToRegex('*.spec.ts')]),
    );
    expect(value).toBe(5);
  });
});
