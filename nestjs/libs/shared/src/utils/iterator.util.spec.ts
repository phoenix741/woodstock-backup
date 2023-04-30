import { of } from 'ix/asynciterable';
import { split } from './iterator.utils';

describe('IteratorUtil', () => {
  describe('split', () => {
    it('should split an async iterable into multiple async iterables based on a predicate', async () => {
      const source = of(
        { header: '/home', manifest: undefined },
        { header: undefined, manifest: 1 },
        { header: undefined, manifest: 2 },
        { header: undefined, manifest: 3 },
        { header: undefined, manifest: 4 },
        { header: '/etc', manifest: undefined },
        { header: undefined, manifest: 5 },
        { header: undefined, manifest: 6 },
        { header: undefined, manifest: 7 },
        { header: undefined, manifest: 8 },
      );
      const predicate = ({ header }: { header: string; manifest: number }) => header;
      const result = source.pipe(split(predicate));

      const resultArray: Record<string, number[]> = {};
      for await (const group of result) {
        for await (const value of group.iterable) {
          resultArray[group.key] = resultArray[group.key] || [];
          resultArray[group.key].push(value);
        }
      }
      expect(resultArray).toMatchSnapshot('resultArray');
    });
  });
});
