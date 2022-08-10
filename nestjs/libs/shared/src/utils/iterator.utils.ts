import { AsyncIterableX, from } from 'ix/asynciterable';
import { filter } from 'ix/asynciterable/operators';
import { OperatorAsyncFunction, UnaryFunction } from 'ix/interfaces';

export function notUndefined<T>(): UnaryFunction<AsyncIterable<T | null | undefined>, AsyncIterableX<T>> {
  return function notUndefinedOperatorFunction(source: AsyncIterable<T | null | undefined>): AsyncIterableX<T> {
    return from(source).pipe(filter((x) => x != null) as OperatorAsyncFunction<T | null | undefined, T>);
  };
}
