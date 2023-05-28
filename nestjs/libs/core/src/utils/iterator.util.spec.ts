import { of } from 'ix/asynciterable';
import { split, SplitedAsyncIterable } from './iterator.utils';

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

      const resultArray: Record<string, string[]> = {};
      for await (const group of result) {
        resultArray[group.key] = [];
        for await (const value of group.iterable) {
          resultArray[group.key].push(value);
        }
      }
      expect(resultArray).toMatchSnapshot('resultArray');
    });

    it('with multiple header', async () => {
      const source = of(
        { chunk: 'file1', data: undefined, result: undefined },
        { chunk: undefined, data: 'data1', result: undefined },
        { chunk: undefined, data: 'data2', result: undefined },
        { chunk: undefined, data: 'data3', result: undefined },
        { chunk: undefined, data: 'data4', result: undefined },
        { chunk: undefined, data: 'data5', result: undefined },
        { chunk: undefined, data: undefined, result: 'ok' },
        { chunk: 'file2', data: undefined, result: undefined },
        { chunk: undefined, data: 'data6', result: undefined },
        { chunk: undefined, data: 'data7', result: undefined },
        { chunk: undefined, data: undefined, result: 'ok' },
        { chunk: 'file3', data: undefined, result: undefined },
        { chunk: undefined, data: undefined, result: 'failed' },
        { chunk: 'file4', data: undefined, result: undefined },
        { chunk: undefined, data: 'data8', result: undefined },
        { chunk: undefined, data: 'data9', result: undefined },
        { chunk: undefined, data: undefined, result: 'ok' },
      );
      const predicate = ({ chunk }: { chunk: string; data: string; result: string }) => chunk;
      const donePredicate = ({ result }: { chunk: string; data: string; result: string }) => result;
      const result: AsyncIterable<
        SplitedAsyncIterable<string, { chunk: string; data: string; result: string }, string>
      > = source.pipe(split(predicate, donePredicate));

      const resultArray: Record<string, { result?: string; data: string[] }> = {};
      for await (const group of result) {
        resultArray[group.key] = { result: '', data: [] };
        for await (const value of group.iterable) {
          resultArray[group.key].data.push(value.data);
        }
        resultArray[group.key].result = await group.result;
      }
      expect(resultArray).toMatchSnapshot('resultArray');
    });
  });
});
