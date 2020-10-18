import { Observable } from 'rxjs';

type ResolvePromise<T> = (value: T) => void;
type RejectPromise = (value: Error) => void;
type PromiseArguments<T> = [ResolvePromise<T>, RejectPromise];

export default function* wrapObservable<T>(observable: Observable<T>): IterableIterator<Promise<T | undefined>> {
  let done = false;

  const sentPromises: PromiseArguments<T | undefined>[] = [];
  const completedPromises: Promise<T | undefined>[] = [];

  const resolve = (item: T | undefined) => {
    if (sentPromises.length > 0) {
      const [resolve] = sentPromises.shift() || [];

      if (resolve) {
        resolve(item);
      }
    } else {
      completedPromises.push(Promise.resolve<T | undefined>(item));
    }
  };

  const reject = (error: Error) => {
    if (sentPromises.length > 0) {
      const [, reject] = sentPromises.shift() || [];

      if (reject) {
        reject(error);
      }
    } else {
      completedPromises.push(Promise.reject<T>(error));
    }
  };

  const subscription = observable.subscribe({
    next: (item: T) => resolve(item),
    error: (error: Error) => reject(error),
    complete: () => {
      done = true;
      resolve(undefined);
    },
  });

  try {
    while (!done || completedPromises.length > 0) {
      if (completedPromises.length > 0) {
        const value = completedPromises.shift();
        if (value) {
          yield value;
        }
      } else {
        yield new Promise((r: ResolvePromise<T | undefined>, e: RejectPromise) => sentPromises.push([r, e]));
      }
    }
  } finally {
    subscription.unsubscribe();
  }
}
