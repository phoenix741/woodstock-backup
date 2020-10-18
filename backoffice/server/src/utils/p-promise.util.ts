import { throwIfAborted } from 'ix/aborterror';
import { AsyncIterableX } from 'ix/asynciterable';
import { wrapWithAbort } from 'ix/asynciterable/operators';
import { OperatorAsyncFunction } from 'ix/interfaces';

export class PPromise<T> {
  private unresolved = new Set<Promise<T>>();
  private resolved = new Array<Promise<T>>();

  constructor(private concurrent: number = 1) {}

  public async push(fn: () => Promise<T>): Promise<boolean> {
    if (this.unresolved.size >= this.concurrent) {
      await Promise.race(this.unresolved.values());
    }

    const promise = fn().then((v) => {
      this.unresolved.delete(promise);
      this.resolved.push(promise);
      return v;
    });

    this.unresolved.add(promise);

    return this.unresolved.size < this.concurrent;
  }

  public async wait(): Promise<void> {
    if (this.unresolved.size > 0) {
      await Promise.all(this.unresolved.values());
    }
  }

  public async getResolved(): Promise<T> {
    if (this.resolved.length === 0 && this.unresolved.size > 0) {
      await Promise.race(this.unresolved.values());
    }

    return this.resolved.shift();
  }

  public get size() {
    return this.resolved.length + this.unresolved.size;
  }
}

export class ConcurentMapAsyncIterable<TSource, TResult> extends AsyncIterableX<TResult> {
  constructor(
    private source: AsyncIterable<TSource>,
    private concurrent: number,
    private selector: (value: TSource, index: number, signal?: AbortSignal) => Promise<TResult> | TResult,
    private thisArg?: any,
  ) {
    super();
  }

  async *[Symbol.asyncIterator](signal?: AbortSignal) {
    throwIfAborted(signal);
    const promises = new PPromise<TResult>(this.concurrent);

    let done = false;
    const it = wrapWithAbort<TSource>(this.source, signal)[Symbol.asyncIterator]();
    let i = 0;

    const process = async () => {
      let item: IteratorResult<TSource, TResult>;
      while (!(item = await it.next()).done) {
        const notfilled = await promises.push(() => this.selector.call(this.thisArg, item.value, i++, signal));

        if (!notfilled) {
          break;
        }
      }

      if (item.done) {
        await promises.wait();
        done = true;
      }
    };

    await process();

    while (!done || promises.size) {
      const result = await promises.getResolved();
      if (result) {
        yield result;
        if (promises.size < this.concurrent) {
          await process();
        }
      }
    }
  }
}

export function concurrentMap<TSource, TResult>(
  concurrent: number,
  selector: (value: TSource, index: number, signal?: AbortSignal) => Promise<TResult> | TResult,
  thisArg?: any,
): OperatorAsyncFunction<TSource, TResult> {
  return function mapOperatorFunction(source: AsyncIterable<TSource>): AsyncIterableX<TResult> {
    return new ConcurentMapAsyncIterable<TSource, TResult>(source, concurrent, selector, thisArg);
  };
}
