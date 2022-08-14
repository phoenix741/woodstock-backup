import { toArray } from 'ix/asynciterable';
import { globStringToRegex } from '../utils/global-regexp.utils.js';
import { FileBrowserService } from './file-browser.service.js';

describe('FileBrowserService', () => {
  test('Search file in all directories', async () => {
    const service = new FileBrowserService();
    const value = (
      await toArray(service.getFiles(Buffer.from(__dirname))(Buffer.alloc(0), [], [globStringToRegex('*.spec.ts')]))
    ).sort((a, b) => a.path.compare(b.path));

    expect(value).toMatchSnapshot(new Array(value.length).fill({ stats: expect.any(Object) }), 'value');
  });
});
