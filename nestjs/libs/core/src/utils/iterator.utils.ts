import { throwIfAborted } from 'ix/aborterror';
import { AsyncIterableX, create, from } from 'ix/asynciterable';
import { filter } from 'ix/asynciterable/operators';
import type { OperatorAsyncFunction, UnaryFunction } from 'ix/interfaces';

export function notUndefined<T>(): UnaryFunction<AsyncIterable<T | null | undefined>, AsyncIterableX<T>> {
  return function notUndefinedOperatorFunction(source: AsyncIterable<T | null | undefined>): AsyncIterableX<T> {
    return from(source).pipe(filter((x) => x != null) as OperatorAsyncFunction<T | null | undefined, T>);
  };
}

export interface SplitedAsyncIterable<K, T, R = unknown> {
  readonly key: K;
  readonly result: Promise<R | undefined>;
  iterable: AsyncIterableX<T>;
}

/**
 * Split an async iterable into multiple async iterables based on a predicate.
 * @param predicate The predicate to use to split the iterable.
 * @returns A function that takes an async iterable and returns an async iterable of async iterables.
 */
export function split<K, T, R = unknown>(
  predicate: (value: T) => K | undefined,
  donePredicate?: (value: T) => R | undefined,
): UnaryFunction<AsyncIterable<T>, AsyncIterableX<SplitedAsyncIterable<K, T, R>>> {
  // Read each item from the source. If the predicate returns a new key, create a new async iterable.
  // Otherwise, add the item to the current async iterable.
  return function splitOperatorFunction(source: AsyncIterable<T>): AsyncIterableX<SplitedAsyncIterable<K, T, R>> {
    const it = source[Symbol.asyncIterator]();
    let currentKey: K;

    let result: Promise<R | undefined> | undefined;
    let resultResolve: ((value: R | undefined) => void) | undefined;

    let next: IteratorResult<T, T> | undefined;

    async function processNext() {
      next = await it.next();
      if (next.value) {
        const key = predicate(next.value);
        if (key !== undefined) {
          resultResolve?.(undefined);

          currentKey = key;
          return key;
        }
        // If we have a stop predicate, we stop the current iteration
        const res = donePredicate?.(next.value);
        if (res !== undefined) {
          resultResolve?.(res);

          next = undefined;
          return false;
        }
      }

      if (next.done) {
        resultResolve?.(undefined);
      }
      return !next.done;
    }

    return create<SplitedAsyncIterable<K, T, R>>((signal) => {
      return {
        async next() {
          throwIfAborted(signal);

          // Waiting that the sub iterable is done before continuing
          if (result) {
            await result;
          }

          if (!next) {
            await processNext();
          }

          if (!next?.done) {
            result = new Promise((resolve) => {
              resultResolve = function (newResult) {
                result = undefined;
                resultResolve = undefined;
                resolve(newResult);
              };
            });

            return {
              done: false,
              value: {
                key: currentKey,
                result,
                iterable: create<T>((signal) => ({
                  async next() {
                    throwIfAborted(signal);
                    if ((await processNext()) !== true) {
                      return { done: true, value: undefined };
                    }

                    // This case should not happen (next is undefined only when processNext return false)
                    if (!next) {
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
