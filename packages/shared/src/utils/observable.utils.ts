import { EMPTY, Observable, OperatorFunction, pipe, UnaryFunction } from 'rxjs';
import { concatMapTo, filter } from 'rxjs/operators';

export function notUndefined<T>(): UnaryFunction<Observable<T | null | undefined>, Observable<T>> {
  return pipe(filter((x) => x != null) as OperatorFunction<T | null | undefined, T>);
}

export const silence = pipe(concatMapTo(EMPTY));
