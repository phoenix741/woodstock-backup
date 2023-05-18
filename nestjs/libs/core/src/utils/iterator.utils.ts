import { throwIfAborted } from 'ix/aborterror';
import { AsyncIterableX, create, from } from 'ix/asynciterable';
import { filter } from 'ix/asynciterable/operators';
import type { OperatorAsyncFunction, UnaryFunction } from 'ix/interfaces';

export function notUndefined<T>(): UnaryFunction<AsyncIterable<T | null | undefined>, AsyncIterableX<T>> {
  return function notUndefinedOperatorFunction(source: AsyncIterable<T | null | undefined>): AsyncIterableX<T> {
    return from(source).pipe(filter((x) => x != null) as OperatorAsyncFunction<T | null | undefined, T>);
  };
}

export interface SplitedAsyncIterable<K, T> {
  readonly key: K;
  iterable: AsyncIterableX<T>;
}

/**
 * Split an async iterable into multiple async iterables based on a predicate.
 * @param predicate The predicate to use to split the iterable.
 * @returns A function that takes an async iterable and returns an async iterable of async iterables.
 */
export function split<K, T>(
  predicate: (value: T) => K | undefined,
): UnaryFunction<AsyncIterable<T>, AsyncIterableX<SplitedAsyncIterable<K, T>>> {
  // Read each item from the source. If the predicate returns a new key, create a new async iterable.
  // Otherwise, add the item to the current async iterable.
  return function splitOperatorFunction(source: AsyncIterable<T>): AsyncIterableX<SplitedAsyncIterable<K, T>> {
    const it = source[Symbol.asyncIterator]();
    let currentKey: K;
    let next: IteratorResult<T, T>;

    async function processNext() {
      next = await it.next();
      if (next.value) {
        const key = predicate(next.value);
        if (key) {
          currentKey = key;
          return key;
        }
      }
      return !next.done;
    }

    return create<SplitedAsyncIterable<K, T>>((signal) => {
      return {
        async next() {
          throwIfAborted(signal);

          if (!next) {
            await processNext();
          }

          if (!next.done) {
            return {
              done: false,
              value: {
                key: currentKey,
                iterable: create<T>((signal) => ({
                  async next() {
                    throwIfAborted(signal);
                    if ((await processNext()) !== true) {
                      return { done: true, value: undefined };
                    }

                    return next;
                  },
                })),
              },
            };
          }

          return { done: true, value: undefined };
        },
      };
    });
  };
}