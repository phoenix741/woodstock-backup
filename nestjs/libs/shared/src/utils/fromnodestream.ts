import { AsyncIterableX } from 'ix/asynciterable';
import { safeRace } from 'ix/util/safeRace';
import type { Readable } from 'stream';

const NON_FLOWING = 0;
const READABLE = 1;
const ENDED = 2;
const ERRORED = 3;

export class ReadableStreamAsyncIterable<T> extends AsyncIterableX<T> implements AsyncIterator<T> {
  #stream: Readable;
  #defaultSize?: number;
  #state: number;
  #error: any;
  #rejectFns: Set<(err: any) => void>;
  #endPromise: Promise<void> | undefined;

  constructor(stream: Readable, size?: number) {
    super();
    this.#stream = stream;
    this.#defaultSize = size;
    this.#state = NON_FLOWING;
    this.#error = null;
    this.#rejectFns = new Set<(err: any) => void>();

    const onError = (err: any) => {
      this.#state = ERRORED;
      this.#error = err;
      for (const rejectFn of this.#rejectFns) {
        rejectFn(err);
      }
    };

    const onEnd = () => {
      this.#state = ENDED;
    };

    this.#stream['once']('error', onError);
    this.#stream['once']('end', onEnd);
  }

  [Symbol.asyncIterator](): AsyncIterator<T> {
    return this;
  }

  async next(size = this.#defaultSize): Promise<IteratorResult<T>> {
    if (this.#state === NON_FLOWING) {
      await safeRace([this.#waitReadable(), this.#waitEnd()]);
      return await this.next(size);
    }

    if (this.#state === ENDED) {
      return { done: true, value: undefined } as any as IteratorResult<T>;
    }

    if (this.#state === ERRORED) {
      throw this.#error;
    }

    const value = this.#stream['read'](size);
    if (value !== null) {
      return { done: false, value };
    } else {
      this.#state = NON_FLOWING;
      return await this.next(size);
    }
  }

  #waitReadable(): Promise<void> {
    return new Promise<void>((resolve, reject) => {
      const onReadable = () => {
        this.#state = READABLE;
        this.#rejectFns.delete(reject);
        resolve();
      };

      this.#rejectFns.add(reject);
      this.#stream['once']('readable', onReadable);
    });
  }

  #waitEnd(): Promise<void> {
    if (!this.#endPromise) {
      this.#endPromise = new Promise<void>((resolve, reject) => {
        const onEnd = () => {
          this.#state = ENDED;
          this.#rejectFns.delete(reject);
          resolve();
        };

        this.#rejectFns.add(reject);
        this.#stream['once']('end', onEnd);
      });
    }
    return this.#endPromise;
  }
}

/**
 * Creates a new async-iterable from a Node.js stream.
 *
 * @param {NodeJS.ReadableStream} stream The Node.js stream to convert to an async-iterable.
 * @param {number} [size] The size of the buffers for the stream.
 * @returns {(AsyncIterableX<T>)} An async-iterable containing data from the stream either in string or Buffer format.
 */
export function fromNodeStream<T>(stream: Readable, size?: number): AsyncIterableX<T> {
  return new ReadableStreamAsyncIterable<T>(stream, size);
}
