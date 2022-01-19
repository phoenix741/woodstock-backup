import { count, toArray } from 'ix/asynciterable';
import { map } from 'ix/asynciterable/operators';
import { globStringToRegex } from '../utils/global-regexp.utils';
import { FileBrowserService } from './file-browser.service';

process.on('unhandledRejection', (reason, p) => {
  console.log('Unhandled Rejection at: Promise', p, 'reason:', reason);
});

describe('FileBrowserService', () => {
  test('Search file in all directories', async () => {
    const service = new FileBrowserService();
    const value = await count(
      service.getFiles(Buffer.from(__dirname))(Buffer.alloc(0), [], [globStringToRegex('*.spec.ts')]),
    );
    expect(value).toBe(6);
  });

  test('Bench', async () => {
    console.time('test');
    const service = new FileBrowserService();
    try {
      const t = await toArray(
        service
          .getFiles(
            Buffer.from('/home/phoenix//Developpement/Eve/GTI/feu_vert/GTIFeuVert-develop/gtifeuvert/var/cache/dev'),
          )(Buffer.alloc(0), [], [])
          .pipe(map((f) => f.path.toString())),
      );
      console.log(t.length);
    } catch (e) {
      console.log('merde', e);
    }

    console.timeEnd('test');
  }, 6000000);
});
